use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use super::decks::LanguageSummaryDto;
use super::error::CommandError;
use super::languages::{variant_dto, LanguageVariantDto};
use crate::modules::card_types::FieldDefinition;
use crate::dependencies::media_store::MediaKind;
use crate::modules::flashcards::{Direction, FlashcardDraft, MediaAttachment};
use crate::orchestrators::{DraftIssue, EditorForm, FlashcardEditorOrchestrator, PublishOutcome};

type Api<'a> = State<'a, Arc<FlashcardEditorOrchestrator>>;
type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinitionDto {
    pub key: String,
    pub label: String,
    /// `FRONT` or `BACK`.
    pub side: String,
    pub position: i64,
    pub is_required: bool,
    /// The field holds HTML and is edited in the rich editor.
    pub is_rich: bool,
    pub media_allowed: bool,
}

fn field_dto(field: &FieldDefinition) -> FieldDefinitionDto {
    FieldDefinitionDto {
        key: field.key().to_owned(),
        label: field.label().to_owned(),
        side: field.side().as_str().to_owned(),
        position: field.position(),
        is_required: field.is_required(),
        is_rich: field.is_rich(),
        media_allowed: field.media_allowed(),
    }
}

/// A flashcard type with its fields in the deck's language.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFormDto {
    pub slug: String,
    pub name: String,
    pub fields: Vec<FieldDefinitionDto>,
}

/// The deck's language and accent plus the form of every flashcard type, since a deck has one
/// list of drafts that mixes types.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashcardFormDto {
    pub deck_name: String,
    pub language: LanguageSummaryDto,
    pub variant: LanguageVariantDto,
    pub types: Vec<TypeFormDto>,
}

fn form_dto(form: &EditorForm) -> FlashcardFormDto {
    FlashcardFormDto {
        deck_name: form.deck.name().to_owned(),
        language: LanguageSummaryDto {
            name: form.language.name().to_owned(),
            iso_639_3: form.language.iso_639_3().to_owned(),
        },
        variant: variant_dto(&form.variant),
        types: form
            .types
            .iter()
            .map(|t| TypeFormDto {
                slug: t.flashcard_type.slug().to_owned(),
                name: t.flashcard_type.name().to_owned(),
                fields: t.fields.iter().map(field_dto).collect(),
            })
            .collect(),
    }
}

/// A media file attached to a field. `path` is where the file is on disk, for the view to
/// preview through the asset protocol.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    pub sha256: String,
    /// `IMAGE` or `AUDIO`.
    pub kind: String,
    pub mime_type: String,
    pub path: Option<String>,
}

fn media_dto(api: &FlashcardEditorOrchestrator, media: &MediaAttachment) -> MediaDto {
    MediaDto {
        sha256: media.sha256().to_owned(),
        kind: match media.kind() {
            MediaKind::Image => "IMAGE",
            MediaKind::Audio => "AUDIO",
        }
        .to_owned(),
        mime_type: media.mime_type().to_owned(),
        path: api.media_path(media.file_name()).map(|p| p.to_string_lossy().into_owned()),
    }
}

/// A draft, identified by its uuid. `values` maps a field key to its content and `media` to its
/// attached files.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftDto {
    pub uuid: String,
    /// The slug of the draft's flashcard type.
    pub type_slug: String,
    pub position: i64,
    pub values: BTreeMap<String, String>,
    pub media: BTreeMap<String, Vec<MediaDto>>,
}

fn draft_dto(
    api: &FlashcardEditorOrchestrator,
    slugs: &HashMap<i64, String>,
    draft: &FlashcardDraft,
) -> DraftDto {
    DraftDto {
        uuid: draft.uuid().to_owned(),
        type_slug: slugs.get(&draft.flashcard_type_id()).cloned().unwrap_or_default(),
        position: draft.position(),
        values: draft.values().clone(),
        media: draft
            .media()
            .iter()
            .map(|(key, items)| (key.clone(), items.iter().map(|m| media_dto(api, m)).collect()))
            .collect(),
    }
}

fn draft_dtos(api: &FlashcardEditorOrchestrator, drafts: &[FlashcardDraft]) -> CommandResult<Vec<DraftDto>> {
    let slugs = api.type_slugs()?;
    Ok(drafts.iter().map(|d| draft_dto(api, &slugs, d)).collect())
}

fn single_draft_dto(api: &FlashcardEditorOrchestrator, draft: &FlashcardDraft) -> CommandResult<DraftDto> {
    Ok(draft_dto(api, &api.type_slugs()?, draft))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftIssueDto {
    pub draft_uuid: String,
    pub field_key: Option<String>,
    pub message: String,
}

fn issue_dto(issue: &DraftIssue) -> DraftIssueDto {
    DraftIssueDto {
        draft_uuid: issue.draft_uuid.clone(),
        field_key: issue.field_key.clone(),
        message: issue.message.clone(),
    }
}

/// When `issues` is not empty nothing was saved.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishResultDto {
    pub published: usize,
    pub issues: Vec<DraftIssueDto>,
}

