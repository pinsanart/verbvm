use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use super::decks::LanguageSummaryDto;
use super::error::CommandError;
use super::languages::{variant_dto, LanguageVariantDto};
use super::media::{media_dto, MediaDto};
use crate::modules::card_types::Side;
use crate::orchestrators::{
    CardState, DeckReview, Intervals, ReviewCardView, ReviewCounts, ReviewNext, ReviewSessionOrchestrator,
};

type Api<'a> = State<'a, Arc<ReviewSessionOrchestrator>>;
type CommandResult<T> = Result<T, CommandError>;

/// How many flashcards of a type are waiting.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCountsDto {
    pub new: usize,
    pub learning: usize,
    pub review: usize,
}

fn counts_dto(counts: ReviewCounts) -> ReviewCountsDto {
    ReviewCountsDto { new: counts.new, learning: counts.learning, review: counts.review }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewTypeDto {
    pub slug: String,
    pub name: String,
    pub counts: ReviewCountsDto,
}

/// A deck as the review overview shows it. Only the types the deck has cards of are listed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewDeckDto {
    pub name: String,
    pub language: LanguageSummaryDto,
    pub variant: LanguageVariantDto,
    pub types: Vec<ReviewTypeDto>,
}

fn deck_dto(deck: &DeckReview) -> ReviewDeckDto {
    ReviewDeckDto {
        name: deck.deck.name().to_owned(),
        language: LanguageSummaryDto {
            name: deck.language.name().to_owned(),
            iso_639_3: deck.language.iso_639_3().to_owned(),
        },
        variant: variant_dto(&deck.variant),
        types: deck
            .types
            .iter()
            .map(|t| ReviewTypeDto {
                slug: t.flashcard_type.slug().to_owned(),
                name: t.flashcard_type.name().to_owned(),
                counts: counts_dto(t.counts),
            })
            .collect(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewFieldDto {
    pub key: String,
    pub label: String,
    /// `FRONT` or `BACK`.
    pub side: String,
    /// The content is sanitized HTML.
    pub is_rich: bool,
    /// `PLAIN`, `RICH` or `AUDIO`.
    pub kind: String,
    /// For an audio field, the key of the field its audio is spoken from.
    pub source_key: Option<String>,
    pub content: String,
    pub media: Vec<MediaDto>,
}

/// A flashcard to review, identified by its uuid.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewCardDto {
    pub uuid: String,
    pub type_slug: String,
    pub type_name: String,
    /// `new`, `learning` or `review`.
    pub state: String,
    pub fields: Vec<ReviewFieldDto>,
}

fn card_dto(api: &ReviewSessionOrchestrator, card: &ReviewCardView) -> ReviewCardDto {
    ReviewCardDto {
        uuid: card.uuid.clone(),
        type_slug: card.flashcard_type.slug().to_owned(),
        type_name: card.flashcard_type.name().to_owned(),
        state: match card.state {
            CardState::New => "new",
            CardState::Learning => "learning",
            CardState::Review => "review",
        }
        .to_owned(),
        fields: card
            .fields
            .iter()
            .map(|field| ReviewFieldDto {
                key: field.key.clone(),
                label: field.label.clone(),
                side: match field.side {
                    Side::Front => "FRONT",
                    Side::Back => "BACK",
                }
                .to_owned(),
                is_rich: field.is_rich,
                kind: field.kind.as_str().to_owned(),
                source_key: field.source_key.clone(),
                content: field.content.clone(),
                media: field
                    .media
                    .iter()
                    .map(|m| media_dto(m, api.media_path(m.file_name())))
                    .collect(),
            })
            .collect(),
    }
}

/// How long until the card comes back for each answer, in seconds.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntervalsDto {
    pub again: i64,
    pub hard: i64,
    pub good: i64,
    pub easy: i64,
}

fn intervals_dto(i: Intervals) -> IntervalsDto {
    IntervalsDto { again: i.again, hard: i.hard, good: i.good, easy: i.easy }
}

/// The next flashcard of a session, if any, and what is still waiting in the type.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextReviewCardDto {
    pub card: Option<ReviewCardDto>,
    pub counts: ReviewCountsDto,
    /// BCP 47 code of the deck's accent, for the `lang` of the text on the card.
    pub language_tag: String,
    /// What each answer schedules for the card. Present when there is a card.
    pub intervals: Option<IntervalsDto>,
    /// When there is no card: seconds until the next one is due, if any is scheduled.
    pub next_due_in_seconds: Option<i64>,
    /// When there is no card: cards are waiting, but the deck's daily limit has been reached.
    pub daily_limit_reached: bool,
    /// The deck's setting: the audio of a card plays by itself.
    pub autoplay_audio: bool,
}

fn next_dto(api: &ReviewSessionOrchestrator, next: &ReviewNext) -> NextReviewCardDto {
    NextReviewCardDto {
        card: next.card.as_ref().map(|c| card_dto(api, c)),
        counts: counts_dto(next.counts),
        language_tag: next.language_tag.clone(),
        intervals: next.intervals.map(intervals_dto),
        next_due_in_seconds: next.next_due_in_seconds,
        daily_limit_reached: next.daily_limit_reached,
        autoplay_audio: next.autoplay_audio,
    }
}

