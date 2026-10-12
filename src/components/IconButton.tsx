import { LucideIcon } from "lucide-react";

interface IconButtonProps {
    label: string;
    icon: LucideIcon;
    iconSize: number
    className?: string;
    onClick?: () => void;
    disabled?: boolean;
}

export function IconButton({label, icon:Icon, iconSize, className, onClick, disabled}: IconButtonProps) {
    return (
        <button
            type="button"
            onClick={onClick}
            disabled={disabled}
            className={`inline-flex items-center justify-center rounded-lg p-2 text-white/60 transition-colors hover:bg-white/10 hover:text-white disabled:pointer-events-none disabled:opacity-30 ${className ?? ""}`}
        >
            <label className="sr-only">{label}</label>
            <Icon size={iconSize}/>
        </button>
    )
}