use super::error::{LanguagesError, LanguagesResult};

/// A language. Built through `new` (unsaved, no id) or `restore` (loaded from storage),
/// so its invariants hold whenever the entity is created or changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    id: Option<i64>,
    iso_639_1: Option<String>,
    iso_639_3: String,
    name: String,
    default_variant_id: Option<i64>,
}

impl Language {
    /// An unsaved language. ISO 639-1 is optional (2 lowercase letters), ISO 639-3 is required
    /// (3 lowercase letters) and the name is trimmed and must not be empty.
    pub fn new(iso_639_1: Option<String>, iso_639_3: String, name: String) -> LanguagesResult<Self> {
        Ok(Self {
            id: None,
            iso_639_1: validate_iso_639_1(iso_639_1)?,
            iso_639_3: validate_iso_639_3(iso_639_3)?,
            name: validate_name("name", name)?,
            default_variant_id: None,
        })
    }

    /// Rebuilds an entity from stored data, which was validated when it was written.
    pub(super) fn restore(
        id: i64,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
        default_variant_id: Option<i64>,
    ) -> Self {
        Self { id: Some(id), iso_639_1, iso_639_3, name, default_variant_id }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn iso_639_1(&self) -> Option<&str> {
        self.iso_639_1.as_deref()
    }

    pub fn iso_639_3(&self) -> &str {
        &self.iso_639_3
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn default_variant_id(&self) -> Option<i64> {
        self.default_variant_id
    }

    /// Replaces the editable attributes. Nothing changes if any of them is invalid.
    pub fn revise(
        &mut self,
        iso_639_1: Option<String>,
        iso_639_3: String,
        name: String,
    ) -> LanguagesResult<()> {
        let iso_639_1 = validate_iso_639_1(iso_639_1)?;
        let iso_639_3 = validate_iso_639_3(iso_639_3)?;
        let name = validate_name("name", name)?;
        self.iso_639_1 = iso_639_1;
        self.iso_639_3 = iso_639_3;
        self.name = name;
        Ok(())
    }

    /// The variant must be saved and belong to this (saved) language.
    pub fn assign_default_variant(&mut self, variant: &LanguageVariant) -> LanguagesResult<()> {
        match (self.id, variant.id) {
            (Some(language_id), Some(variant_id)) if variant.language_id == language_id => {
                self.default_variant_id = Some(variant_id);
                Ok(())
            }
            _ => Err(LanguagesError::Invalid {
                field: "variant_id",
                reason: "variant does not belong to this language",
            }),
        }
    }
}

/// A regional or script variant of a language, identified by its BCP 47 code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageVariant {
    id: Option<i64>,
    language_id: i64,
    bcp47_code: String,
    variant_name: String,
    country_code: Option<String>,
}

impl LanguageVariant {
    /// An unsaved variant. The BCP 47 code is 1-10 characters of letters, digits or `-`, and the
    /// country code, when given, is 2 uppercase letters.
    pub fn new(
        language_id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<Self> {
        Ok(Self {
            id: None,
            language_id,
            bcp47_code: validate_bcp47(bcp47_code)?,
            variant_name: validate_name("variant_name", variant_name)?,
            country_code: validate_country_code(country_code)?,
        })
    }

    pub(super) fn restore(
        id: i64,
        language_id: i64,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> Self {
        Self { id: Some(id), language_id, bcp47_code, variant_name, country_code }
    }

    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn language_id(&self) -> i64 {
        self.language_id
    }

    pub fn bcp47_code(&self) -> &str {
        &self.bcp47_code
    }

    pub fn variant_name(&self) -> &str {
        &self.variant_name
    }

    pub fn country_code(&self) -> Option<&str> {
        self.country_code.as_deref()
    }

    /// Replaces the editable attributes. The owning language never changes.
    pub fn revise(
        &mut self,
        bcp47_code: String,
        variant_name: String,
        country_code: Option<String>,
    ) -> LanguagesResult<()> {
        let bcp47_code = validate_bcp47(bcp47_code)?;
        let variant_name = validate_name("variant_name", variant_name)?;
        let country_code = validate_country_code(country_code)?;
        self.bcp47_code = bcp47_code;
        self.variant_name = variant_name;
        self.country_code = country_code;
        Ok(())
    }
}

fn validate_name(field: &'static str, value: String) -> LanguagesResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(LanguagesError::Invalid { field, reason: "must not be empty" });
    }
    Ok(value)
}

fn validate_iso_639_1(value: Option<String>) -> LanguagesResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(v) if v.len() == 2 && v.chars().all(|c| c.is_ascii_lowercase()) => Ok(Some(v)),
        Some(_) => Err(LanguagesError::Invalid {
            field: "iso_639_1",
            reason: "must be 2 lowercase letters",
        }),
    }
}

