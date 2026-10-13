import { RichInputField } from "./RichInputField";
import { PlainInputField } from "./PlainInputField";
import { AudioInputField } from "./AudioInputField";
import { useTranslation } from "react-i18next";
import { useFieldLabel } from "../i18n/useFieldLabel";
import type { FieldDefinition } from "../types/FlashcardForm";
import type { Media } from "../types/Media";

export interface FieldInputProps {
    definition: FieldDefinition;
    value: string;
    error?: string;
    /** The files attached to the field. Only used when the field allows media. */
    media?: Media[];
    onChange: (value: string) => void;
    onAttachMedia?: () => void;
    /** Generates the audio of an audio field or the pronunciation of an IPA field. Resolves when the attempt is over. */
    onGenerate?: () => Promise<unknown>;
    /** The field an audio or IPA field is generated from has text. */
    canGenerate?: boolean;
    onRemoveMedia?: (sha256: string) => void;
}

/**
 * The input a field definition calls for: the rich editor for rich fields, audio only for audio
 * fields and plain text otherwise. A field that allows media gets the attach button and a preview of its files.
 */
export function FieldInput({
    definition,
    value,
    error,
    media,
    onChange,
    onAttachMedia,
    onGenerate,
    canGenerate,
    onRemoveMedia,
}: FieldInputProps) {
    const { t } = useTranslation();
    const label = useFieldLabel()(definition.key, definition.label);
    if (definition.kind === "AUDIO") {
        return (
            <AudioInputField
                label={label}
                required={definition.isRequired}
                error={error}
                media={media}
                canGenerate={canGenerate ?? false}
                onAttachMedia={onAttachMedia}
                onGenerate={onGenerate}
                onRemoveMedia={onRemoveMedia}
            />
        );
    }
    if (definition.isRich) {
        return (
            <RichInputField
                ipa={definition.kind === "IPA"}
                onGenerate={definition.kind === "IPA" ? onGenerate : undefined}
                generateLabel={definition.kind === "IPA" ? t("media.generatePronunciation") : undefined}
                canGenerate={canGenerate ?? false}
                label={label}
                value={value}
                onChange={onChange}
                allowMedia={definition.mediaAllowed}
                media={media}
                onAttachMedia={onAttachMedia}
                onRemoveMedia={onRemoveMedia}
                required={definition.isRequired}
                error={error}
            />
        );
    }
    return (
        <PlainInputField
            label={label}
            value={value}
            onChange={onChange}
            required={definition.isRequired}
            error={error}
            allowMedia={definition.mediaAllowed}
            media={media}
            onAttachMedia={onAttachMedia}
            onRemoveMedia={onRemoveMedia}
        />
    );
}
