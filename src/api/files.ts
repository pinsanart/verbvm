import { open } from "@tauri-apps/plugin-dialog";

const MEDIA_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "webp", "mp3", "ogg", "wav", "flac", "m4a", "aac", "mp4", "webm", "mov"];

/** Opens the system file picker for images, audio and video. Resolves to the chosen paths, or none. */
export async function pickMediaFiles(): Promise<string[]> {
    const picked = await open({
        multiple: true,
        directory: false,
        filters: [{ name: "Images, audio and video", extensions: MEDIA_EXTENSIONS }],
    });
    if (picked === null) return [];
    return Array.isArray(picked) ? picked : [picked];
}
