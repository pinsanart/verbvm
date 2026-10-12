//! Local storage of flashcard media (images and audio).
//!
//! Media files live in S3 and are read offline from a local folder. This is the local side: it
//! copies a picked file into the folder under the name `<sha256>.<ext>`, so the same file is
//! stored once, and tells where a stored file is. Uploading and downloading are part of a future
//! sync, which can compare the stored names with the files on disk.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Largest file that is imported, in bytes.
pub const MAX_MEDIA_BYTES: u64 = 50 * 1024 * 1024;

/// Formats that are accepted, by detected MIME type. SVG is left out on purpose: it can carry
/// scripts.
const ALLOWED_MIME_TYPES: [&str; 10] = [
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "audio/mpeg",
    "audio/ogg",
    "audio/x-wav",
    "audio/x-flac",
    "audio/m4a",
    "audio/aac",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Audio,
}

impl MediaKind {
    /// The value stored in the database.
    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::Image => "IMAGE",
            MediaKind::Audio => "AUDIO",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "IMAGE" => Some(MediaKind::Image),
            "AUDIO" => Some(MediaKind::Audio),
            _ => None,
        }
    }
}

/// A file that was copied into the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMedia {
    pub sha256: String,
    /// `<sha256>.<ext>`, relative to the store folder.
    pub file_name: String,
    pub mime_type: String,
    pub kind: MediaKind,
    pub size_bytes: u64,
}

#[derive(Debug, Error)]
pub enum MediaError {
    #[error("the file does not exist or is not a regular file")]
    NotAFile,

    #[error("the file is larger than {} MB", MAX_MEDIA_BYTES / (1024 * 1024))]
    TooLarge,

    #[error("unsupported file type: only JPEG, PNG, GIF, WebP, MP3, OGG, WAV, FLAC, M4A and AAC are accepted")]
    UnsupportedType,

    #[error("could not read or store the file: {0}")]
    Io(#[from] std::io::Error),
}

/// Where media files go and where to find them.
pub trait MediaStorage: Send + Sync {
    /// Validates the file by its content (not its extension) and copies it into the store.
    /// Importing the same content again returns the same entry.
    fn import(&self, source: &Path) -> Result<StoredMedia, MediaError>;

