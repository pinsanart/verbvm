use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use super::error::CommandError;
use super::languages::{variant_dto, LanguageVariantDto};
use crate::modules::decks::DeckSettings;
use crate::orchestrators::{DeckOverview, DeckOverviewOrchestrator};

type Api<'a> = State<'a, Arc<DeckOverviewOrchestrator>>;
type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageSummaryDto {
    pub name: String,
    #[serde(rename = "iso6393")]
    pub iso_639_3: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeCountDto {
    pub type_slug: String,
    pub type_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatsDto {
    pub total_cards: i64,
    pub cards_by_type: Vec<TypeCountDto>,
}

/// A deck as the view sees it, identified by its name.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckDto {
    pub name: String,
    pub description: Option<String>,
    /// UTC, `YYYY-MM-DD HH:MM:SS`.
    pub created_at: Option<String>,
    /// How many cards seen for the first time a day lets in.
    pub new_cards_per_day: i64,
    /// How many answers a day lets in for cards that had been recalled before.
    pub max_reviews_per_day: i64,
    /// Whether the audio of a card plays by itself in a review.
    pub autoplay_audio: bool,
    pub language: LanguageSummaryDto,
    pub variant: LanguageVariantDto,
    pub stats: DeckStatsDto,
}

pub fn deck_dto(overview: &DeckOverview) -> DeckDto {
    DeckDto {
        name: overview.deck.name().to_owned(),
        description: overview.deck.description().map(str::to_owned),
        created_at: overview.deck.created_at().map(crate::dependencies::clock::format_utc),
        new_cards_per_day: overview.deck.settings().new_cards_per_day(),
        max_reviews_per_day: overview.deck.settings().max_reviews_per_day(),
        autoplay_audio: overview.deck.settings().autoplay_audio(),
        language: LanguageSummaryDto {
            name: overview.language.name().to_owned(),
            iso_639_3: overview.language.iso_639_3().to_owned(),
        },
        variant: variant_dto(&overview.variant),
        stats: DeckStatsDto {
            total_cards: overview.total_cards(),
            cards_by_type: overview
                .counts
                .iter()
                .map(|(flashcard_type, count)| TypeCountDto {
                    type_slug: flashcard_type.slug().to_owned(),
                    type_name: flashcard_type.name().to_owned(),
                    count: *count,
                })
                .collect(),
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewDeckInput {
    name: String,
    #[serde(rename = "languageIso6393")]
    language_iso_639_3: String,
    /// BCP 47 code of the accent. The language's default variant is used when absent.
    variant_bcp47_code: Option<String>,
    description: Option<String>,
    /// The defaults (20 new cards, 200 reviews, audio on) are used for what is absent.
    new_cards_per_day: Option<i64>,
    max_reviews_per_day: Option<i64>,
    autoplay_audio: Option<bool>,
}

#[tauri::command]
pub async fn list_decks(api: Api<'_>) -> CommandResult<Vec<DeckDto>> {
    Ok(api.list()?.iter().map(deck_dto).collect())
}

#[tauri::command]
pub async fn create_deck(api: Api<'_>, input: NewDeckInput) -> CommandResult<DeckDto> {
    let defaults = DeckSettings::default();
    let settings = DeckSettings::new(
        input.new_cards_per_day.unwrap_or(defaults.new_cards_per_day()),
        input.max_reviews_per_day.unwrap_or(defaults.max_reviews_per_day()),
        input.autoplay_audio.unwrap_or(defaults.autoplay_audio()),
    )
    .map_err(crate::orchestrators::OrchestratorError::from)?;
    let overview = api.create(
        input.name,
        &input.language_iso_639_3,
        input.variant_bcp47_code.as_deref(),
        input.description,
        settings,
    )?;
    Ok(deck_dto(&overview))
}

/// The editable attributes of a deck. The language and accent are fixed once it is created.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeckInput {
    name: String,
    description: Option<String>,
    new_cards_per_day: i64,
    max_reviews_per_day: i64,
    autoplay_audio: bool,
}

#[tauri::command]
pub async fn update_deck(api: Api<'_>, name: String, input: UpdateDeckInput) -> CommandResult<DeckDto> {
    let settings = DeckSettings::new(input.new_cards_per_day, input.max_reviews_per_day, input.autoplay_audio)
        .map_err(crate::orchestrators::OrchestratorError::from)?;
    Ok(deck_dto(&api.update(&name, input.name, input.description, settings)?))
}

#[tauri::command]
pub async fn delete_deck(api: Api<'_>, name: String) -> CommandResult<()> {
    Ok(api.delete(&name)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deck_dto_matches_the_typescript_type() {
        let deck = DeckDto {
            name: "Travel".into(),
            description: None,
            created_at: Some("2026-10-12 08:00:00".into()),
            new_cards_per_day: 20,
            max_reviews_per_day: 200,
            autoplay_audio: true,
            language: LanguageSummaryDto { name: "English".into(), iso_639_3: "eng".into() },
            variant: LanguageVariantDto {
                bcp47_code: "en-US".into(),
                variant_name: "English (United States)".into(),
                country_code: Some("US".into()),
            },
            stats: DeckStatsDto {
                total_cards: 3,
                cards_by_type: vec![TypeCountDto {
                    type_slug: "vocabulary".into(),
                    type_name: "Vocabulary".into(),
                    count: 3,
                }],
            },
        };
        assert_eq!(
            serde_json::to_value(deck).unwrap(),
            json!({
                "name": "Travel",
                "description": null,
                "createdAt": "2026-10-12 08:00:00",
                "newCardsPerDay": 20,
                "maxReviewsPerDay": 200,
                "autoplayAudio": true,
                "language": {"name": "English", "iso6393": "eng"},
                "variant": {"bcp47Code": "en-US", "variantName": "English (United States)", "countryCode": "US"},
                "stats": {"totalCards": 3, "cardsByType": [{"typeSlug": "vocabulary", "typeName": "Vocabulary", "count": 3}]}
            })
        );
    }

    #[test]
    fn new_deck_input_is_read_from_the_typescript_shape() {
        let input: NewDeckInput = serde_json::from_value(json!({
            "name": "Travel", "languageIso6393": "eng", "variantBcp47Code": null, "description": "x"
        }))
        .unwrap();
        assert_eq!(input.language_iso_639_3, "eng");
        assert!(input.variant_bcp47_code.is_none());
        assert!(input.new_cards_per_day.is_none() && input.autoplay_audio.is_none(), "defaults when absent");

        let with: NewDeckInput = serde_json::from_value(json!({
            "name": "T", "languageIso6393": "eng", "newCardsPerDay": 5, "maxReviewsPerDay": 50, "autoplayAudio": false
        }))
        .unwrap();
        assert_eq!((with.new_cards_per_day, with.max_reviews_per_day, with.autoplay_audio), (Some(5), Some(50), Some(false)));
    }

    #[test]
    fn update_deck_input_is_read_from_the_typescript_shape() {
        let input: UpdateDeckInput =
            serde_json::from_value(json!({
                "name": "Trips", "description": null,
                "newCardsPerDay": 10, "maxReviewsPerDay": 100, "autoplayAudio": false
            }))
            .unwrap();
        assert_eq!((input.name.as_str(), input.description), ("Trips", None));
        assert_eq!((input.new_cards_per_day, input.max_reviews_per_day, input.autoplay_audio), (10, 100, false));
    }
}
