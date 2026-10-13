//! Reads the notes of an Anki collection (`collection.anki2`), which is a SQLite file.
//!
//! Only reading: the collection is opened read-only and never changed. Anki keeps a collection
//! it has open locked, and then the file can only be read as it was when Anki last wrote it to
//! disk, without the changes still in its write-ahead log. That is reported, so the caller can
//! tell the user.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, ErrorCode, OpenFlags};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnkiError {
    #[error("that file is not an Anki collection")]
    NotACollection,

    #[error("could not read the Anki collection: {0}")]
    Db(#[from] rusqlite::Error),
}

/// A note, as the first card of it is filed: the deck it is in and its fields in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnkiNote {
    /// The parts of the deck's name, outermost first (Anki joins them with a unit separator).
    pub deck_path: Vec<String>,
    pub fields: Vec<String>,
}

pub struct AnkiCollection {
    connection: Connection,
    media_dir: PathBuf,
    /// The collection was locked by Anki, so the changes in its write-ahead log were not read.
    partial: bool,
}

/// Anki compares deck names without regard to case with its own collation, which the schema
/// refers to, so the connection has to provide one even though the notes are read by id.
fn register_collation(connection: &Connection) -> rusqlite::Result<()> {
    connection.create_collation("unicase", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))
}

/// A file path as the path of a `file:` URI.
fn uri_path(path: &Path) -> String {
    let mut encoded = String::new();
    for byte in path.to_string_lossy().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => encoded.push(byte as char),
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

fn is_locked(error: &rusqlite::Error) -> bool {
    matches!(
        error.sqlite_error_code(),
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    )
}

impl AnkiCollection {
    /// Opens a collection for reading. The media folder is `collection.media` next to the file.
    pub fn open(path: &Path) -> Result<Self, AnkiError> {
        let media_dir = path.parent().map(|dir| dir.join("collection.media")).unwrap_or_default();
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;

        let check = |connection: &Connection| -> rusqlite::Result<()> {
            connection.query_row("SELECT COUNT(*) FROM notes", [], |_| Ok(()))
        };
        let open = |query: &str| -> rusqlite::Result<Connection> {
            let connection = Connection::open_with_flags(format!("file:{}{query}", uri_path(path)), flags)?;
            register_collation(&connection)?;
            Ok(connection)
        };

        let connection = open("?mode=ro")?;
        // A collection Anki has open is locked: do not wait long to find out.
        connection.busy_timeout(std::time::Duration::from_millis(100))?;
        match check(&connection) {
            Ok(()) => Ok(Self { connection, media_dir, partial: false }),
            Err(error) if is_locked(&error) => {
                // Anki has it open. Read the file as it is on disk, ignoring the lock.
                let connection = open("?mode=ro&immutable=1")?;
                check(&connection).map_err(|_| AnkiError::NotACollection)?;
                Ok(Self { connection, media_dir, partial: true })
            }
            Err(_) => Err(AnkiError::NotACollection),
        }
    }

    /// Whether changes that Anki had not yet written to the file may be missing.
    pub fn is_partial(&self) -> bool {
        self.partial
    }

    /// Where a media file of the collection is, if the name is a plain file name.
    pub fn media_path(&self, name: &str) -> Option<PathBuf> {
        let plain = !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != "..";
        plain.then(|| self.media_dir.join(name))
    }

    /// Every note with the deck of its first card, in the order of the cards.
    pub fn notes(&self) -> Result<Vec<AnkiNote>, AnkiError> {
        let mut statement = self.connection.prepare(
            "SELECT d.name, n.flds
             FROM cards c
             JOIN notes n ON n.id = c.nid
             JOIN decks d ON d.id = c.did
             WHERE c.id = (SELECT MIN(id) FROM cards WHERE nid = n.id)
             ORDER BY c.id",
        )?;
        let notes = statement
            .query_map([], |row| {
                let deck: String = row.get(0)?;
                let fields: String = row.get(1)?;
                Ok(AnkiNote {
                    deck_path: deck.split('\u{1f}').map(str::to_owned).collect(),
                    fields: fields.split('\u{1f}').map(str::to_owned).collect(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(notes)
    }
}

/// Builds collections like Anki's, for the tests of this module and of the importer.
#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    /// A collection like Anki's: the notes, their cards and the decks, with the same collation.
    pub fn write_collection(path: &Path, notes: &[(&str, &[&str])]) {
        let connection = Connection::open(path).unwrap();
        register_collation(&connection).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE notes (id INTEGER PRIMARY KEY, guid TEXT, mid INTEGER, flds TEXT);
                 CREATE TABLE cards (id INTEGER PRIMARY KEY, nid INTEGER, did INTEGER);
                 CREATE TABLE decks (id INTEGER PRIMARY KEY, name TEXT NOT NULL COLLATE unicase);
                 CREATE UNIQUE INDEX idx_decks_name ON decks (name);",
            )
            .unwrap();
        let mut decks: Vec<String> = Vec::new();
        for (index, (deck, fields)) in notes.iter().enumerate() {
            let deck_id = match decks.iter().position(|d| d == deck) {
                Some(position) => position as i64 + 1,
                None => {
                    decks.push((*deck).to_owned());
                    connection.execute("INSERT INTO decks (id, name) VALUES (?1, ?2)", (decks.len() as i64, deck)).unwrap();
                    decks.len() as i64
                }
            };
            let id = index as i64 + 1;
            connection
                .execute("INSERT INTO notes (id, guid, mid, flds) VALUES (?1, 'g', 1, ?2)", (id, fields.join("\u{1f}")))
                .unwrap();
            connection.execute("INSERT INTO cards (id, nid, did) VALUES (?1, ?1, ?2)", (id, deck_id)).unwrap();
        }
    }

}

#[cfg(test)]
mod tests {
    use super::fixtures::write_collection;
    use super::*;
    use std::fs;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "verbvm-anki-{name}-{}-{:?} with space",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn reads_the_notes_with_their_deck_path_and_fields() {
        let tmp = TempDir::new("read");
        let path = tmp.0.join("collection.anki2");
        write_collection(
            &path,
            &[
                ("Línguas\u{1f}Inglês\u{1f}Pronúncia", &["It's my elder brother.", "<strong>it's</strong>: /ɪts/<br>"]),
                ("Línguas\u{1f}Inglês\u{1f}Vocabulário", &["It's my <strong>elder</strong> brother.", "<strong>elder:</strong> older"]),
            ],
        );
        let collection = AnkiCollection::open(&path).unwrap();
        assert!(!collection.is_partial());
        let notes = collection.notes().unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].deck_path, vec!["Línguas", "Inglês", "Pronúncia"]);
        assert_eq!(notes[0].fields, vec!["It's my elder brother.", "<strong>it's</strong>: /ɪts/<br>"]);
        assert_eq!(notes[1].deck_path[2], "Vocabulário");
    }

    #[test]
    fn a_file_that_is_not_a_collection_is_refused() {
        let tmp = TempDir::new("refuse");
        let path = tmp.0.join("other.db");
        Connection::open(&path).unwrap().execute_batch("CREATE TABLE other (id INTEGER)").unwrap();
        assert!(matches!(AnkiCollection::open(&path), Err(AnkiError::NotACollection)));
        assert!(matches!(AnkiCollection::open(&tmp.0.join("missing.anki2")), Err(AnkiError::NotACollection | AnkiError::Db(_))));
    }

    #[test]
    fn a_collection_locked_by_anki_is_read_from_the_file_and_reported() {
        let tmp = TempDir::new("locked");
        let path = tmp.0.join("collection.anki2");
        write_collection(&path, &[("Deck", &["a", "b"])]);
        // Anki keeps the collection open with an exclusive lock.
        let holder = Connection::open(&path).unwrap();
        holder.execute_batch("PRAGMA locking_mode = EXCLUSIVE; BEGIN EXCLUSIVE;").unwrap();

        let collection = AnkiCollection::open(&path).unwrap();
        assert!(collection.is_partial(), "the lock was bypassed, so recent changes may be missing");
        assert_eq!(collection.notes().unwrap().len(), 1);
    }

    #[test]
    fn media_files_are_looked_up_in_the_folder_next_to_the_collection() {
        let tmp = TempDir::new("media");
        let path = tmp.0.join("collection.anki2");
        write_collection(&path, &[("Deck", &["a", "b"])]);
        let collection = AnkiCollection::open(&path).unwrap();
        assert_eq!(collection.media_path("pic.jpg"), Some(tmp.0.join("collection.media").join("pic.jpg")));
        for bad in ["", ".", "..", "../x.jpg", "a/b.jpg"] {
            assert_eq!(collection.media_path(bad), None, "{bad}");
        }
    }
}
