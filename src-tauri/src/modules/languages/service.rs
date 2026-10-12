use std::sync::Arc;

use super::error::{LanguagesError, LanguagesResult};
use super::model::{Language, LanguageVariant};
use super::repository::{
    LanguageRepository, LanguageVariantRepository, SqliteLanguageRepository,
    SqliteLanguageVariantRepository,
};
use crate::dependencies::db::connection::SqliteHandle;

/// Public API of the languages module. Other modules depend on this trait, never on
/// `LanguageService` or the repositories.
pub trait LanguageApi: Send + Sync {
    fn create_language(
        &self,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language>;
    fn get_language(&self, id: i64) -> LanguagesResult<Language>;
    fn list_languages(&self) -> LanguagesResult<Vec<Language>>;
    fn update_language(
        &self,
        id: i64,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language>;
    /// Deletes the language and its variants. Conflicts while a deck or a field definition uses it.
    fn delete_language(&self, id: i64) -> LanguagesResult<()>;

    /// The first variant created for a language becomes its default.
    fn create_variant(
        &self,
        language_id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant>;
    fn get_variant(&self, id: i64) -> LanguagesResult<LanguageVariant>;
    fn list_variants(&self, language_id: i64) -> LanguagesResult<Vec<LanguageVariant>>;
    fn update_variant(
        &self,
        id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant>;
    /// The variant must belong to the language.
    fn set_default_variant(&self, language_id: i64, variant_id: i64) -> LanguagesResult<Language>;
    /// Conflicts while a deck uses the variant.
    fn delete_variant(&self, id: i64) -> LanguagesResult<()>;
}

/// Business rules of the module, over any [`LanguageRepository`] and [`LanguageVariantRepository`].
pub struct LanguageService {
    languages: Arc<dyn LanguageRepository>,
    variants: Arc<dyn LanguageVariantRepository>,
}

impl LanguageService {
    pub fn new(
        languages: Arc<dyn LanguageRepository>,
        variants: Arc<dyn LanguageVariantRepository>,
    ) -> Self {
        Self { languages, variants }
    }

    /// Service backed by the SQLite repositories.
    pub fn sqlite(db: Arc<SqliteHandle>) -> Self {
        Self::new(
            Arc::new(SqliteLanguageRepository::new(db.clone())),
            Arc::new(SqliteLanguageVariantRepository::new(db)),
        )
    }
}

impl LanguageApi for LanguageService {
    fn create_language(
        &self,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language> {
        let language = Language::new(iso_639_1, iso_639_3, name)?;
        Ok(self.languages.insert(&language)?)
    }

    fn get_language(&self, id: i64) -> LanguagesResult<Language> {
        self.languages.find(id)?.ok_or(LanguagesError::NotFound("language"))
    }

    fn list_languages(&self) -> LanguagesResult<Vec<Language>> {
        Ok(self.languages.find_all()?)
    }

    fn update_language(
        &self,
        id: i64,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language> {
        let mut language = self.get_language(id)?;
        language.revise(iso_639_1, iso_639_3, name)?;
        if !self.languages.update(&language)? {
            return Err(LanguagesError::NotFound("language"));
        }
        Ok(language)
    }

    fn delete_language(&self, id: i64) -> LanguagesResult<()> {
        if self.languages.delete(id)? {
            Ok(())
        } else {
            Err(LanguagesError::NotFound("language"))
        }
    }

    fn create_variant(
        &self,
        language_id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant> {
        let mut language = self.get_language(language_id)?;
        let variant = LanguageVariant::new(language_id, bcp47_code, variant_name, country_code)?;
        let variant = self.variants.insert(&variant)?;
        if language.default_variant_id().is_none() {
            language.assign_default_variant(&variant)?;
            self.languages.update(&language)?;
        }
        Ok(variant)
    }

    fn get_variant(&self, id: i64) -> LanguagesResult<LanguageVariant> {
        self.variants.find(id)?.ok_or(LanguagesError::NotFound("language variant"))
    }

    fn list_variants(&self, language_id: i64) -> LanguagesResult<Vec<LanguageVariant>> {
        self.get_language(language_id)?;
        Ok(self.variants.find_by_language(language_id)?)
    }

    fn update_variant(
        &self,
        id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant> {
        let mut variant = self.get_variant(id)?;
        variant.revise(bcp47_code, variant_name, country_code)?;
        if !self.variants.update(&variant)? {
            return Err(LanguagesError::NotFound("language variant"));
        }
        Ok(variant)
    }

    fn set_default_variant(&self, language_id: i64, variant_id: i64) -> LanguagesResult<Language> {
        let mut language = self.get_language(language_id)?;
        let variant = self.get_variant(variant_id)?;
        language.assign_default_variant(&variant)?;
        self.languages.update(&language)?;
        Ok(language)
    }

    fn delete_variant(&self, id: i64) -> LanguagesResult<()> {
        if self.variants.delete(id)? {
            Ok(())
        } else {
            Err(LanguagesError::NotFound("language variant"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> (LanguageService, std::path::PathBuf) {
        let path = std::env::temp_dir()
            .join(format!("verbvm-test-{}-{:?}.db", std::process::id(), std::thread::current().id()));
        let _ = std::fs::remove_file(&path);
        let db = Arc::new(SqliteHandle::new(&path).unwrap());
        (LanguageService::sqlite(db), path)
    }

    fn japanese(svc: &LanguageService) -> LanguagesResult<Language> {
        svc.create_language(Some("ja".into()), "jpn".into(), "Japanese".into())
    }

    fn variant(svc: &LanguageService, language_id: i64, code: &str) -> LanguageVariant {
        svc.create_variant(language_id, code.into(), code.into(), Some("JP".into())).unwrap()
    }

    #[test]
    fn first_variant_becomes_default_and_can_be_changed() {
        let (svc, path) = service();
        let lang = japanese(&svc).unwrap();
        let lang_id = lang.id().unwrap();
        let v1 = variant(&svc, lang_id, "ja-JP");
        let v2 = variant(&svc, lang_id, "ja-Hira");
        assert_eq!(svc.get_language(lang_id).unwrap().default_variant_id(), v1.id());
        let lang = svc.set_default_variant(lang_id, v2.id().unwrap()).unwrap();
        assert_eq!(lang.default_variant_id(), v2.id());
        assert_eq!(svc.get_language(lang_id).unwrap().default_variant_id(), v2.id());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_invalid_input_and_duplicates() {
        let (svc, path) = service();
        assert!(matches!(
            svc.create_language(None, "JP".into(), "Japanese".into()),
            Err(LanguagesError::Invalid { .. })
        ));
        japanese(&svc).unwrap();
        assert!(matches!(japanese(&svc), Err(LanguagesError::Conflict(_))));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn default_variant_must_belong_to_language() {
        let (svc, path) = service();
        let ja = japanese(&svc).unwrap();
        let pt = svc.create_language(Some("pt".into()), "por".into(), "Portuguese".into()).unwrap();
        let pt_br = variant(&svc, pt.id().unwrap(), "pt-BR");
        assert!(matches!(
            svc.set_default_variant(ja.id().unwrap(), pt_br.id().unwrap()),
            Err(LanguagesError::Invalid { .. })
        ));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn crud_roundtrip_and_delete_cascades_variants() {
        let (svc, path) = service();
        let lang = japanese(&svc).unwrap();
        let lang_id = lang.id().unwrap();
        let v = variant(&svc, lang_id, "ja-JP");
        let updated = svc
            .update_language(lang_id, Some("ja".into()), "jpn".into(), " Nihongo ".into())
            .unwrap();
        assert_eq!(updated.name(), "Nihongo");
        assert_eq!(svc.get_language(lang_id).unwrap().name(), "Nihongo");
        let renamed = svc.update_variant(v.id().unwrap(), "ja-JP".into(), "Japan".into(), None).unwrap();
        assert_eq!(renamed.country_code(), None);
        svc.delete_language(lang_id).unwrap();
        assert!(matches!(svc.get_variant(v.id().unwrap()), Err(LanguagesError::NotFound(_))));
        assert!(matches!(svc.delete_language(lang_id), Err(LanguagesError::NotFound(_))));
        let _ = std::fs::remove_file(path);
    }
}
