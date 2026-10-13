//! [`PronunciationSource`] over the `ipa-dict` word lists (open-dict-data), which are text files
//! with `word<TAB>/ipa/[, /ipa/...]` on each line.
//!
//! The lists are big (millions of words) and are not part of the repository. An import turns them
//! into one indexed SQLite file next to the app database, which is then read-only.

use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use super::{PronunciationError, PronunciationSource};

/// The languages the app has pronunciation for: BCP 47 code and the name of the `ipa-dict` file.
const FILES: [(&str, &str); 7] = [
    ("en-US", "en_US"),
    ("en-GB", "en_UK"),
    ("fr-FR", "fr_FR"),
    ("es-ES", "es_ES"),
    ("es-MX", "es_MX"),
    ("de-DE", "de"),
    ("pt-BR", "pt_BR"),
];

/// A language of the dictionary and how many words it has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryLanguage {
    pub language: String,
    pub words: u64,
}

/// What an import stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub languages: Vec<DictionaryLanguage>,
}

pub struct IpaDictionary {
    path: PathBuf,
    connection: Mutex<Option<Connection>>,
}

impl IpaDictionary {
    /// The dictionary stored at `path` (it need not exist yet).
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into(), connection: Mutex::new(None) }
    }

    /// The BCP 47 codes this dictionary can hold.
    pub fn supported_languages() -> impl Iterator<Item = &'static str> {
        FILES.iter().map(|(tag, _)| *tag)
    }

    /// The installed languages with their word counts, empty when nothing is installed.
    pub fn status(&self) -> Result<Vec<DictionaryLanguage>, PronunciationError> {
        match self.with_connection(|conn| {
            conn.prepare("SELECT language, words FROM languages ORDER BY language")?
                .query_map([], |row| Ok(DictionaryLanguage { language: row.get(0)?, words: row.get::<_, i64>(1)? as u64 }))?
                .collect::<rusqlite::Result<Vec<_>>>()
        }) {
            Err(PronunciationError::NotInstalled) => Ok(Vec::new()),
            other => other,
        }
    }

    /// Builds the dictionary from the `ipa-dict` text files found in `folder`, in `folder/data` or
    /// in `folder/*/data`. The result replaces the installed one only when the whole import
    /// worked, so a failed import leaves what was there.
    pub fn import(&self, folder: &Path) -> Result<ImportReport, PronunciationError> {
        let data = locate_data(folder).ok_or(PronunciationError::NothingToImport)?;
        let part = self.path.with_extension("db.part");
        let _ = fs::remove_file(&part);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let report = match build(&part, &data) {
            Ok(report) => report,
            Err(error) => {
                let _ = fs::remove_file(&part);
                return Err(error);
            }
        };

        // Close the old file before replacing it, then open the new one on the next lookup.
        let mut guard = self.connection.lock().unwrap_or_else(|p| p.into_inner());
        *guard = None;
        fs::rename(&part, &self.path)?;
        Ok(report)
    }

    fn with_connection<T>(
        &self,
        f: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Result<T, PronunciationError> {
        let mut guard = self.connection.lock().unwrap_or_else(|p| p.into_inner());
        if guard.is_none() {
            if !self.path.exists() {
                return Err(PronunciationError::NotInstalled);
            }
            *guard = Some(Connection::open_with_flags(&self.path, OpenFlags::SQLITE_OPEN_READ_ONLY)?);
        }
        Ok(f(guard.as_ref().expect("opened above"))?)
    }
}

impl PronunciationSource for IpaDictionary {
    fn lookup(&self, language: &str, word: &str) -> Result<Option<String>, PronunciationError> {
        self.with_connection(|conn| {
            let installed: bool = conn
                .query_row("SELECT 1 FROM languages WHERE language = ?1", [language], |_| Ok(true))
                .optional()?
                .unwrap_or(false);
            if !installed {
                return Ok(Err(language.to_owned()));
            }
            let find = |candidate: &str| -> rusqlite::Result<Option<String>> {
                conn.query_row(
                    "SELECT ipa FROM entries WHERE language = ?1 AND word = ?2",
                    params![language, candidate],
                    |row| row.get(0),
                )
                .optional()
            };
            let mut found = find(word)?;
            if found.is_none() {
                let lower = word.to_lowercase();
                if lower != word {
                    found = find(&lower)?;
                }
            }
            Ok(Ok(found))
        })?
        .map_err(PronunciationError::Unsupported)
    }
}

