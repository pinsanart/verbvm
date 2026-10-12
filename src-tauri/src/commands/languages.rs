use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use super::error::CommandError;
use crate::modules::languages::{LanguageApi, Language, LanguageVariant};

type Api<'a> = State<'a, Arc<dyn LanguageApi>>;
type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageDto {
    id: i64,
    #[serde(rename = "iso6391")]
    iso_639_1: Option<String>,
    #[serde(rename = "iso6393")]
    iso_639_3: String,
    name: String,
    default_variant_id: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageVariantDto {
    id: i64,
    language_id: i64,
    bcp47_code: String,
    variant_name: String,
    country_code: Option<String>,
}

fn language_dto(language: Language) -> CommandResult<LanguageDto> {
    Ok(LanguageDto {
        id: language.id().ok_or_else(|| CommandError::internal("language was not saved"))?,
        iso_639_1: language.iso_639_1().map(str::to_owned),
        iso_639_3: language.iso_639_3().to_owned(),
        name: language.name().to_owned(),
        default_variant_id: language.default_variant_id(),
    })
}

fn variant_dto(variant: LanguageVariant) -> CommandResult<LanguageVariantDto> {
    Ok(LanguageVariantDto {
        id: variant.id().ok_or_else(|| CommandError::internal("variant was not saved"))?,
        language_id: variant.language_id(),
        bcp47_code: variant.bcp47_code().to_owned(),
        variant_name: variant.variant_name().to_owned(),
        country_code: variant.country_code().map(str::to_owned),
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
    language_id: i64,
    bcp47_code: String,
    variant_name: String,
    country_code: Option<String>,
}

#[tauri::command]
pub async fn list_languages(api: Api<'_>) -> CommandResult<Vec<LanguageDto>> {
    api.list_languages()?.into_iter().map(language_dto).collect()
}

#[tauri::command]
pub async fn create_language(api: Api<'_>, input: NewLanguageInput) -> CommandResult<LanguageDto> {
    language_dto(api.create_language(input.iso_639_1, input.iso_639_3, input.name)?)
}

#[tauri::command]
pub async fn delete_language(api: Api<'_>, id: i64) -> CommandResult<()> {
    Ok(api.delete_language(id)?)
}

#[tauri::command]
pub async fn list_language_variants(
    api: Api<'_>,
    language_id: i64,
) -> CommandResult<Vec<LanguageVariantDto>> {
    api.list_variants(language_id)?.into_iter().map(variant_dto).collect()
}

#[tauri::command]
pub async fn create_language_variant(
    api: Api<'_>,
    input: NewLanguageVariantInput,
) -> CommandResult<LanguageVariantDto> {
    variant_dto(api.create_variant(
        input.language_id,
        input.bcp47_code,
        input.variant_name,
        input.country_code,
    )?)
}

#[tauri::command]
pub async fn set_default_language_variant(
    api: Api<'_>,
    language_id: i64,
    variant_id: i64,
) -> CommandResult<LanguageDto> {
    language_dto(api.set_default_variant(language_id, variant_id)?)
}

#[tauri::command]
pub async fn delete_language_variant(api: Api<'_>, id: i64) -> CommandResult<()> {
    Ok(api.delete_variant(id)?)
}
