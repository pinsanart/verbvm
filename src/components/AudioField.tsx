import { Paperclip } from "lucide-react";
import { Button } from "./Button";
import { MediaPreview } from "./MediaPreview";
import { Media } from "../types/Media";

export interface AudioFieldProps {
    media?: Media[];
    onAttach: () => void;
    onRemove: (sha256: string) => void;
}

/** Attaches images and audio to a plain field and previews what is attached. */
export function AudioField({ media, onAttach, onRemove }: AudioFieldProps) {
    return (
        <div className="flex flex-wrap items-center gap-3">
            <Button variant="outline" onClick={onAttach} className="min-h-10">
                <Paperclip size={16} aria-hidden />
                Add image or audio
            </Button>
            <MediaPreview media={media} onRemove={onRemove} />
        </div>
    );
}
