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
pub struct TypeSettingsDto {
    pub type_slug: String,
    /// How many cards seen for the first time a day lets in.
    pub new_cards_per_day: i64,
    /// How many answers a day lets in for cards that had been recalled before.
    pub max_reviews_per_day: i64,
    /// Whether the audio of a card plays by itself in a review.
    pub autoplay_audio: bool,
    /// When the study day starts, `HH:MM` in UTC.
    pub day_starts_at: String,
}

/// Minutes after midnight as `HH:MM`.
fn format_minute(minute: i64) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

/// `HH:MM` as minutes after midnight.
fn parse_minute(text: &str) -> Option<i64> {
    let (hours, minutes) = text.split_once(':')?;
    let digits = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_digit());
    if !digits(hours) || !digits(minutes) {
        return None;
    }
    let (hours, minutes): (i64, i64) = (hours.parse().ok()?, minutes.parse().ok()?);
    (hours < 24 && minutes < 60).then_some(hours * 60 + minutes)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckDto {
    pub name: String,
    pub description: Option<String>,
    /// UTC, `YYYY-MM-DD HH:MM:SS`.
    pub created_at: Option<String>,
    /// The study options of each flashcard type of the deck.
    pub type_settings: Vec<TypeSettingsDto>,
    pub language: LanguageSummaryDto,
    pub variant: LanguageVariantDto,
    pub stats: DeckStatsDto,
}

pub fn deck_dto(overview: &DeckOverview) -> DeckDto {
    DeckDto {
        name: overview.deck.name().to_owned(),
        description: overview.deck.description().map(str::to_owned),
        created_at: overview.deck.created_at().map(crate::dependencies::clock::format_utc),
        type_settings: overview
            .settings
            .iter()
            .map(|(flashcard_type, settings)| TypeSettingsDto {
                type_slug: flashcard_type.slug().to_owned(),
                new_cards_per_day: settings.new_cards_per_day(),
                max_reviews_per_day: settings.max_reviews_per_day(),
                autoplay_audio: settings.autoplay_audio(),
                day_starts_at: format_minute(settings.day_starts_at_minute()),
            })
            .collect(),
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
}

#[tauri::command]
pub async fn list_decks(api: Api<'_>) -> CommandResult<Vec<DeckDto>> {
    Ok(api.list()?.iter().map(deck_dto).collect())
}

#[tauri::command]
pub async fn create_deck(api: Api<'_>, input: NewDeckInput) -> CommandResult<DeckDto> {
    let overview = api.create(
        input.name,
        &input.language_iso_639_3,
        input.variant_bcp47_code.as_deref(),
        input.description,
    )?;
    Ok(deck_dto(&overview))
}

/// The editable attributes of a deck. The language and accent are fixed once it is created.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeckInput {
    name: String,
    description: Option<String>,
    /// The options of the types to change. A type that is left out keeps its own.
    #[serde(default)]
    type_settings: Vec<TypeSettingsInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeSettingsInput {
    type_slug: String,
    new_cards_per_day: i64,
    max_reviews_per_day: i64,
    autoplay_audio: bool,
    /// `HH:MM` in UTC.
    day_starts_at: String,
}

#[tauri::command]
pub async fn update_deck(api: Api<'_>, name: String, input: UpdateDeckInput) -> CommandResult<DeckDto> {
    let type_settings = input
        .type_settings
        .into_iter()
        .map(|t| {
            let minute = parse_minute(&t.day_starts_at).ok_or_else(|| {
                crate::orchestrators::OrchestratorError::Invalid("the time the day starts at must be HH:MM".into())
            })?;
            DeckSettings::new(t.new_cards_per_day, t.max_reviews_per_day, t.autoplay_audio)
                .and_then(|settings| settings.with_day_starts_at(minute))
                .map(|settings| (t.type_slug, settings))
                .map_err(crate::orchestrators::OrchestratorError::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(deck_dto(&api.update(&name, input.name, input.description, &type_settings)?))
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
            type_settings: vec![TypeSettingsDto {
                type_slug: "vocabulary".into(),
                new_cards_per_day: 20,
                max_reviews_per_day: 200,
                autoplay_audio: true,
                day_starts_at: "06:00".into(),
            }],
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
                "typeSettings": [
                    {"typeSlug": "vocabulary", "newCardsPerDay": 20, "maxReviewsPerDay": 200, "autoplayAudio": true, "dayStartsAt": "06:00"}
                ],
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
    }

    #[test]
    fn the_time_the_day_starts_at_is_written_and_read_as_hh_mm() {
        assert_eq!((format_minute(360), format_minute(0), format_minute(1_439)), ("06:00".into(), "00:00".into(), "23:59".into()));
        assert_eq!((parse_minute("06:00"), parse_minute("23:59"), parse_minute("00:05")), (Some(360), Some(1_439), Some(5)));
        for bad in ["", "6:00", "24:00", "12:60", "12-30", "ab:cd", "12:3", "+1:00"] {
            assert_eq!(parse_minute(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn update_deck_input_is_read_from_the_typescript_shape() {
        let input: UpdateDeckInput =
            serde_json::from_value(json!({
                "name": "Trips", "description": null,
                "typeSettings": [
                    {"typeSlug": "phonetic", "newCardsPerDay": 10, "maxReviewsPerDay": 100, "autoplayAudio": false, "dayStartsAt": "03:30"}
                ]
            }))
            .unwrap();
        assert_eq!((input.name.as_str(), input.description), ("Trips", None));
        let t = &input.type_settings[0];
        assert_eq!((t.type_slug.as_str(), t.new_cards_per_day, t.max_reviews_per_day, t.autoplay_audio), ("phonetic", 10, 100, false));
        assert_eq!(t.day_starts_at, "03:30");

        let without: UpdateDeckInput = serde_json::from_value(json!({"name": "T", "description": null})).unwrap();
        assert!(without.type_settings.is_empty(), "the types that are left out keep their options");
    }
}
