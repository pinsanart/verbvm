import type { Media } from "../types/Media";

/**
 * The audio that generating just produced: the one that was not attached before. Generating the
 * same text twice gives the same file, which is stored once, so then the last audio of the field
 * is the one to play.
 */
export function generatedAudio(before: Media[] | undefined, after: Media[] | undefined): Media | undefined {
    const audio = (after ?? []).filter((item) => item.kind === "AUDIO");
    const known = new Set((before ?? []).map((item) => item.sha256));
    return audio.find((item) => !known.has(item.sha256)) ?? audio[audio.length - 1];
}
