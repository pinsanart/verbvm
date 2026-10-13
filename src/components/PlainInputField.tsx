import { useId } from "react";
import { useTranslation } from "react-i18next";
import { Paperclip } from "lucide-react";
import { editorControlClass, editorLabelClass, errorClass } from "./formStyles";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import type { Media } from "../types/Media";

export interface PlainInputFieldProps {
    label: string;
    value: string;
    onChange: (value: string) => void;
    required?: boolean;
    error?: string;
    /** Shows the attach button and the files of the field. */
    allowMedia?: boolean;
    media?: Media[];
    onAttachMedia?: () => void;
    onRemoveMedia?: (sha256: string) => void;
}

export function PlainInputField({
    label,
    value,
    onChange,
    required,
    error,
    allowMedia,
    media,
    onAttachMedia,
    onRemoveMedia,
}: PlainInputFieldProps) {
    const { t } = useTranslation();
    const id = useId();
    const errorId = `${id}-error`;

    return (
        <div>
            <div className={`mb-1 flex items-center justify-between ${allowMedia ? "min-h-10 md:min-h-9" : ""}`}>
                <label htmlFor={id} className={`${editorLabelClass} mb-0`}>
                    {label}
                    {required && (
                        <span className="ml-1 text-subtle" aria-hidden>
                            *
                        </span>
                    )}
                </label>
                {allowMedia && <IconButton label={t("media.attach")} icon={Paperclip} iconSize={16} onClick={onAttachMedia} />}
            </div>
            <textarea
                id={id}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                aria-required={required || undefined}
                aria-invalid={error ? true : undefined}
                aria-describedby={error ? errorId : undefined}
                className={`${editorControlClass} min-h-20 resize-y`}
            />
            {error && (
                <p id={errorId} className={errorClass}>
                    {error}
                </p>
            )}
            {allowMedia && (
                <div className="mt-2">
                    <MediaPreview media={media} onRemove={onRemoveMedia} />
                </div>
            )}
        </div>
    );
}
