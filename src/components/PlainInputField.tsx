export interface PlainInputFieldProps {
    label: string;
    value: string;
    onChange: (value: string) => void;
}

export function PlainInputField({ label, value, onChange }: PlainInputFieldProps) {
    return (
        <div>
            <label className="mb-1.5 block text-xs font-medium uppercase tracking-wide text-white/40">{label}</label>
            <textarea
                value={value}
                onChange={(e) => onChange(e.target.value)}
                className="min-h-20 w-full resize-y rounded-xl border border-white/10 bg-white/5 p-3 text-sm text-white outline-none transition-colors placeholder:text-white/30 focus:border-white/30 focus:bg-white/[0.07]"
            />
        </div>
    );
}
