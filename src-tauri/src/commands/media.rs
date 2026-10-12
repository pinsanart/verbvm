use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use super::error::CommandError;
use crate::orchestrators::FlashcardEditorOrchestrator;

use crate::dependencies::media_store::MediaKind;
use crate::modules::flashcards::MediaAttachment;

/// A media file attached to a field. `path` is where the file is on disk, for the view to
/// preview through the asset protocol.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    pub sha256: String,
    /// `IMAGE`, `AUDIO` or `VIDEO`.
    pub kind: String,
    pub mime_type: String,
    pub path: Option<String>,
    /// The name of the stored file, to read its bytes with `read_media`.
    pub file_name: String,
}

/// Builds the DTO of an attachment. `path` is the absolute path of its stored file, if known.
pub fn media_dto(media: &MediaAttachment, path: Option<PathBuf>) -> MediaDto {
    MediaDto {
        sha256: media.sha256().to_owned(),
        kind: match media.kind() {
            MediaKind::Image => "IMAGE",
            MediaKind::Audio => "AUDIO",
            MediaKind::Video => "VIDEO",
        }
        .to_owned(),
        mime_type: media.mime_type().to_owned(),
        path: path.map(|p| p.to_string_lossy().into_owned()),
        file_name: media.file_name().to_owned(),
    }
}

/// The bytes of a stored media file. Audio is played from these instead of through the asset
/// protocol, which the Linux webview cannot stream. Only files in the media folder can be read.
#[tauri::command]
pub async fn read_media(
    api: State<'_, Arc<FlashcardEditorOrchestrator>>,
    file_name: String,
) -> Result<tauri::ipc::Response, CommandError> {
    let path = api
        .media_path(&file_name)
        .ok_or_else(|| CommandError::invalid("Not a media file"))?;
    let bytes = std::fs::read(path).map_err(|_| CommandError::not_found("The media file was not found"))?;
    Ok(tauri::ipc::Response::new(bytes))
}
