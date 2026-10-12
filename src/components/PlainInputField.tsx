import { useId } from "react";
import { controlClass, errorClass, labelClass } from "./formStyles";

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
            <label htmlFor={id} className={labelClass}>
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
                className={`${controlClass} min-h-20 resize-y leading-6`}
            />
            {error && (
                <p id={errorId} className={errorClass}>
                    {error}
                </p>
            )}
        </div>
    );
}
