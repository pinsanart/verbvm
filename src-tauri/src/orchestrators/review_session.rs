use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveTime, Utc};

use super::error::{OrchestratorError, OrchestratorResult};
use crate::dependencies::clock::Clock;
use crate::dependencies::media_store::MediaStorage;
use crate::modules::card_types::{CardTypeApi, FieldKind, FlashcardType, Side};
use crate::modules::decks::{Deck, DeckApi, DeckSettings};
use crate::modules::flashcards::{Flashcard, FlashcardApi, MediaAttachment};
use crate::modules::languages::{Language, LanguageApi, LanguageVariant};
use crate::modules::reviews::{Rating, ReviewApi, Sm2State};

/// Where a flashcard is in its life, which decides when it is shown in a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardState {
    /// Never reviewed.
    New,
    /// Due, and its last answer was Again.
    Learning,
    /// Due, and it has been recalled before.
    Review,
}

/// How many flashcards of a type are waiting. A flashcard that is not due yet is in none of them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReviewCounts {
    pub new: usize,
    pub learning: usize,
    pub review: usize,
}

impl ReviewCounts {
    /// Nothing is waiting: the type is completed for now.
    pub fn is_empty(&self) -> bool {
        self.new + self.learning + self.review == 0
    }
}

/// A flashcard type in a deck, with what is waiting in it.
#[derive(Debug, Clone)]
pub struct TypeReview {
    pub flashcard_type: FlashcardType,
    pub counts: ReviewCounts,
}

/// A deck with its language and accent, and the types it has flashcards of.
#[derive(Debug, Clone)]
pub struct DeckReview {
    pub deck: Deck,
    pub language: Language,
    pub variant: LanguageVariant,
    /// Only the types the deck has at least one flashcard of.
    pub types: Vec<TypeReview>,
}

/// One field of a flashcard as it is shown in a review.
#[derive(Debug, Clone)]
pub struct ReviewField {
    pub key: String,
    pub label: String,
    pub side: Side,
    /// The content is sanitized HTML.
    pub is_rich: bool,
    /// `PLAIN`, `RICH` or `AUDIO`.
    pub kind: FieldKind,
    /// For an audio field, the key of the field its audio is spoken from.
    pub source_key: Option<String>,
    pub content: String,
    pub media: Vec<MediaAttachment>,
}

/// A flashcard to review. Fields with no text and no media are left out.
#[derive(Debug, Clone)]
pub struct ReviewCardView {
    pub uuid: String,
    pub flashcard_type: FlashcardType,
    pub state: CardState,
    pub fields: Vec<ReviewField>,
}

/// How long until a flashcard comes back for each answer, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Intervals {
    pub again: i64,
    pub hard: i64,
    pub good: i64,
    pub easy: i64,
}

/// The next flashcard of a session, if any, and what is still waiting in the type.
#[derive(Debug, Clone)]
pub struct ReviewNext {
    pub card: Option<ReviewCardView>,
    pub counts: ReviewCounts,
    /// BCP 47 code of the deck's accent, for the language of the text on the card.
    pub language_tag: String,
    /// What each answer would schedule for the card, when there is a card.
    pub intervals: Option<Intervals>,
    /// When there is no card: seconds until the next one of the type is due, if any is scheduled.
    pub next_due_in_seconds: Option<i64>,
    /// When there is no card: cards are waiting but the deck's daily limit of new cards or of
    /// reviews has been reached, so they come back tomorrow.
    pub daily_limit_reached: bool,
    /// The deck's setting: whether the audio of a card plays by itself.
    pub autoplay_audio: bool,
}

/// Reviews the flashcards of a deck, one type at a time.
///
/// A session is stateless: after each answer the next flashcard is asked for again, in the order
/// learning (oldest due first), review (oldest due first), then new (in creation order). That way
/// a flashcard answered Again comes back by itself once its interval has passed.
pub struct ReviewSessionOrchestrator {
    decks: Arc<dyn DeckApi>,
    languages: Arc<dyn LanguageApi>,
    card_types: Arc<dyn CardTypeApi>,
    flashcards: Arc<dyn FlashcardApi>,
    reviews: Arc<dyn ReviewApi>,
    clock: Arc<dyn Clock>,
    media: Arc<dyn MediaStorage>,
}

