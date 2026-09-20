import { LucideIcon } from "lucide-react";

interface IconButtonProps {
    label: string;
    icon: LucideIcon;
    iconSize: number
    className?: string;
    onClick?: () => void;
}

export function IconButton({label, icon:Icon, iconSize, className, onClick}: IconButtonProps) {
    return (
        <button
            type="button"
            onClick={onClick}
            className={`inline-flex items-center justify-center rounded-lg p-2 text-white/60 transition-colors hover:bg-white/10 hover:text-white ${className ?? ""}`}
        >
            <label className="sr-only">{label}</label>
            <Icon size={iconSize}/>
        </button>
    )
}