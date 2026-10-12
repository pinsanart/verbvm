import { AnimatePresence, motion } from "motion/react";
import { TriangleAlert, X } from "lucide-react";
import { Media } from "../types/Media";

export interface MediaPreviewProps {
    media?: Media[];
    /** Called with the hash of the file to detach. Without it there is no remove button. */
    onRemove?: (sha256: string) => void;
}

/** A preview of the images and audio attached to a field. */
export function MediaPreview({ media, onRemove }: MediaPreviewProps) {
    if (!media || media.length === 0) return null;

    return (
        <div className="flex flex-wrap items-start gap-2 rounded-xl border border-white/10 bg-white/5 p-2">
            <AnimatePresence initial={false}>
            {media.map((item) => (
                <motion.div
                    key={item.sha256}
                    layout
                    initial={{ opacity: 0, scale: 0.8 }}
                    animate={{ opacity: 1, scale: 1 }}
                    exit={{ opacity: 0, scale: 0.8 }}
                    transition={{ duration: 0.15 }}
                    className="relative"
                >
                    {item.url === null ? (
                        <div className="flex h-16 items-center gap-1.5 rounded-lg bg-white/5 px-3 text-xs text-muted">
                            <TriangleAlert size={14} />
                            File not found
                        </div>
                    ) : item.kind === "IMAGE" ? (
                        <img src={item.url} alt="" className="h-16 w-16 rounded-lg object-cover" />
                    ) : (
                        <audio controls src={item.url} className="h-9" />
                    )}
                    {onRemove && (
                        <button
                            type="button"
                            aria-label="Remove media"
                            onClick={() => onRemove(item.sha256)}
                            className="absolute -right-1.5 -top-1.5 rounded-full bg-black/70 p-0.5 text-white/80 hover:bg-black hover:text-white"
                        >
                            <X size={12} />
                        </button>
                    )}
                </motion.div>
            ))}
            </AnimatePresence>
        </div>
    );
}
