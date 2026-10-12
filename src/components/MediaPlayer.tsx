import { useEffect, useState } from "react";
import { TriangleAlert } from "lucide-react";
import { readMedia } from "../api/media";
import type { Media } from "../types/Media";

export interface MediaPlayerProps {
    /** An audio or video file. */
    media: Media;
    className?: string;
    /** Marks the audio so the review screen can replay it with the R key. */
    reviewAudio?: boolean;
}

/**
 * Plays an audio or video file of a card. The webview on Linux cannot stream media through the
 * asset protocol, so the backend reads the file and the player gets an in-memory URL with the
 * right type.
 */
export function MediaPlayer({ media, className, reviewAudio }: MediaPlayerProps) {
    const [src, setSrc] = useState<string | null>(null);
    const [failed, setFailed] = useState(false);

    useEffect(() => {
        let cancelled = false;
        let url: string | null = null;
        setSrc(null);
        setFailed(false);
        readMedia(media.fileName)
            .then((bytes) => {
                if (cancelled) return;
                url = URL.createObjectURL(new Blob([bytes], { type: media.mimeType }));
                setSrc(url);
            })
            .catch(() => !cancelled && setFailed(true));
        return () => {
            cancelled = true;
            if (url) URL.revokeObjectURL(url);
        };
    }, [media.fileName, media.mimeType]);

    if (failed) {
        return (
            <p className="flex items-center gap-1.5 text-meta text-muted">
                <TriangleAlert size={14} aria-hidden />
                The {media.kind === "VIDEO" ? "video" : "audio"} could not be loaded
            </p>
        );
    }
    const common = {
        controls: true,
        preload: "metadata" as const,
        src: src ?? undefined,
        onError: () => {
            if (src) setFailed(true);
        },
    };
    return media.kind === "VIDEO" ? (
        <video {...common} playsInline className={className} />
    ) : (
        <audio {...common} data-review-audio={reviewAudio ? "" : undefined} className={className} />
    );
}