/// The folder that holds the word lists: `folder` itself, `folder/data`, or `data` in a folder
/// inside it (how the project's zip unpacks).
fn locate_data(folder: &Path) -> Option<PathBuf> {
    let has_lists = |dir: &Path| FILES.iter().any(|(_, file)| dir.join(format!("{file}.txt")).is_file());
    let mut candidates = vec![folder.to_path_buf(), folder.join("data")];
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                candidates.push(entry.path().join("data"));
            }
        }
    }
    candidates.into_iter().find(|dir| has_lists(dir))
}

/// The first pronunciation of an entry, wrapped in slashes. The British English list has invisible
/// zero-width joiners inside its diphthongs, which are dropped so the text compares and copies as
/// it looks.
fn first_pronunciation(ipa: &str) -> Option<String> {
    let cleaned = ipa.replace('\u{200d}', "");
    let first = cleaned.split(", ").next()?.trim();
    let inner = first.trim_matches('/');
    (!inner.is_empty()).then(|| format!("/{inner}/"))
}

fn build(part: &Path, data: &Path) -> Result<ImportReport, PronunciationError> {
    let mut conn = Connection::open(part)?;
    // The file is rebuilt from scratch and replaced only when complete, so no journal is needed.
    conn.execute_batch(
        "PRAGMA journal_mode = OFF; PRAGMA synchronous = OFF;
         CREATE TABLE entries (
             language TEXT NOT NULL,
             word     TEXT NOT NULL,
             ipa      TEXT NOT NULL,
             PRIMARY KEY (language, word)
         ) WITHOUT ROWID;
         CREATE TABLE languages (language TEXT PRIMARY KEY, words INTEGER NOT NULL);",
    )?;

    let mut languages = Vec::new();
    let tx = conn.transaction()?;
    {
        let mut insert = tx.prepare("INSERT OR IGNORE INTO entries (language, word, ipa) VALUES (?1, ?2, ?3)")?;
        for (tag, file) in FILES {
            let path = data.join(format!("{file}.txt"));
            if !path.is_file() {
                continue;
            }
            let mut words = 0u64;
            for line in BufReader::new(File::open(&path)?).lines() {
                let line = line?;
                let Some((word, ipa)) = line.split_once('\t') else { continue };
                let (word, Some(ipa)) = (word.trim(), first_pronunciation(ipa)) else { continue };
                if word.is_empty() {
                    continue;
                }
                words += insert.execute(params![tag, word, ipa])? as u64;
            }
            tx.execute("INSERT INTO languages (language, words) VALUES (?1, ?2)", params![tag, words as i64])?;
            languages.push(DictionaryLanguage { language: tag.to_owned(), words });
        }
    }
    tx.commit()?;
    if languages.is_empty() {
        return Err(PronunciationError::NothingToImport);
    }
    Ok(ImportReport { languages })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "verbvm-ipa-{name}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn write(&self, relative: &str, text: &str) {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn lists(dir: &TempDir) {
        dir.write("source/ipa-dict/data/en_US.txt", "hello\t/həˈɫoʊ/, /hɛˈɫoʊ/\neveryone\t/ˈɛvɹiˌwən/\n\n");
        dir.write("source/ipa-dict/data/de.txt", "Hallo\t/haˈloː/\nhallo\t/haˈloː/, /ˈhalo/\nWelt\tvɛlt\n");
        dir.write("source/ipa-dict/data/it.txt", "ciao\t/tʃao/\n");
        dir.write("source/ipa-dict/data/en_UK.txt", "hello\t/həlˈə\u{200d}ʊ/\n");
    }

    #[test]
    fn nothing_is_installed_until_an_import() {
        let tmp = TempDir::new("none");
        let dictionary = IpaDictionary::new(tmp.0.join("ipa.db"));
        assert!(matches!(dictionary.lookup("en-US", "hello"), Err(PronunciationError::NotInstalled)));
        assert!(dictionary.status().unwrap().is_empty());
    }

    #[test]
    fn imports_the_supported_lists_wherever_the_folder_points() {
        let tmp = TempDir::new("import");
        lists(&tmp);
        for folder in ["source", "source/ipa-dict", "source/ipa-dict/data"] {
            let dictionary = IpaDictionary::new(tmp.0.join(format!("{}.db", folder.replace('/', "-"))));
            let report = dictionary.import(&tmp.0.join(folder)).unwrap();
            let tags: Vec<_> = report.languages.iter().map(|l| (l.language.as_str(), l.words)).collect();
            assert_eq!(tags, vec![("en-US", 2), ("en-GB", 1), ("de-DE", 3)], "{folder}: Italian is not a supported language");
            let mut listed = report.languages.clone();
            listed.sort_by(|a, b| a.language.cmp(&b.language));
            assert_eq!(dictionary.status().unwrap(), listed, "the status lists the installed languages by code");
        }
    }

    #[test]
    fn lookup_is_exact_then_lower_case_and_keeps_the_first_pronunciation() {
        let tmp = TempDir::new("lookup");
        lists(&tmp);
        let dictionary = IpaDictionary::new(tmp.0.join("ipa.db"));
        dictionary.import(&tmp.0.join("source")).unwrap();

        assert_eq!(dictionary.lookup("en-US", "hello").unwrap().as_deref(), Some("/həˈɫoʊ/"));
        assert_eq!(dictionary.lookup("en-GB", "hello").unwrap().as_deref(), Some("/həlˈəʊ/"), "zero-width joiners are dropped");
        assert_eq!(dictionary.lookup("en-US", "Hello").unwrap().as_deref(), Some("/həˈɫoʊ/"), "lower case is tried");
        assert_eq!(dictionary.lookup("en-US", "nonexistent").unwrap(), None);
        assert_eq!(dictionary.lookup("de-DE", "Hallo").unwrap().as_deref(), Some("/haˈloː/"));
        assert_eq!(dictionary.lookup("de-DE", "hallo").unwrap().as_deref(), Some("/haˈloː/"), "exact entries come first");
        assert_eq!(dictionary.lookup("de-DE", "Welt").unwrap().as_deref(), Some("/vɛlt/"), "slashes are added");
        assert!(matches!(dictionary.lookup("fr-FR", "bonjour"), Err(PronunciationError::Unsupported(tag)) if tag == "fr-FR"));
    }

    #[test]
    fn a_failed_import_keeps_what_was_installed() {
        let tmp = TempDir::new("keep");
        lists(&tmp);
        let dictionary = IpaDictionary::new(tmp.0.join("ipa.db"));
        dictionary.import(&tmp.0.join("source")).unwrap();

        fs::create_dir_all(tmp.0.join("empty")).unwrap();
        assert!(matches!(dictionary.import(&tmp.0.join("empty")), Err(PronunciationError::NothingToImport)));
        assert_eq!(dictionary.lookup("en-US", "hello").unwrap().as_deref(), Some("/həˈɫoʊ/"));
        assert!(!tmp.0.join("ipa.db.part").exists(), "no half-written file is left");
    }

    #[test]
    fn a_new_import_replaces_the_old_one() {
        let tmp = TempDir::new("replace");
        lists(&tmp);
        let dictionary = IpaDictionary::new(tmp.0.join("ipa.db"));
        dictionary.import(&tmp.0.join("source")).unwrap();
        assert!(dictionary.lookup("en-US", "goodbye").unwrap().is_none());

        tmp.write("source/ipa-dict/data/en_US.txt", "goodbye\t/ɡʊdˈbaɪ/\n");
        dictionary.import(&tmp.0.join("source")).unwrap();
        assert_eq!(dictionary.lookup("en-US", "goodbye").unwrap().as_deref(), Some("/ɡʊdˈbaɪ/"));
        assert!(dictionary.lookup("en-US", "hello").unwrap().is_none());
    }

    /// Imports the real word lists. Run it by hand, with the folder in the environment:
    /// `VERBVM_IPA_DICT_DIR=/path/to/ipa-dict cargo test real_word_lists -- --ignored --nocapture`.
    #[test]
    #[ignore = "needs the ipa-dict word lists, which are not in the repository"]
    fn real_word_lists() {
        let folder = std::env::var("VERBVM_IPA_DICT_DIR").expect("set VERBVM_IPA_DICT_DIR");
        let tmp = TempDir::new("real");
        let dictionary = IpaDictionary::new(tmp.0.join("ipa.db"));
        let started = std::time::Instant::now();
        let report = dictionary.import(Path::new(&folder)).unwrap();
        println!("imported in {:?}", started.elapsed());
        for language in &report.languages {
            println!("{}: {} words", language.language, language.words);
        }
        println!("ipa.db is {} MB", fs::metadata(tmp.0.join("ipa.db")).unwrap().len() / 1_000_000);
        for (language, word) in [
            ("en-US", "hello"), ("en-GB", "hello"), ("en-GB", "everyone"), ("fr-FR", "bonjour"), ("fr-FR", "l'"),
            ("es-ES", "mundo"), ("de-DE", "Hallo"), ("pt-BR", "olá"),
        ] {
            println!("{language} {word}: {:?}", dictionary.lookup(language, word).unwrap());
        }
    }
}
