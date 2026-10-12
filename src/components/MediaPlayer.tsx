import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { TriangleAlert } from "lucide-react";
import { readMedia } from "../api/media";
import type { Media } from "../types/Media";

export interface MediaPlayerProps {
    /** An audio or video file. */
    media: Media;
    className?: string;
    /** Marks the audio so the review screen can replay it with the R key. */
    reviewAudio?: boolean;
    /** Starts playing as soon as the file is loaded. */
    autoPlay?: boolean;
}

/**
 * Plays an audio or video file of a card. The webview on Linux cannot stream media through the
 * asset protocol, so the backend reads the file and the player gets an in-memory URL with the
 * right type.
 */
export function MediaPlayer({ media, className, reviewAudio, autoPlay }: MediaPlayerProps) {
    const { t } = useTranslation();
    const [src, setSrc] = useState<string | null>(null);
    const [failed, setFailed] = useState(false);
    /** The webview refused to start the audio by itself, so the learner is told to press play. */
    const [blocked, setBlocked] = useState(false);
    const element = useRef<HTMLAudioElement & HTMLVideoElement>(null);

    useEffect(() => {
        let cancelled = false;
        let url: string | null = null;
        setSrc(null);
        setFailed(false);
        setBlocked(false);
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

    // Play as soon as the file is loaded. The audio is set from an in-memory URL after the element
    // exists, so playback is started explicitly instead of relying on the `autoplay` attribute.
    useEffect(() => {
        if (autoPlay && src) element.current?.play().catch(() => setBlocked(true));
    }, [autoPlay, src]);

    if (failed) {
        return (
            <p className="flex items-center gap-1.5 text-meta text-muted">
                <TriangleAlert size={14} aria-hidden />
                {media.kind === "VIDEO" ? t("media.videoFailed") : t("media.audioFailed")}
            </p>
        );
    }
    // While the file is being read the player has nothing to show, so a placeholder of the same
    // size holds the place and says so.
    if (!src) {
        return (
            <div
                role="status"
                aria-label={t("media.loadingAudio")}
                className={`${className ?? ""} min-h-10 animate-pulse rounded-lg bg-white/10`}
            />
        );
    }
    const common = {
        controls: true,
        onPlaying: () => setBlocked(false),
        ref: element,
        preload: "metadata" as const,
        src: src ?? undefined,
        onError: () => {
            if (src) setFailed(true);
        },
    };
    return media.kind === "VIDEO" ? (
        <video {...common} playsInline className={className} />
    ) : (
        <>
            <audio {...common} data-review-audio={reviewAudio ? "" : undefined} className={className} />
            {blocked && <p className="text-meta text-muted">{t("media.autoplayBlocked")}</p>}
        </>
    );
}
