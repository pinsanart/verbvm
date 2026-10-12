use std::path::PathBuf;

use serde::Serialize;

use crate::dependencies::media_store::MediaKind;
use crate::modules::flashcards::MediaAttachment;

/// A media file attached to a field. `path` is where the file is on disk, for the view to
/// preview through the asset protocol.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    pub sha256: String,
    /// `IMAGE` or `AUDIO`.
    pub kind: String,
    pub mime_type: String,
    pub path: Option<String>,
}

/// Builds the DTO of an attachment. `path` is the absolute path of its stored file, if known.
pub fn media_dto(media: &MediaAttachment, path: Option<PathBuf>) -> MediaDto {
    MediaDto {
        sha256: media.sha256().to_owned(),
        kind: match media.kind() {
            MediaKind::Image => "IMAGE",
            MediaKind::Audio => "AUDIO",
        }
        .to_owned(),
        mime_type: media.mime_type().to_owned(),
        path: path.map(|p| p.to_string_lossy().into_owned()),
    }
}