/// How far ahead a card that is still being learned is brought forward when nothing else is
/// waiting, so a card answered Again, or passed once, comes back in the same session instead of
/// ending it. It covers the 10 minute steps.
const LEARN_AHEAD: Duration = Duration::minutes(20);

/// The flashcards of one type split by state, each queue already in the order they are shown.
#[derive(Default)]
struct Queues<'a> {
    learning: Vec<&'a Flashcard>,
    review: Vec<&'a Flashcard>,
    new: Vec<&'a Flashcard>,
    /// How many cards a daily limit kept out of `new` and `review`.
    held_back: usize,
    /// Cards still being learned that come back soon (within [`LEARN_AHEAD`]) but are not due
    /// yet. They are shown early only when nothing else is waiting.
    learn_ahead: Vec<&'a Flashcard>,
    /// The soonest due time among the flashcards that are scheduled but not due yet.
    next_due: Option<DateTime<Utc>>,
}

impl Queues<'_> {
    fn counts(&self) -> ReviewCounts {
        ReviewCounts { new: self.new.len(), learning: self.learning.len(), review: self.review.len() }
    }
}

fn card_id(card: &Flashcard) -> OrchestratorResult<i64> {
    card.id().ok_or_else(|| OrchestratorError::Internal("flashcard was not saved".into()))
}

impl ReviewSessionOrchestrator {
    pub fn new(
        decks: Arc<dyn DeckApi>,
        languages: Arc<dyn LanguageApi>,
        card_types: Arc<dyn CardTypeApi>,
        flashcards: Arc<dyn FlashcardApi>,
        reviews: Arc<dyn ReviewApi>,
        clock: Arc<dyn Clock>,
        media: Arc<dyn MediaStorage>,
    ) -> Self {
        Self { decks, languages, card_types, flashcards, reviews, clock, media }
    }

    /// Every deck with the types it has flashcards of and what is waiting in each.
    pub fn overview(&self) -> OrchestratorResult<Vec<DeckReview>> {
        let now = self.clock.now();
        let types = self.card_types.list_types()?;
        let mut overview = Vec::new();

        for deck in self.decks.list_decks()? {
            let deck_id = deck.id().ok_or_else(|| OrchestratorError::Internal("deck was not saved".into()))?;
            let variant = self.languages.get_variant(deck.language_variant_id())?;
            let language = self.languages.get_language(variant.language_id())?;
            let cards = self.flashcards.list_flashcards(deck_id)?;

            let stored = self.decks.list_type_settings(deck_id)?;
            let mut deck_types = Vec::new();
            for flashcard_type in &types {
                let of_type: Vec<&Flashcard> =
                    cards.iter().filter(|c| Some(c.flashcard_type_id()) == flashcard_type.id()).collect();
                if of_type.is_empty() {
                    continue;
                }
                let settings = stored
                    .iter()
                    .find(|(id, _)| Some(*id) == flashcard_type.id())
                    .map_or_else(DeckSettings::default, |(_, settings)| *settings);
                let queues = self.queues(&of_type, now, settings)?;
                deck_types.push(TypeReview { flashcard_type: flashcard_type.clone(), counts: queues.counts() });
            }
            overview.push(DeckReview { deck, language, variant, types: deck_types });
        }
        Ok(overview)
    }

    /// The next flashcard to review in a deck's type, or none when nothing is waiting.
    pub fn next_card(&self, deck_name: &str, type_slug: &str) -> OrchestratorResult<ReviewNext> {
        self.present(deck_name, type_slug, None)
    }

    /// Takes back the latest answer given to a flashcard and returns that same flashcard, ready
    /// to be answered again, with the counts as they are now. Each call goes one answer further
    /// back for that flashcard. Invalid when the flashcard has no answer to take back.
    pub fn undo_answer(&self, deck_name: &str, type_slug: &str, card_uuid: &str) -> OrchestratorResult<ReviewNext> {
        let card = self.flashcards.get_flashcard(card_uuid)?;
        if !self.reviews.undo_last_review(card_id(&card)?)? {
            return Err(OrchestratorError::Invalid("there is no answer to undo for this card".into()));
        }
        self.present(deck_name, type_slug, Some(card_uuid))
    }

