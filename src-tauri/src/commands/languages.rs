use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use super::error::CommandError;
use crate::modules::languages::{Language, LanguageApi, LanguageVariant};

type Api<'a> = State<'a, Arc<dyn LanguageApi>>;
type CommandResult<T> = Result<T, CommandError>;

/// A language as the view sees it. Languages are identified by `iso6393` and variants by their
/// BCP 47 code, so no database id crosses the boundary.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageDto {
    #[serde(rename = "iso6391")]
    pub iso_639_1: Option<String>,
    #[serde(rename = "iso6393")]
    pub iso_639_3: String,
    pub name: String,
    /// BCP 47 code of the default variant.
    pub default_variant_code: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageVariantDto {
    pub bcp47_code: String,
    pub variant_name: String,
    pub country_code: Option<String>,
}

pub fn variant_dto(variant: &LanguageVariant) -> LanguageVariantDto {
    LanguageVariantDto {
        bcp47_code: variant.bcp47_code().to_owned(),
        variant_name: variant.variant_name().to_owned(),
        country_code: variant.country_code().map(str::to_owned),
    }
}

/// Builds the DTO, looking up the default variant's code through the API.
fn language_dto(api: &Arc<dyn LanguageApi>, language: &Language) -> CommandResult<LanguageDto> {
    let default_variant_code = match language.default_variant_id() {
        Some(id) => Some(api.get_variant(id)?.bcp47_code().to_owned()),
        None => None,
    };
    Ok(LanguageDto {
        iso_639_1: language.iso_639_1().map(str::to_owned),
        iso_639_3: language.iso_639_3().to_owned(),
        name: language.name().to_owned(),
        default_variant_code,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLanguageInput {
    #[serde(rename = "iso6391")]
    iso_639_1: Option<String>,
    #[serde(rename = "iso6393")]
    iso_639_3: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewLanguageVariantInput {
    #[serde(rename = "languageIso6393")]
    language_iso_639_3: String,
    bcp47_code: String,
    variant_name: String,
    country_code: Option<String>,
}

#[tauri::command]
pub async fn list_languages(api: Api<'_>) -> CommandResult<Vec<LanguageDto>> {
    let languages = api.list_languages()?;
    languages.iter().map(|language| language_dto(&api, language)).collect()
}

#[tauri::command]
pub async fn create_language(api: Api<'_>, input: NewLanguageInput) -> CommandResult<LanguageDto> {
    let language = api.create_language(input.iso_639_1, input.iso_639_3, input.name)?;
    language_dto(&api, &language)
}

#[tauri::command]
pub async fn delete_language(api: Api<'_>, iso: String) -> CommandResult<()> {
    Ok(api.delete_language(&iso)?)
}

#[tauri::command]
pub async fn list_language_variants(
    api: Api<'_>,
    language_iso: String,
) -> CommandResult<Vec<LanguageVariantDto>> {
    Ok(api.list_variants(&language_iso)?.iter().map(variant_dto).collect())
}

#[tauri::command]
pub async fn create_language_variant(
    api: Api<'_>,
    input: NewLanguageVariantInput,
) -> CommandResult<LanguageVariantDto> {
    let variant = api.create_variant(
        &input.language_iso_639_3,
        input.bcp47_code,
        input.variant_name,
        input.country_code,
    )?;
    Ok(variant_dto(&variant))
}

#[tauri::command]
pub async fn set_default_language_variant(
    api: Api<'_>,
    language_iso: String,
    bcp47_code: String,
) -> CommandResult<LanguageDto> {
    let language = api.set_default_variant(&language_iso, &bcp47_code)?;
    language_dto(&api, &language)
}

#[tauri::command]
pub async fn delete_language_variant(api: Api<'_>, bcp47_code: String) -> CommandResult<()> {
    Ok(api.delete_variant(&bcp47_code)?)
}
