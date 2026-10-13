import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { AudioLines, LoaderCircle, Paperclip } from "lucide-react";
import { editorLabelClass, errorClass } from "./formStyles";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import type { Media } from "../types/Media";

export interface AudioInputFieldProps {
    label: string;
    required?: boolean;
    error?: string;
    media?: Media[];
    /** The field the audio is generated from has text. */
    canGenerate: boolean;
    onAttachMedia?: () => void;
    /** Generates the audio. Resolves when the attempt is over, whether or not it worked. */
    onGenerate?: () => Promise<unknown>;
    onRemoveMedia?: (sha256: string) => void;
}

/**
 * A field that holds only audio: no text box, just the upload button, the button that speaks the
 * source field and the audio already attached.
 */
export function AudioInputField({
    label,
    required,
    error,
    media,
    canGenerate,
    onAttachMedia,
    onGenerate,
    onRemoveMedia,
}: AudioInputFieldProps) {
    const { t } = useTranslation();
    const id = useId();
    const [generating, setGenerating] = useState(false);

    const generate = async () => {
        if (!onGenerate) return;
        setGenerating(true);
        try {
            await onGenerate();
        } finally {
            setGenerating(false);
        }
    };

    return (
        <div role="group" aria-labelledby={id}>
            <div className="mb-1 flex min-h-10 items-center justify-between md:min-h-9">
                <span id={id} className={`${editorLabelClass} mb-0`}>
                    {label}
                    {required && (
                        <span className="ml-1 text-subtle" aria-hidden>
                            *
                        </span>
                    )}
                </span>
                <div className="flex items-center gap-0.5">
                    <IconButton
                        label={generating ? t("media.generating") : t("media.generate")}
                        icon={generating ? LoaderCircle : AudioLines}
                        iconSize={16}
                        className={generating ? "[&>svg]:animate-spin" : undefined}
                        onClick={generate}
                        disabled={!canGenerate || generating}
                    />
                    <IconButton label={t("media.attach")} icon={Paperclip} iconSize={16} onClick={onAttachMedia} />
                </div>
            </div>
            {!canGenerate && (!media || media.length === 0) && (
                <p className="text-meta text-muted">{t("media.generateHint")}</p>
            )}
            {error && <p className={errorClass}>{error}</p>}
            <div className="mt-2">
                <MediaPreview media={media} onRemove={onRemoveMedia} />
            </div>
        </div>
    );
}
