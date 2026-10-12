mod commands;
mod dependencies;
mod modules;
mod orchestrators;

use std::sync::Arc;

use tauri::Manager;

use dependencies::db::connection::SqliteHandle;
use dependencies::media_store::{LocalMediaStore, MediaStorage};
use dependencies::sanitizer::{AmmoniaSanitizer, HtmlSanitizer};
use modules::card_types::{CardTypeApi, CardTypeService};
use modules::decks::{DeckApi, DeckService};
use modules::flashcards::{FlashcardApi, FlashcardService};
use modules::languages::{LanguageApi, LanguageService};
use orchestrators::{DeckOverviewOrchestrator, FlashcardEditorOrchestrator};

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
            let flashcards: Arc<dyn FlashcardApi> = Arc::new(FlashcardService::sqlite(db));
            let sanitizer: Arc<dyn HtmlSanitizer> = Arc::new(AmmoniaSanitizer::new());

            app.manage(languages.clone());
            app.manage(Arc::new(DeckOverviewOrchestrator::new(
                decks.clone(),
                languages.clone(),
                card_types.clone(),
                flashcards.clone(),
            )));
            app.manage(Arc::new(FlashcardEditorOrchestrator::new(
                decks, languages, card_types, flashcards, sanitizer, media,
            )));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::languages::list_languages,
            commands::languages::list_language_variants,
            commands::decks::list_decks,
            commands::decks::create_deck,
            commands::decks::delete_deck,
            commands::flashcards::get_flashcard_form,
            commands::flashcards::list_drafts,
            commands::flashcards::create_draft,
            commands::flashcards::update_draft,
            commands::flashcards::duplicate_draft,
            commands::flashcards::move_draft,
            commands::flashcards::delete_draft,
            commands::flashcards::publish_drafts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
