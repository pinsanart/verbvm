import { RichInputField } from "./RichInputField";
import { PlainInputField } from "./PlainInputField";
import type { FieldDefinition } from "../types/FlashcardForm";

export interface FieldInputProps {
    definition: FieldDefinition;
    value: string;
    error?: string;
    onChange: (value: string) => void;
}

/** The input a field definition calls for: the rich editor for rich fields, plain text otherwise. */
export function FieldInput({ definition, value, error, onChange }: FieldInputProps) {
    if (definition.isRich) {
        return (
            <RichInputField
                label={definition.label}
                value={value}
                onChange={onChange}
                allowMedia={definition.mediaAllowed}
                required={definition.isRequired}
                error={error}
            />
        );
    }
    return (
        <PlainInputField
            label={definition.label}
            value={value}
            onChange={onChange}
            required={definition.isRequired}
            error={error}
        />
    );
}
