import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Media, MediaKind } from "../types/Media";

/** What the backend sends for a media file: it comes with its path on disk. */
export interface MediaDto {
    sha256: string;
    kind: MediaKind;
    mimeType: string;
    path: string | null;
    fileName: string;
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
    fileName: dto.fileName,
});

/** The bytes of a stored media file, read by the backend. */
export const readMedia = (fileName: string) => invoke<ArrayBuffer>("read_media", { fileName });

/**
 * Plays an audio file once, right away, without any player on the screen. Used when audio has
 * just been generated. The promise resolves when playback starts and rejects when it cannot.
 */
export const playAudio = async (media: Media) => {
    const bytes = await readMedia(media.fileName);
    const url = URL.createObjectURL(new Blob([bytes], { type: media.mimeType }));
    const audio = new Audio(url);
    const release = () => URL.revokeObjectURL(url);
    audio.addEventListener("ended", release, { once: true });
    audio.addEventListener("error", release, { once: true });
    try {
        await audio.play();
    } catch (error) {
        release();
        throw error;
    }
};
