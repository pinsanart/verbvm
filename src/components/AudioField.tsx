import { Paperclip, Wand2 } from "lucide-react";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import { Media } from "../types/Media";

export interface AudioFieldProps {
    label: string;
    media?: Media[];
    onAttach: () => void;
    onRemove: (sha256: string) => void;
}

export function AudioField({ label, media, onAttach, onRemove }: AudioFieldProps) {
    function handleGenerate() {
        // Visual structure only: generating audio with TTS from the phrase comes in a later step.
    }

    return (
        <div>
            <div className="mb-1.5 flex items-center justify-between">
                <label className="text-xs font-medium uppercase tracking-wide text-white/40">{label}</label>
                <div className="flex items-center gap-0.5">
                    <IconButton label="Attach media" icon={Paperclip} iconSize={14} onClick={onAttach} />
                    <IconButton label="Generate audio" icon={Wand2} iconSize={14} onClick={handleGenerate} />
                </div>
            </div>
            <MediaPreview media={media} onRemove={onRemove} />
        </div>
    );
}
