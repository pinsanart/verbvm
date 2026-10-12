import { useRef } from "react";
import { Volume2, TriangleAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "./Button";
import { MediaPlayer } from "./MediaPlayer";
import { replayAudio } from "../lib/audio";
import type { Media } from "../types/Media";

/**
 * The images and audio of a field, at the size they are meant to be seen and heard in a review.
 * Audio is marked so the review screen can replay it with the R key, and a visible Replay button
 * does the same for the files of this field. With `autoPlay` the first audio plays by itself, and
 * only the first, so several files do not overlap.
 */
export function ReviewMedia({ media, autoPlay }: { media: Media[]; autoPlay?: boolean }) {
    const { t } = useTranslation();
    const root = useRef<HTMLDivElement>(null);
    if (media.length === 0) return null;
    const hasAudio = media.some((item) => item.kind === "AUDIO" && item.url !== null);
    const firstAudio = media.find((item) => item.kind === "AUDIO" && item.url !== null);

    return (
        <div ref={root} className="flex w-full flex-col items-center gap-3">
            {media.map((item) => {
                if (item.url === null) {
                    return (
                        <p key={item.sha256} className="flex items-center gap-1.5 text-meta text-muted">
                            <TriangleAlert size={14} aria-hidden />
                            {t("common.fileNotFound")}
                        </p>
                    );
                }
                return item.kind === "IMAGE" ? (
                    <img key={item.sha256} src={item.url} alt="" className="max-h-64 max-w-full rounded-xl object-contain" />
                ) : item.kind === "VIDEO" ? (
                    <MediaPlayer key={item.sha256} media={item} className="max-h-80 w-full max-w-2xl rounded-xl bg-black" />
                ) : (
                    <MediaPlayer
                        key={item.sha256}
                        media={item}
                        reviewAudio
                        autoPlay={autoPlay && item === firstAudio}
                        className="w-full max-w-md"
                    />
                );
            })}
            {hasAudio && (
                <Button variant="outline" onClick={() => replayAudio(root.current ?? document)} aria-keyshortcuts="R">
                    <Volume2 size={16} aria-hidden />
                    {t("media.replay")}
                    <kbd className="hidden rounded border border-line-strong px-1.5 py-0.5 font-sans text-label font-normal text-ink md:inline">
                        R
                    </kbd>
                </Button>
            )}
        </div>
    );
}
