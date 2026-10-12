mod commands;
mod dependencies;
mod modules;
mod orchestrators;

use std::sync::Arc;

use tauri::Manager;

use dependencies::db::connection::SqliteHandle;
use modules::languages::{LanguageApi, LanguageService};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = Arc::new(SqliteHandle::new(app.path().app_data_dir()?.join("verbvm.db"))?);

            let languages: Arc<dyn LanguageApi> = Arc::new(LanguageService::sqlite(db));
            app.manage(languages);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::languages::list_languages,
            commands::languages::create_language,
            commands::languages::delete_language,
            commands::languages::list_language_variants,
            commands::languages::create_language_variant,
            commands::languages::set_default_language_variant,
            commands::languages::delete_language_variant,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
