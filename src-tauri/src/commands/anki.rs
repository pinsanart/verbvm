use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use super::error::CommandError;
use crate::orchestrators::{AnkiImportOrchestrator, AnkiImportReport, DeckImport, OrchestratorError};

type CommandResult<T> = Result<T, CommandError>;

/// What was brought in for one language.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckImportDto {
    pub name: String,
    /// Stacks of two cards (vocabulary and phonetic).
    pub stacks: usize,
    /// Vocabulary cards with no pronunciation to pair with.
    pub vocabulary: usize,
    /// Phonetic cards with no vocabulary to pair with.
    pub phonetic: usize,
    pub images: usize,
    /// Why nothing was created for this language, if so.
    pub skipped: Option<String>,
}

fn deck_dto(deck: &DeckImport) -> DeckImportDto {
    DeckImportDto {
        name: deck.name.clone(),
        stacks: deck.stacks,
        vocabulary: deck.vocabulary,
        phonetic: deck.phonetic,
        images: deck.images,
        skipped: deck.skipped.clone(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnkiImportReportDto {
    pub decks: Vec<DeckImportDto>,
    pub warnings: Vec<String>,
    /// Anki had the collection open, so its latest changes may be missing.
    pub partial: bool,
}

fn report_dto(report: &AnkiImportReport) -> AnkiImportReportDto {
    AnkiImportReportDto {
        decks: report.decks.iter().map(deck_dto).collect(),
        warnings: report.warnings.clone(),
        partial: report.partial,
    }
}

/// Imports the content of an Anki collection (`collection.anki2`) as new decks. The reviews are
/// not imported. It takes a few seconds, so it runs off the main thread.
#[tauri::command]
pub async fn import_anki_collection(
    importer: State<'_, Arc<AnkiImportOrchestrator>>,
    path: String,
) -> CommandResult<AnkiImportReportDto> {
    let importer = Arc::clone(&importer);
    let path = PathBuf::from(path);
    let report = tauri::async_runtime::spawn_blocking(move || importer.import(&path))
        .await
        .map_err(|e| OrchestratorError::Internal(e.to_string()))??;
    Ok(report_dto(&report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_report_matches_the_typescript_type() {
        let report = AnkiImportReport {
            decks: vec![DeckImport { name: "Inglês".into(), stacks: 2, vocabulary: 0, phonetic: 1, images: 3, skipped: None }],
            warnings: vec!["w".into()],
            partial: true,
        };
        assert_eq!(
            serde_json::to_value(report_dto(&report)).unwrap(),
            json!({
                "decks": [{"name": "Inglês", "stacks": 2, "vocabulary": 0, "phonetic": 1, "images": 3, "skipped": null}],
                "warnings": ["w"],
                "partial": true
            })
        );
    }
}
