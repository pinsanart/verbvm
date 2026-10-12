import { AnimatePresence, motion } from "motion/react";
import { TriangleAlert, X } from "lucide-react";
import { MediaPlayer } from "./MediaPlayer";
import { TRANSITION_FAST } from "../lib/motion";
import type { Media, MediaKind } from "../types/Media";

export interface MediaPreviewProps {
    media?: Media[];
    /** Called with the hash of the file to detach. Without it there is no remove button. */
    onRemove?: (sha256: string) => void;
}

// A section appears only when the field has files of that kind.
const GROUPS: { kind: MediaKind; title: string }[] = [
    { kind: "IMAGE", title: "Images" },
    { kind: "AUDIO", title: "Audio" },
    { kind: "VIDEO", title: "Videos" },
];

function RemoveButton({ onRemove }: { onRemove: () => void }) {
    return (
        <button
            type="button"
            aria-label="Remove media"
            onClick={onRemove}
            className="flex size-6 shrink-0 items-center justify-center rounded-full bg-black/70 text-ink transition-colors hover:bg-black hover:text-white"
        >
            <X size={12} aria-hidden />
        </button>
    );
}

/**
 * The files attached to a field, grouped by kind: images as thumbnails and audio as players, each
 * under its own heading. A group is shown only when it has files.
 */
export function MediaPreview({ media, onRemove }: MediaPreviewProps) {
    if (!media || media.length === 0) return null;

    return (
        <div className="flex flex-col gap-4 rounded-xl border border-line bg-white/5 p-3">
            {GROUPS.map(({ kind, title }) => {
                const items = media.filter((item) => item.kind === kind);
                if (items.length === 0) return null;
                return (
                    <section key={kind} aria-label={title} className="flex flex-col gap-2">
                        <h4 className="flex items-center gap-2 text-label font-medium uppercase tracking-wider text-muted">
                            {title}
                            <span className="tabular-nums text-subtle">{items.length}</span>
                        </h4>
                        <ul className={kind === "IMAGE" ? "flex flex-wrap gap-2" : "flex flex-col gap-3"}>
                            <AnimatePresence initial={false}>
                                {items.map((item) => (
                                    <motion.li
                                        key={item.sha256}
                                        layout
                                        initial={{ opacity: 0, scale: 0.95 }}
                                        animate={{ opacity: 1, scale: 1 }}
                                        exit={{ opacity: 0, scale: 0.95 }}
                                        transition={TRANSITION_FAST}
                                        className={kind === "IMAGE" ? "relative" : "flex items-center gap-2"}
                                    >
                                        {item.url === null ? (
                                            <p className="flex h-12 items-center gap-1.5 rounded-lg bg-white/5 px-3 text-meta text-muted">
                                                <TriangleAlert size={14} aria-hidden />
                                                File not found
                                            </p>
                                        ) : kind === "IMAGE" ? (
                                            <img src={item.url} alt="" className="size-20 rounded-lg object-cover" />
                                        ) : kind === "VIDEO" ? (
                                            <MediaPlayer media={item} className="max-h-56 min-w-0 flex-1 rounded-lg bg-black" />
                                        ) : (
                                            <MediaPlayer media={item} className="h-10 min-w-0 flex-1" />
                                        )}
                                        {onRemove &&
                                            (kind === "IMAGE" ? (
                                                <span className="absolute -right-1.5 -top-1.5">
                                                    <RemoveButton onRemove={() => onRemove(item.sha256)} />
                                                </span>
                                            ) : (
                                                <RemoveButton onRemove={() => onRemove(item.sha256)} />
                                            ))}
                                    </motion.li>
                                ))}
                            </AnimatePresence>
                        </ul>
                    </section>
                );
            })}
        </div>
    );
}
