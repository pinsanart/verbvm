import { useEffect, useState } from "react";
import { TriangleAlert } from "lucide-react";
import { readMedia } from "../api/media";
import type { Media } from "../types/Media";

/**
 * Plays an audio file of a card. The webview on Linux cannot stream audio through the asset
 * protocol, so the backend reads the file and the player gets an in-memory URL with the right type.
 */
export function AudioPlayer({ media, className, reviewAudio }: { media: Media; className?: string; reviewAudio?: boolean }) {
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
                The audio could not be loaded
            </p>
        );
    }
    return (
        <audio
            controls
            preload="auto"
            src={src ?? undefined}
            data-review-audio={reviewAudio ? "" : undefined}
            onError={() => src && setFailed(true)}
            className={className}
        />
    );
}
