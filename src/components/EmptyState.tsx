import { LucideIcon } from "lucide-react";

export interface EmptyStateProps {
    icon: LucideIcon;
    title: string;
    /** What this place is for and what to do next. */
    description: string;
    action?: React.ReactNode;
}

/** What a screen shows when it has nothing yet. It teaches the next step instead of saying "empty". */
export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
    return (
        <div className="flex flex-col items-center gap-3 rounded-2xl border border-dashed border-line px-6 py-12 text-center">
            <Icon size={24} className="text-subtle" aria-hidden />
            <h2 className="text-body font-semibold text-ink">{title}</h2>
            <p className="max-w-sm text-ui text-muted">{description}</p>
            {action && <div className="mt-2">{action}</div>}
        </div>
    );
}
