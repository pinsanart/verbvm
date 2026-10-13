use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use super::error::CommandError;
use crate::dependencies::pronunciation::{DictionaryLanguage, IpaDictionary};
use crate::orchestrators::OrchestratorError;

type CommandResult<T> = Result<T, CommandError>;

/// A language of the pronunciation dictionary and how many words it has.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryLanguageDto {
    /// BCP 47 code of the accent, such as `en-GB`.
    pub variant_code: String,
    pub words: u64,
}

fn language_dto(language: &DictionaryLanguage) -> DictionaryLanguageDto {
    DictionaryLanguageDto { variant_code: language.language.clone(), words: language.words }
}

/// What the dictionary holds. `supported` lists the accents it can hold, installed or not.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryStatusDto {
    pub installed: Vec<DictionaryLanguageDto>,
    pub supported: Vec<String>,
}

fn status_dto(installed: &[DictionaryLanguage]) -> DictionaryStatusDto {
    DictionaryStatusDto {
        installed: installed.iter().map(language_dto).collect(),
        supported: IpaDictionary::supported_languages().map(str::to_owned).collect(),
    }
}

#[tauri::command]
pub async fn pronunciation_status(dictionary: State<'_, Arc<IpaDictionary>>) -> CommandResult<DictionaryStatusDto> {
    let installed = dictionary.status().map_err(OrchestratorError::from)?;
    Ok(status_dto(&installed))
}

/// Builds the dictionary from the `ipa-dict` word lists in a folder. It takes a while, so it runs
/// off the main thread. A failed import leaves the dictionary that was installed.
#[tauri::command]
pub async fn import_pronunciation_dictionary(
    dictionary: State<'_, Arc<IpaDictionary>>,
    folder: String,
) -> CommandResult<DictionaryStatusDto> {
    let dictionary = Arc::clone(&dictionary);
    let folder = PathBuf::from(folder);
    let report = tauri::async_runtime::spawn_blocking(move || dictionary.import(&folder))
        .await
        .map_err(|e| OrchestratorError::Internal(e.to_string()))?
        .map_err(OrchestratorError::from)?;
    Ok(status_dto(&report.languages))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_status_matches_the_typescript_type() {
        let dto = status_dto(&[DictionaryLanguage { language: "en-US".into(), words: 3 }]);
        let value = serde_json::to_value(dto).unwrap();
        assert_eq!(value["installed"], json!([{"variantCode": "en-US", "words": 3}]));
        assert!(value["supported"].as_array().unwrap().contains(&json!("de-DE")));
    }
}
