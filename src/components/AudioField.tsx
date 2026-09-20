import { useRef } from "react";
import { Paperclip, Wand2 } from "lucide-react";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import { Media } from "../types/Media";

export interface AudioFieldProps {
    label: string;
    media?: Media[];
}

export function AudioField({ label, media }: AudioFieldProps) {
    const fileInputRef = useRef<HTMLInputElement>(null);

    function handleFilesSelected(_e: React.ChangeEvent<HTMLInputElement>) {
        // Estrutura visual apenas: anexar áudio de verdade fica para uma etapa futura.
    }

    function handleGenerate() {
        // Estrutura visual apenas: gerar áudio via TTS a partir da phrase fica para uma etapa futura.
    }

    return (
        <div>
            <div className="mb-1.5 flex items-center justify-between">
                <label className="text-xs font-medium uppercase tracking-wide text-white/40">{label}</label>
                <div className="flex items-center gap-0.5">
                    <IconButton label="Attach audio" icon={Paperclip} iconSize={14} onClick={() => fileInputRef.current?.click()} />
                    <input ref={fileInputRef} type="file" accept="audio/*" multiple hidden onChange={handleFilesSelected} />
                    <IconButton label="Generate audio" icon={Wand2} iconSize={14} onClick={handleGenerate} />
                </div>
            </div>
            <MediaPreview media={media} />
        </div>
    );
}
