export type MediaKind = "IMAGE" | "AUDIO";

/** A file attached to a field, ready to preview. `url` is null when the file is missing. */
export interface Media {
    /** Hash of the content, which identifies the file within a field. */
    sha256: string;
    kind: MediaKind;
    mimeType: string;
    url: string | null;
}
