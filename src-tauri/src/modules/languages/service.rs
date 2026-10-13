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
///
/// Languages are identified by their ISO 639-3 code and variants by their BCP 47 code, so callers
/// outside the backend never handle database ids. `get_language` and `get_variant` take an id and
/// exist for modules that hold a foreign key.
pub trait LanguageApi: Send + Sync {
    fn create_language(
        &self,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language>;
    fn get_language(&self, id: i64) -> LanguagesResult<Language>;
    fn get_language_by_iso(&self, iso_639_3: &str) -> LanguagesResult<Language>;
    fn list_languages(&self) -> LanguagesResult<Vec<Language>>;
    fn update_language(
        &self,
        iso_639_3: &str,
        new_iso_639_1: Option<String>,
        new_iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language>;
    /// Deletes the language and its variants. Conflicts while a deck or a field definition uses it.
    fn delete_language(&self, iso_639_3: &str) -> LanguagesResult<()>;

    /// The first variant created for a language becomes its default.
    fn create_variant(
        &self,
        language_iso_639_3: &str,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant>;
    fn get_variant(&self, id: i64) -> LanguagesResult<LanguageVariant>;
    fn get_variant_by_bcp47(&self, bcp47_code: &str) -> LanguagesResult<LanguageVariant>;
    fn list_variants(&self, language_iso_639_3: &str) -> LanguagesResult<Vec<LanguageVariant>>;
    fn update_variant(
        &self,
        bcp47_code: &str,
        new_bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant>;
    /// The variant must belong to the language.
    fn set_default_variant(
        &self,
        language_iso_639_3: &str,
        bcp47_code: &str,
    ) -> LanguagesResult<Language>;
    /// Conflicts while a deck uses the variant.
    fn delete_variant(&self, bcp47_code: &str) -> LanguagesResult<()>;
}

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

    fn get_language_by_iso(&self, iso_639_3: &str) -> LanguagesResult<Language> {
        self.languages
            .find_by_iso_639_3(iso_639_3)?
            .ok_or(LanguagesError::NotFound("language"))
    }

    fn list_languages(&self) -> LanguagesResult<Vec<Language>> {
        Ok(self.languages.find_all()?)
    }

    fn update_language(
        &self,
        iso_639_3: &str,
        new_iso_639_1: Option<String>,
        new_iso_639_3: String,
        name: String,
    ) -> LanguagesResult<Language> {
        let mut language = self.get_language_by_iso(iso_639_3)?;
        language.revise(new_iso_639_1, new_iso_639_3, name)?;
        if !self.languages.update(&language)? {
            return Err(LanguagesError::NotFound("language"));
        }
        Ok(language)
    }

    fn delete_language(&self, iso_639_3: &str) -> LanguagesResult<()> {
        let language = self.get_language_by_iso(iso_639_3)?;
        let id = language.id().ok_or(LanguagesError::NotFound("language"))?;
        if self.languages.delete(id)? {
            Ok(())
        } else {
            Err(LanguagesError::NotFound("language"))
        }
    }

    fn create_variant(
        &self,
        language_iso_639_3: &str,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant> {
        let mut language = self.get_language_by_iso(language_iso_639_3)?;
        let language_id = language.id().ok_or(LanguagesError::NotFound("language"))?;
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

    fn get_variant_by_bcp47(&self, bcp47_code: &str) -> LanguagesResult<LanguageVariant> {
        self.variants
            .find_by_bcp47(bcp47_code)?
            .ok_or(LanguagesError::NotFound("language variant"))
    }

    fn list_variants(&self, language_iso_639_3: &str) -> LanguagesResult<Vec<LanguageVariant>> {
        let language = self.get_language_by_iso(language_iso_639_3)?;
        let language_id = language.id().ok_or(LanguagesError::NotFound("language"))?;
        Ok(self.variants.find_by_language(language_id)?)
    }

    fn update_variant(
        &self,
        bcp47_code: &str,
        new_bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<LanguageVariant> {
        let mut variant = self.get_variant_by_bcp47(bcp47_code)?;
        variant.revise(new_bcp47_code, variant_name, country_code)?;
        if !self.variants.update(&variant)? {
            return Err(LanguagesError::NotFound("language variant"));
        }
        Ok(variant)
    }

    fn set_default_variant(
        &self,
        language_iso_639_3: &str,
        bcp47_code: &str,
    ) -> LanguagesResult<Language> {
        let mut language = self.get_language_by_iso(language_iso_639_3)?;
        let variant = self.get_variant_by_bcp47(bcp47_code)?;
        language.assign_default_variant(&variant)?;
        self.languages.update(&language)?;
        Ok(language)
    }

    fn delete_variant(&self, bcp47_code: &str) -> LanguagesResult<()> {
        let variant = self.get_variant_by_bcp47(bcp47_code)?;
        let id = variant.id().ok_or(LanguagesError::NotFound("language variant"))?;
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

    // The migration seeds the common languages, so tests use codes it does not contain.
    fn klingon(svc: &LanguageService) -> LanguagesResult<Language> {
        svc.create_language(Some("kl".into()), "tlh".into(), "Klingon".into())
    }

    fn variant(svc: &LanguageService, iso: &str, code: &str) -> LanguageVariant {
        svc.create_variant(iso, code.into(), code.into(), Some("KL".into())).unwrap()
    }

    #[test]
    fn migration_seeds_languages_with_default_variants() {
        let (svc, path) = service();
        let french = svc.get_language_by_iso("fra").unwrap();
        let default = svc.get_variant(french.default_variant_id().unwrap()).unwrap();
        assert_eq!(default.bcp47_code(), "fr-FR");
        let english = svc.get_language_by_iso("eng").unwrap();
        let accents: Vec<_> =
            svc.list_variants("eng").unwrap().iter().map(|v| v.bcp47_code().to_owned()).collect();
        assert_eq!(accents, vec!["en-GB", "en-US"], "listed by name");
        assert_eq!(svc.get_variant(english.default_variant_id().unwrap()).unwrap().bcp47_code(), "en-US");
        assert_eq!(svc.list_languages().unwrap().len(), 5);
        let spanish: Vec<_> =
            svc.list_variants("spa").unwrap().iter().map(|v| v.bcp47_code().to_owned()).collect();
        assert_eq!(spanish, vec!["es-MX", "es-ES"], "Spanish has Mexico and Spain, listed by name");
        let default = svc.get_language_by_iso("spa").unwrap().default_variant_id().unwrap();
        assert_eq!(svc.get_variant(default).unwrap().bcp47_code(), "es-ES");
        let german = svc.get_language_by_iso("deu").unwrap();
        assert_eq!(svc.get_variant(german.default_variant_id().unwrap()).unwrap().bcp47_code(), "de-DE");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn first_variant_becomes_default_and_can_be_changed() {
        let (svc, path) = service();
        klingon(&svc).unwrap();
        let v1 = variant(&svc, "tlh", "tlh-a");
        let v2 = variant(&svc, "tlh", "tlh-b");
        assert_eq!(svc.get_language_by_iso("tlh").unwrap().default_variant_id(), v1.id());
        let lang = svc.set_default_variant("tlh", "tlh-b").unwrap();
        assert_eq!(lang.default_variant_id(), v2.id());
        assert_eq!(svc.get_language_by_iso("tlh").unwrap().default_variant_id(), v2.id());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rejects_invalid_input_and_duplicates() {
        let (svc, path) = service();
        assert!(matches!(
            svc.create_language(None, "TL".into(), "Klingon".into()),
            Err(LanguagesError::Invalid { .. })
        ));
        klingon(&svc).unwrap();
        assert!(matches!(klingon(&svc), Err(LanguagesError::Conflict(_))));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn default_variant_must_belong_to_language() {
        let (svc, path) = service();
        klingon(&svc).unwrap();
        // pt-BR belongs to Portuguese, which is seeded.
        assert!(matches!(
            svc.set_default_variant("tlh", "pt-BR"),
            Err(LanguagesError::Invalid { .. })
        ));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn lookups_by_natural_key_report_missing_entries() {
        let (svc, path) = service();
        assert!(matches!(svc.get_language_by_iso("xxx"), Err(LanguagesError::NotFound(_))));
        assert!(matches!(svc.get_variant_by_bcp47("xx-XX"), Err(LanguagesError::NotFound(_))));
        assert!(matches!(svc.list_variants("xxx"), Err(LanguagesError::NotFound(_))));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn crud_roundtrip_and_delete_cascades_variants() {
        let (svc, path) = service();
        klingon(&svc).unwrap();
        variant(&svc, "tlh", "tlh-a");
        let updated = svc
            .update_language("tlh", Some("kl".into()), "tlh".into(), " Klingon! ".into())
            .unwrap();
        assert_eq!(updated.name(), "Klingon!");
        assert_eq!(svc.get_language_by_iso("tlh").unwrap().name(), "Klingon!");
        let renamed = svc.update_variant("tlh-a", "tlh-a".into(), "Qo'noS".into(), None).unwrap();
        assert_eq!(renamed.country_code(), None);
        svc.delete_language("tlh").unwrap();
        assert!(matches!(svc.get_variant_by_bcp47("tlh-a"), Err(LanguagesError::NotFound(_))));
        assert!(matches!(svc.delete_language("tlh"), Err(LanguagesError::NotFound(_))));
        let _ = std::fs::remove_file(path);
    }
}