    /// Builds what a session shows: the next card in queue order, or `forced` when given.
    fn present(&self, deck_name: &str, type_slug: &str, forced: Option<&str>) -> OrchestratorResult<ReviewNext> {
        let now = self.clock.now();
        let deck = self.decks.get_deck_by_name(deck_name)?;
        let deck_id = deck.id().ok_or_else(|| OrchestratorError::Internal("deck was not saved".into()))?;
        let flashcard_type = self.card_types.get_type_by_slug(type_slug)?;
        let variant = self.languages.get_variant(deck.language_variant_id())?;
        let language = self.languages.get_language(variant.language_id())?;

        let cards = self.flashcards.list_flashcards(deck_id)?;
        let of_type: Vec<&Flashcard> =
            cards.iter().filter(|c| Some(c.flashcard_type_id()) == flashcard_type.id()).collect();
        let type_id = flashcard_type
            .id()
            .ok_or_else(|| OrchestratorError::Internal("flashcard type was not saved".into()))?;
        let settings = self.decks.type_settings(deck_id, type_id)?;
        let queues = self.queues(&of_type, now, settings)?;
        let counts = queues.counts();
        let autoplay_audio = settings.autoplay_audio();

        let mut counts = counts;
        let forced = match forced {
            Some(uuid) => {
                let card = of_type
                    .iter()
                    .find(|c| c.uuid() == uuid)
                    .ok_or_else(|| OrchestratorError::NotFound("flashcard not found in this deck and type".into()))?;
                let state = match self.reviews.states_for(&[card_id(card)?])?.first() {
                    None => CardState::New,
                    Some(state) if state.repetitions() == 0 => CardState::Learning,
                    Some(_) => CardState::Review,
                };
                Some((*card, state))
            }
            None => None,
        };
        let next = forced.or_else(|| queues
            .learning
            .first()
            .map(|c| (*c, CardState::Learning))
            .or_else(|| queues.review.first().map(|c| (*c, CardState::Review)))
            .or_else(|| queues.new.first().map(|c| (*c, CardState::New)))
            .or_else(|| {
                // Nothing is due: bring the cards still being learned forward, so they are
                // reviewed until they are recalled.
                counts.learning = queues.learn_ahead.len();
                queues.learn_ahead.first().map(|c| (*c, CardState::Learning))
            }));
        let language_tag = variant.bcp47_code().to_owned();
        let Some((card, state)) = next else {
            return Ok(ReviewNext {
                card: None,
                counts,
                language_tag,
                intervals: None,
                next_due_in_seconds: queues.next_due.map(|due| (due - now).num_seconds().max(0)),
                daily_limit_reached: queues.held_back > 0,
                autoplay_audio,
            });
        };

        let definitions = self.card_types.fields_for(flashcard_type.slug(), language.id())?;
        let mut fields = Vec::new();
        for definition in definitions {
            let Some(definition_id) = definition.id() else { continue };
            let content = card
                .fields()
                .iter()
                .find(|f| f.field_definition_id() == definition_id)
                .map(|f| f.content().to_owned())
                .unwrap_or_default();
            let media: Vec<MediaAttachment> = card
                .media()
                .iter()
                .filter(|m| m.field_definition_id == definition_id)
                .map(|m| m.attachment.clone())
                .collect();
            if content.trim().is_empty() && media.is_empty() {
                continue;
            }
            fields.push(ReviewField {
                key: definition.key().to_owned(),
                label: definition.label().to_owned(),
                side: definition.side(),
                is_rich: definition.is_rich(),
                kind: definition.kind(),
                source_key: definition.source_key().map(str::to_owned),
                content,
                media,
            });
        }

        let intervals = self.intervals_for(card, now)?;
        Ok(ReviewNext {
            card: Some(ReviewCardView { uuid: card.uuid().to_owned(), flashcard_type, state, fields }),
            counts,
            language_tag,
            intervals: Some(intervals),
            next_due_in_seconds: None,
            daily_limit_reached: false,
            autoplay_audio,
        })
    }