    /// The absolute path of a stored file, or `None` when the name is not a plain file name.
    fn path_of(&self, file_name: &str) -> Option<PathBuf>;
}

/// [`MediaStorage`] in a folder on disk.
pub struct LocalMediaStore {
    dir: PathBuf,
    max_bytes: u64,
}

impl LocalMediaStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), max_bytes: MAX_MEDIA_BYTES }
    }

    #[cfg(test)]
    fn with_limit(dir: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self { dir: dir.into(), max_bytes }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl MediaStorage for LocalMediaStore {
    fn import(&self, source: &Path) -> Result<StoredMedia, MediaError> {
        let metadata = fs::metadata(source).map_err(|_| MediaError::NotAFile)?;
        if !metadata.is_file() {
            return Err(MediaError::NotAFile);
        }
        if metadata.len() > self.max_bytes {
            return Err(MediaError::TooLarge);
        }

        let bytes = fs::read(source)?;
        let kind = infer::get(&bytes).ok_or(MediaError::UnsupportedType)?;
        let mime_type = kind.mime_type();
        if !ALLOWED_MIME_TYPES.contains(&mime_type) {
            return Err(MediaError::UnsupportedType);
        }
        let media_kind = if mime_type.starts_with("image/") { MediaKind::Image } else { MediaKind::Audio };

        let sha256 = hex(&Sha256::digest(&bytes));
        let file_name = format!("{sha256}.{}", kind.extension());
        let target = self.dir.join(&file_name);
        if !target.exists() {
            fs::create_dir_all(&self.dir)?;
            // Write to a temporary name and rename, so a half-written file is never seen.
            let temporary = self.dir.join(format!("{file_name}.part"));
            let mut file = fs::File::create(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &target)?;
        }

        Ok(StoredMedia {
            sha256,
            file_name,
            mime_type: mime_type.to_owned(),
            kind: media_kind,
            size_bytes: bytes.len() as u64,
        })
    }

    fn path_of(&self, file_name: &str) -> Option<PathBuf> {
        let plain = !file_name.is_empty()
            && !file_name.contains(['/', '\\'])
            && file_name != "."
            && file_name != "..";
        plain.then(|| self.dir.join(file_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "verbvm-media-{name}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, bytes).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
    const MP3: &[u8] = b"ID3\x03\x00\x00\x00\x00\x00\x00audio-bytes";

    #[test]
    fn imports_by_content_under_the_hash_name() {
        let tmp = TempDir::new("import");
        let store = LocalMediaStore::new(tmp.0.join("media"));
        // The extension on the source is wrong on purpose: the content decides.
        let stored = store.import(&tmp.write("picture.txt", PNG)).unwrap();
        assert_eq!(stored.kind, MediaKind::Image);
        assert_eq!(stored.mime_type, "image/png");
        assert_eq!(stored.file_name, format!("{}.png", stored.sha256));
        assert_eq!(stored.sha256.len(), 64);
        assert_eq!(stored.size_bytes, PNG.len() as u64);
        assert_eq!(fs::read(store.path_of(&stored.file_name).unwrap()).unwrap(), PNG);

        let audio = store.import(&tmp.write("song", MP3)).unwrap();
        assert_eq!((audio.kind, audio.mime_type.as_str()), (MediaKind::Audio, "audio/mpeg"));
    }

    #[test]
    fn the_same_content_is_stored_once() {
        let tmp = TempDir::new("dedupe");
        let store = LocalMediaStore::new(tmp.0.join("media"));
        let first = store.import(&tmp.write("a.png", PNG)).unwrap();
        let second = store.import(&tmp.write("b.png", PNG)).unwrap();
        assert_eq!(first, second);
        assert_eq!(fs::read_dir(tmp.0.join("media")).unwrap().count(), 1, "no copy and no .part file is left");
    }

    #[test]
    fn rejects_unsupported_oversized_and_missing_files() {
        let tmp = TempDir::new("reject");
        let store = LocalMediaStore::with_limit(tmp.0.join("media"), 10);
        assert!(matches!(store.import(&tmp.write("note.txt", b"hello")), Err(MediaError::UnsupportedType)));
        assert!(matches!(store.import(&tmp.write("big.png", PNG)), Err(MediaError::TooLarge)));
        assert!(matches!(store.import(&tmp.0.join("missing.png")), Err(MediaError::NotAFile)));
        assert!(matches!(store.import(&tmp.0), Err(MediaError::NotAFile)));
    }

    #[test]
    fn svg_is_not_accepted() {
        let tmp = TempDir::new("svg");
        let store = LocalMediaStore::new(tmp.0.join("media"));
        let svg = tmp.write("x.svg", b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>1</script></svg>");
        assert!(matches!(store.import(&svg), Err(MediaError::UnsupportedType)));
    }

    #[test]
    fn path_of_only_accepts_plain_file_names() {
        let store = LocalMediaStore::new("/media");
        assert_eq!(store.path_of("abc.png"), Some(PathBuf::from("/media/abc.png")));
        for bad in ["", ".", "..", "../x.png", "a/b.png", "a\\b.png"] {
            assert_eq!(store.path_of(bad), None, "{bad}");
        }
    }

    #[test]
    fn kind_round_trips_through_its_stored_value() {
        assert_eq!(MediaKind::parse(MediaKind::Image.as_str()), Some(MediaKind::Image));
        assert_eq!(MediaKind::parse(MediaKind::Audio.as_str()), Some(MediaKind::Audio));
        assert_eq!(MediaKind::parse("VIDEO"), None);
    }
}
