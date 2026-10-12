export interface PlainInputFieldProps {
    label: string;
    value: string;
    onChange: (value: string) => void;
    required?: boolean;
    error?: string;
}

export function PlainInputField({ label, value, onChange, required, error }: PlainInputFieldProps) {
    return (
        <div>
            <label className="mb-1.5 block text-xs font-medium uppercase tracking-wider text-muted">
                {label}
                {required && <span className="ml-1 text-subtle">*</span>}
            </label>
            <textarea
                value={value}
                onChange={(e) => onChange(e.target.value)}
                aria-invalid={error ? true : undefined}
                className={`min-h-20 w-full resize-y rounded-xl border bg-white/5 p-3 text-[15px] leading-6 text-white outline-none transition-colors focus:bg-white/[0.07] ${
                    error ? "border-stack/70" : "border-line focus:border-white/30"
                }`}
            />
            {error && <p className="mt-1 text-xs text-stack-ink">{error}</p>}
        </div>
    );
}