    /// Records the answer to a flashcard. `rating` is 1 (Again) to 4 (Easy).
    pub fn answer(&self, card_uuid: &str, rating: i64, duration_ms: Option<i64>) -> OrchestratorResult<()> {
        let rating = Rating::try_from(rating)?;
        let card = self.flashcards.get_flashcard(card_uuid)?;
        self.reviews.record_review(card_id(&card)?, rating, duration_ms)?;
        Ok(())
    }

    /// Where a stored media file is on disk, for the view to preview.
    pub fn media_path(&self, file_name: &str) -> Option<PathBuf> {
        self.media.path_of(file_name)
    }

    /// What each answer would schedule for a flashcard if it were answered now.
    fn intervals_for(&self, card: &Flashcard, now: DateTime<Utc>) -> OrchestratorResult<Intervals> {
        let id = card_id(card)?;
        let state = self
            .reviews
            .states_for(&[id])?
            .into_iter()
            .next()
            .unwrap_or_else(|| Sm2State::initial(id, now));
        let seconds = |rating: Rating| {
            let mut next = state.clone();
            next.review(rating, now);
            (next.due_at() - now).num_seconds().max(0)
        };
        Ok(Intervals {
            again: seconds(Rating::Again),
            hard: seconds(Rating::Hard),
            good: seconds(Rating::Good),
            easy: seconds(Rating::Easy),
        })
    }

    /// Splits flashcards of one type by state. Flashcards that are not due are left out.
    ///
    /// The daily limits of the deck for this type cut the new and review queues: a limit counts what was already
    /// done since the start of the UTC day. Cards being learned are never limited.
    fn queues<'a>(
        &self,
        cards: &[&'a Flashcard],
        now: DateTime<Utc>,
        settings: DeckSettings,
    ) -> OrchestratorResult<Queues<'a>> {
        let ids = cards.iter().map(|c| card_id(c)).collect::<OrchestratorResult<Vec<_>>>()?;
        let states: HashMap<i64, Sm2State> =
            self.reviews.states_for(&ids)?.into_iter().map(|s| (s.flashcard_id(), s)).collect();

        let mut queues = Queues::default();
        let mut learning = Vec::new();
        let mut review = Vec::new();
        let mut learn_ahead = Vec::new();
        for (card, id) in cards.iter().zip(&ids) {
            match states.get(id) {
                None => queues.new.push(*card),
                Some(state) if state.due_at() <= now => {
                    let entry = (state.due_at(), *id, *card);
                    if state.repetitions() == 0 {
                        learning.push(entry);
                    } else {
                        review.push(entry);
                    }
                }
                Some(state) => {
                    if state.repetitions() == 0 && state.due_at() <= now + LEARN_AHEAD {
                        learn_ahead.push((state.due_at(), *id, *card));
                    }
                    queues.next_due = Some(queues.next_due.map_or(state.due_at(), |due| due.min(state.due_at())));
                }
            }
        }
        learning.sort_by_key(|(due, id, _)| (*due, *id));
        review.sort_by_key(|(due, id, _)| (*due, *id));
        learn_ahead.sort_by_key(|(due, id, _)| (*due, *id));
        queues.learn_ahead = learn_ahead.into_iter().map(|(_, _, c)| c).collect();
        queues.learning = learning.into_iter().map(|(_, _, c)| c).collect();
        queues.review = review.into_iter().map(|(_, _, c)| c).collect();

        let day_start = now.date_naive().and_time(NaiveTime::MIN).and_utc();
        let done = self.reviews.activity_since(&ids, day_start)?;
        let allowed = |limit: i64, done: i64| usize::try_from((limit - done).max(0)).unwrap_or(0);
        let new_allowed = allowed(settings.new_cards_per_day(), done.new_started);
        let review_allowed = allowed(settings.max_reviews_per_day(), done.reviews_done);
        queues.held_back = queues.new.len().saturating_sub(new_allowed) + queues.review.len().saturating_sub(review_allowed);
        queues.new.truncate(new_allowed);
        queues.review.truncate(review_allowed);
        Ok(queues)
    }
}
