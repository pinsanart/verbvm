import { RichInputField } from "./RichInputField";
import { PlainInputField } from "./PlainInputField";
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
    onRemoveMedia?: (sha256: string) => void;
}

/**
 * The input a field definition calls for: the rich editor for rich fields and plain text
 * otherwise. A field that allows media gets the attach button and a preview of its files.
 */
export function FieldInput({ definition, value, error, media, onChange, onAttachMedia, onRemoveMedia }: FieldInputProps) {
    const label = useFieldLabel()(definition.key, definition.label);
    if (definition.isRich) {
        return (
            <RichInputField
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
