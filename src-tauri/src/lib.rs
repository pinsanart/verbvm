mod commands;
mod dependencies;
mod modules;
mod orchestrators;

use std::sync::Arc;

use tauri::Manager;

use dependencies::clock::{Clock, SystemClock};
use dependencies::db::connection::SqliteHandle;
use dependencies::media_store::{LocalMediaStore, MediaStorage};
use dependencies::sanitizer::{AmmoniaSanitizer, HtmlSanitizer};
use dependencies::speech::{GoogleTranslateSynthesizer, SpeechSynthesizer};
use modules::card_types::{CardTypeApi, CardTypeService};
use modules::decks::{DeckApi, DeckService};
use modules::flashcards::{FlashcardApi, FlashcardService};
use modules::languages::{LanguageApi, LanguageService};
use modules::reviews::{ReviewApi, ReviewService};
use orchestrators::{DeckOverviewOrchestrator, FlashcardEditorOrchestrator, ReviewSessionOrchestrator};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let db = Arc::new(SqliteHandle::new(data_dir.join("verbvm.db"))?);
            // The asset protocol scope in tauri.conf.json is limited to this folder.
            let media: Arc<dyn MediaStorage> = Arc::new(LocalMediaStore::new(data_dir.join("media")));

            let languages: Arc<dyn LanguageApi> = Arc::new(LanguageService::sqlite(db.clone()));
            let decks: Arc<dyn DeckApi> = Arc::new(DeckService::sqlite(db.clone()));
            let card_types: Arc<dyn CardTypeApi> = Arc::new(CardTypeService::sqlite(db.clone()));
            let flashcards: Arc<dyn FlashcardApi> = Arc::new(FlashcardService::sqlite(db.clone()));
            let clock: Arc<dyn Clock> = Arc::new(SystemClock);
            let reviews: Arc<dyn ReviewApi> = Arc::new(ReviewService::sqlite(db, clock.clone()));
            let sanitizer: Arc<dyn HtmlSanitizer> = Arc::new(AmmoniaSanitizer::new());
            // Google Translate is the only provider for now. Another one only needs to implement
            // `SpeechSynthesizer` and be built here.
            let speech: Arc<dyn SpeechSynthesizer> = Arc::new(GoogleTranslateSynthesizer::new());

            app.manage(languages.clone());
            app.manage(Arc::new(DeckOverviewOrchestrator::new(
                decks.clone(),
                languages.clone(),
                card_types.clone(),
                flashcards.clone(),
            )));
            app.manage(Arc::new(FlashcardEditorOrchestrator::new(
                decks.clone(),
                languages.clone(),
                card_types.clone(),
                flashcards.clone(),
                sanitizer,
                media.clone(),
                speech,
            )));
            app.manage(Arc::new(ReviewSessionOrchestrator::new(
                decks, languages, card_types, flashcards, reviews, clock, media,
            )));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::languages::list_languages,
            commands::languages::list_language_variants,
            commands::decks::list_decks,
            commands::decks::create_deck,
            commands::decks::update_deck,
            commands::decks::delete_deck,
            commands::flashcards::get_flashcard_form,
            commands::flashcards::list_drafts,
            commands::flashcards::create_draft,
            commands::flashcards::update_draft,
            commands::flashcards::duplicate_draft,
            commands::flashcards::move_draft,
            commands::flashcards::delete_draft,
            commands::flashcards::attach_media,
            commands::flashcards::generate_media,
            commands::flashcards::remove_media,
            commands::flashcards::publish_drafts,
            commands::media::read_media,
            commands::flashcards::list_saved_cards,
            commands::flashcards::update_saved_card,
            commands::flashcards::attach_saved_card_media,
            commands::flashcards::generate_saved_card_media,
            commands::flashcards::remove_saved_card_media,
            commands::flashcards::delete_saved_card,
            commands::review::list_review_overview,
            commands::review::next_review_card,
            commands::review::answer_review_card,
            commands::review::undo_review_answer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
