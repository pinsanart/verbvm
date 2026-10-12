import { TriangleAlert } from "lucide-react";
import type { Media } from "../types/Media";

/** The images and audio of a field, at the size they are meant to be seen and heard in a review. */
export function ReviewMedia({ media }: { media: Media[] }) {
    if (media.length === 0) return null;
    return (
        <div className="flex flex-col items-center gap-3">
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
                    <audio key={item.sha256} controls src={item.url} className="w-full max-w-md" />
                );
            })}
        </div>
    );
}
