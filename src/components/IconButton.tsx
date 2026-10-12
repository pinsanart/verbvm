import { LucideIcon } from "lucide-react";

interface IconButtonProps {
    label: string;
    icon: LucideIcon;
    iconSize: number;
    className?: string;
    onClick?: () => void;
    disabled?: boolean;
    /** Marks a toggle button as on or off. */
    pressed?: boolean;
    ref?: React.Ref<HTMLButtonElement>;
}

/** An icon-only button. The label names it for screen readers and shows as a tooltip. */
export function IconButton({ label, icon: Icon, iconSize, className, onClick, disabled, pressed, ref }: IconButtonProps) {
    return (
        <button
            ref={ref}
            type="button"
            aria-label={label}
            title={label}
            aria-pressed={pressed}
            onClick={onClick}
            disabled={disabled}
            className={`inline-flex size-10 items-center justify-center rounded-lg text-muted transition-colors hover:bg-white/10 hover:text-ink aria-pressed:bg-white/10 aria-pressed:text-ink disabled:pointer-events-none disabled:opacity-30 md:size-9 ${className ?? ""}`}
        >
            <Icon size={iconSize} aria-hidden />
        </button>
    );
}