fn publish_dto(outcome: &PublishOutcome) -> PublishResultDto {
    PublishResultDto {
        published: outcome.published,
        issues: outcome.issues.iter().map(issue_dto).collect(),
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectionInput {
    Up,
    Down,
}

impl From<DirectionInput> for Direction {
    fn from(direction: DirectionInput) -> Self {
        match direction {
            DirectionInput::Up => Direction::Up,
            DirectionInput::Down => Direction::Down,
        }
    }
}

#[tauri::command]
pub async fn get_flashcard_form(api: Api<'_>, deck_name: String) -> CommandResult<FlashcardFormDto> {
    Ok(form_dto(&api.form(&deck_name)?))
}

#[tauri::command]
pub async fn list_drafts(api: Api<'_>, deck_name: String) -> CommandResult<Vec<DraftDto>> {
    draft_dtos(&api, &api.drafts(&deck_name)?)
}

#[tauri::command]
pub async fn create_draft(
    api: Api<'_>,
    deck_name: String,
    type_slug: String,
) -> CommandResult<Vec<DraftDto>> {
    draft_dtos(&api, &api.add_draft(&deck_name, &type_slug)?)
}

#[tauri::command]
pub async fn update_draft(
    api: Api<'_>,
    draft_uuid: String,
    values: BTreeMap<String, String>,
) -> CommandResult<DraftDto> {
    single_draft_dto(&api, &api.update_draft(&draft_uuid, values)?)
}

#[tauri::command]
pub async fn duplicate_draft(api: Api<'_>, draft_uuid: String) -> CommandResult<Vec<DraftDto>> {
    draft_dtos(&api, &api.duplicate_draft(&draft_uuid)?)
}

#[tauri::command]
pub async fn move_draft(
    api: Api<'_>,
    draft_uuid: String,
    direction: DirectionInput,
) -> CommandResult<Vec<DraftDto>> {
    draft_dtos(&api, &api.move_draft(&draft_uuid, direction.into())?)
}

#[tauri::command]
pub async fn delete_draft(api: Api<'_>, draft_uuid: String) -> CommandResult<Vec<DraftDto>> {
    draft_dtos(&api, &api.delete_draft(&draft_uuid)?)
}

/// Attaches a file picked on disk to a field of a draft. The backend copies it into its own
/// media folder, so the original can be moved or deleted afterwards.
#[tauri::command]
pub async fn attach_media(
    api: Api<'_>,
    draft_uuid: String,
    field_key: String,
    source_path: String,
) -> CommandResult<DraftDto> {
    let draft = api.attach_media(&draft_uuid, &field_key, std::path::Path::new(&source_path))?;
    single_draft_dto(&api, &draft)
}

#[tauri::command]
pub async fn remove_media(
    api: Api<'_>,
    draft_uuid: String,
    field_key: String,
    sha256: String,
) -> CommandResult<DraftDto> {
    let draft = api.remove_media(&draft_uuid, &field_key, &sha256)?;
    single_draft_dto(&api, &draft)
}

#[tauri::command]
pub async fn publish_drafts(api: Api<'_>, deck_name: String) -> CommandResult<PublishResultDto> {
    Ok(publish_dto(&api.publish_drafts(&deck_name)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dtos_match_the_typescript_types() {
        let field = FieldDefinitionDto {
            key: "phrase".into(),
            label: "Phrase".into(),
            side: "FRONT".into(),
            position: 0,
            is_required: true,
            is_rich: true,
            media_allowed: true,
        };
        assert_eq!(
            serde_json::to_value(field).unwrap(),
            json!({"key": "phrase", "label": "Phrase", "side": "FRONT", "position": 0,
                   "isRequired": true, "isRich": true, "mediaAllowed": true})
        );

        let mut values = BTreeMap::new();
        values.insert("phrase".to_string(), "hi".to_string());
        let mut media = BTreeMap::new();
        media.insert(
            "phrase".to_string(),
            vec![MediaDto {
                sha256: "ab".into(),
                kind: "IMAGE".into(),
                mime_type: "image/png".into(),
                path: Some("/data/media/ab.png".into()),
            }],
        );
        let draft = DraftDto { uuid: "u".into(), type_slug: "vocabulary".into(), position: 1, values, media };
        assert_eq!(
            serde_json::to_value(draft).unwrap(),
            json!({
                "uuid": "u", "typeSlug": "vocabulary", "position": 1, "values": {"phrase": "hi"},
                "media": {"phrase": [{"sha256": "ab", "kind": "IMAGE", "mimeType": "image/png", "path": "/data/media/ab.png"}]}
            })
        );

        let result = PublishResultDto {
            published: 0,
            issues: vec![DraftIssueDto {
                draft_uuid: "u".into(),
                field_key: Some("phrase".into()),
                message: "Phrase is required".into(),
            }],
        };
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!({"published": 0, "issues": [{"draftUuid": "u", "fieldKey": "phrase", "message": "Phrase is required"}]})
        );
    }

    #[test]
    fn direction_is_read_as_lowercase_text() {
        let up: DirectionInput = serde_json::from_value(json!("up")).unwrap();
        let down: DirectionInput = serde_json::from_value(json!("down")).unwrap();
        assert_eq!(Direction::from(up), Direction::Up);
        assert_eq!(Direction::from(down), Direction::Down);
        assert!(serde_json::from_value::<DirectionInput>(json!("left")).is_err());
    }

    #[test]
    fn command_errors_carry_a_kind_and_a_message() {
        let error = CommandError::from(crate::orchestrators::OrchestratorError::NotFound("deck not found".into()));
        assert_eq!(serde_json::to_value(error).unwrap(), json!({"kind": "not_found", "message": "deck not found"}));
    }
}
