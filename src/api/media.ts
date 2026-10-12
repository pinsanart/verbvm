import { convertFileSrc } from "@tauri-apps/api/core";
import type { Media, MediaKind } from "../types/Media";

/** What the backend sends for a media file: it comes with its path on disk. */
export interface MediaDto {
    sha256: string;
    kind: MediaKind;
    mimeType: string;
    path: string | null;
}

/**
 * The webview cannot read a path directly, so it is turned into an asset protocol URL. The
 * backend only allows that protocol for its own media folder.
 */
export const toMedia = (dto: MediaDto): Media => ({
    sha256: dto.sha256,
    kind: dto.kind,
    mimeType: dto.mimeType,
    url: dto.path ? convertFileSrc(dto.path) : null,
});
