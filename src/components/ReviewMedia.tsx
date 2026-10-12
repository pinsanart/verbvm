import { TriangleAlert } from "lucide-react";
import type { Media } from "../types/Media";

/**
 * The images and audio of a field, at the size they are meant to be seen and heard in a review.
 * Audio is marked so the review screen can replay it with the R key.
 */
export function ReviewMedia({ media }: { media: Media[] }) {
    if (media.length === 0) return null;
    const hasAudio = media.some((item) => item.kind === "AUDIO" && item.url !== null);

    return (
        <div className="flex w-full flex-col items-center gap-3">
            {media.map((item) => {
                if (item.url === null) {
                    return (
                        <p key={item.sha256} className="flex items-center gap-1.5 text-meta text-muted">
                            <TriangleAlert size={14} aria-hidden />
                            File not found
                        </p>
                    );
                }
                return item.kind === "IMAGE" ? (
                    <img key={item.sha256} src={item.url} alt="" className="max-h-64 max-w-full rounded-xl object-contain" />
                ) : (
                    <audio key={item.sha256} controls data-review-audio src={item.url} className="w-full max-w-md" />
                );
            })}
            {hasAudio && (
                <p className="text-meta text-muted">
                    Press <kbd className="rounded border border-line-strong px-1.5 py-0.5 font-sans text-label text-ink">R</kbd> to play
                    it again
                </p>
            )}
        </div>
    );
}