#[tauri::command]
pub async fn list_review_overview(api: Api<'_>) -> CommandResult<Vec<ReviewDeckDto>> {
    Ok(api.overview()?.iter().map(deck_dto).collect())
}

#[tauri::command]
pub async fn next_review_card(
    api: Api<'_>,
    deck_name: String,
    type_slug: String,
) -> CommandResult<NextReviewCardDto> {
    Ok(next_dto(&api, &api.next_card(&deck_name, &type_slug)?))
}

/// Takes back the latest answer given to a flashcard and returns that flashcard again, so the
/// view can show it as it was before the answer. Each call goes one answer further back.
#[tauri::command]
pub async fn undo_review_answer(
    api: Api<'_>,
    deck_name: String,
    type_slug: String,
    card_uuid: String,
) -> CommandResult<NextReviewCardDto> {
    Ok(next_dto(&api, &api.undo_answer(&deck_name, &type_slug, &card_uuid)?))
}

/// Records the answer to a flashcard. `rating` is 1 (Again) to 4 (Easy), and `duration_ms` is how
/// long the answer took.
#[tauri::command]
pub async fn answer_review_card(
    api: Api<'_>,
    card_uuid: String,
    rating: i64,
    duration_ms: Option<i64>,
) -> CommandResult<()> {
    Ok(api.answer(&card_uuid, rating, duration_ms)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dtos_match_the_typescript_types() {
        let deck = ReviewDeckDto {
            name: "Travel".into(),
            language: LanguageSummaryDto { name: "French".into(), iso_639_3: "fra".into() },
            variant: LanguageVariantDto {
                bcp47_code: "fr-FR".into(),
                variant_name: "French (France)".into(),
                country_code: Some("FR".into()),
            },
            types: vec![ReviewTypeDto {
                slug: "vocabulary".into(),
                name: "Vocabulary".into(),
                counts: ReviewCountsDto { new: 5, learning: 1, review: 2 },
            }],
        };
        assert_eq!(
            serde_json::to_value(deck).unwrap(),
            json!({
                "name": "Travel",
                "language": {"name": "French", "iso6393": "fra"},
                "variant": {"bcp47Code": "fr-FR", "variantName": "French (France)", "countryCode": "FR"},
                "types": [{"slug": "vocabulary", "name": "Vocabulary", "counts": {"new": 5, "learning": 1, "review": 2}}]
            })
        );

        let next = NextReviewCardDto {
            card: Some(ReviewCardDto {
                uuid: "u".into(),
                type_slug: "vocabulary".into(),
                type_name: "Vocabulary".into(),
                state: "learning".into(),
                fields: vec![ReviewFieldDto {
                    key: "phrase".into(),
                    label: "Phrase".into(),
                    side: "FRONT".into(),
                    is_rich: true,
                    kind: "RICH".into(),
                    source_key: None,
                    content: "<b>hi</b>".into(),
                    media: vec![MediaDto {
                        sha256: "ab".into(),
                        kind: "AUDIO".into(),
                        mime_type: "audio/mpeg".into(),
                        path: Some("/m/ab.mp3".into()),
                        file_name: "ab.mp3".into(),
                    }],
                }],
            }),
            counts: ReviewCountsDto { new: 0, learning: 1, review: 0 },
            language_tag: "fr-FR".into(),
            intervals: Some(IntervalsDto { again: 600, hard: 86400, good: 86400, easy: 112320 }),
            next_due_in_seconds: None,
            daily_limit_reached: false,
            autoplay_audio: true,
        };
        assert_eq!(
            serde_json::to_value(next).unwrap(),
            json!({
                "card": {
                    "uuid": "u", "typeSlug": "vocabulary", "typeName": "Vocabulary", "state": "learning",
                    "fields": [{
                        "key": "phrase", "label": "Phrase", "side": "FRONT", "isRich": true, "kind": "RICH", "sourceKey": null,
                        "content": "<b>hi</b>",
                        "media": [{"sha256": "ab", "kind": "AUDIO", "mimeType": "audio/mpeg", "path": "/m/ab.mp3", "fileName": "ab.mp3"}]
                    }]
                },
                "counts": {"new": 0, "learning": 1, "review": 0},
                "languageTag": "fr-FR",
                "intervals": {"again": 600, "hard": 86400, "good": 86400, "easy": 112320},
                "nextDueInSeconds": null,
                "dailyLimitReached": false,
                "autoplayAudio": true
            })
        );
    }

    #[test]
    fn no_next_card_is_a_null_card() {
        let none = NextReviewCardDto {
            card: None,
            counts: ReviewCountsDto { new: 0, learning: 0, review: 0 },
            language_tag: "en-US".into(),
            intervals: None,
            next_due_in_seconds: Some(600),
            daily_limit_reached: true,
            autoplay_audio: false,
        };
        assert_eq!(
            serde_json::to_value(none).unwrap(),
            json!({
                "card": null, "counts": {"new": 0, "learning": 0, "review": 0},
                "languageTag": "en-US", "intervals": null, "nextDueInSeconds": 600,
                "dailyLimitReached": true, "autoplayAudio": false
            })
        );
    }
}
