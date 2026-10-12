import { useId } from "react";
import { editorControlClass, editorLabelClass, errorClass } from "./formStyles";

export interface PlainInputFieldProps {
    label: string;
    value: string;
    onChange: (value: string) => void;
    required?: boolean;
    error?: string;
}

export function PlainInputField({ label, value, onChange, required, error }: PlainInputFieldProps) {
    const id = useId();
    const errorId = `${id}-error`;

    return (
        <div>
            <label htmlFor={id} className={editorLabelClass}>
                {label}
                {required && (
                    <span className="ml-1 text-subtle" aria-hidden>
                        *
                    </span>
                )}
            </label>
            <textarea
                id={id}
                value={value}
                onChange={(e) => onChange(e.target.value)}
                aria-required={required || undefined}
                aria-invalid={error ? true : undefined}
                aria-describedby={error ? errorId : undefined}
                className={`${editorControlClass} min-h-24 resize-y`}
            />
            {error && (
                <p id={errorId} className={errorClass}>
                    {error}
                </p>
            )}
        </div>
    );
}