fn validate_iso_639_3(value: String) -> LanguagesResult<String> {
    if value.len() == 3 && value.chars().all(|c| c.is_ascii_lowercase()) {
        Ok(value)
    } else {
        Err(LanguagesError::Invalid { field: "iso_639_3", reason: "must be 3 lowercase letters" })
    }
}

fn validate_bcp47(value: String) -> LanguagesResult<String> {
    let valid = !value.is_empty()
        && value.len() <= 10
        && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if valid {
        Ok(value)
    } else {
        Err(LanguagesError::Invalid {
            field: "bcp47_code",
            reason: "must be 1-10 characters of letters, digits or '-'",
        })
    }
}

fn validate_country_code(value: Option<String>) -> LanguagesResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(v) if v.len() == 2 && v.chars().all(|c| c.is_ascii_uppercase()) => Ok(Some(v)),
        Some(_) => Err(LanguagesError::Invalid {
            field: "country_code",
            reason: "must be 2 uppercase letters",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_rejects_invalid_codes_and_trims_name() {
        assert!(Language::new(None, "JP".into(), "Japanese".into()).is_err());
        assert!(Language::new(Some("jpn".into()), "jpn".into(), "Japanese".into()).is_err());
        assert!(Language::new(None, "jpn".into(), "  ".into()).is_err());
        let lang = Language::new(Some("ja".into()), "jpn".into(), " Japanese ".into()).unwrap();
        assert_eq!(lang.name(), "Japanese");
        assert_eq!(lang.id(), None);
    }

    #[test]
    fn failed_revise_leaves_the_entity_untouched() {
        let mut lang = Language::new(None, "jpn".into(), "Japanese".into()).unwrap();
        assert!(lang.revise(None, "BAD".into(), "Nihongo".into()).is_err());
        assert_eq!((lang.iso_639_3(), lang.name()), ("jpn", "Japanese"));
    }

    #[test]
    fn default_variant_must_be_a_saved_variant_of_this_language() {
        let mut lang = Language::restore(1, None, "jpn".into(), "Japanese".into(), None);
        let own = LanguageVariant::restore(10, 1, "ja-JP".into(), "Japan".into(), None);
        let foreign = LanguageVariant::restore(11, 2, "pt-BR".into(), "Brazil".into(), None);
        let unsaved = LanguageVariant::new(1, "ja-Hira".into(), "Hiragana".into(), None).unwrap();
        assert!(lang.assign_default_variant(&foreign).is_err());
        assert!(lang.assign_default_variant(&unsaved).is_err());
        lang.assign_default_variant(&own).unwrap();
        assert_eq!(lang.default_variant_id(), Some(10));
    }

    #[test]
    fn variant_validates_codes() {
        assert!(LanguageVariant::new(1, "".into(), "x".into(), None).is_err());
        assert!(LanguageVariant::new(1, "ja_JP".into(), "x".into(), None).is_err());
        assert!(LanguageVariant::new(1, "ja-JP".into(), "x".into(), Some("jp".into())).is_err());
        assert!(LanguageVariant::new(1, "ja-JP".into(), "Japan".into(), Some("JP".into())).is_ok());
    }
}
