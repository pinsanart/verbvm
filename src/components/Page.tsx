import { ArrowLeft } from "lucide-react";
import { Link } from "react-router";

export interface PageProps {
    title: string;
    /** A link to the screen this one belongs to. */
    back?: { to: string; label: string };
    /** A line under the title, such as the language of a deck. */
    meta?: React.ReactNode;
    /** Controls for the whole page, aligned with the title. */
    actions?: React.ReactNode;
    /** `reading` suits a single column of forms, `wide` suits a grid of cards. */
    width?: "reading" | "wide";
    children: React.ReactNode;
}

/** The frame every screen shares: width, padding, the title and the spacing under it. */
export function Page({ title, back, meta, actions, width = "reading", children }: PageProps) {
    return (
        <div className={`mx-auto w-full px-4 py-6 md:px-8 md:py-10 ${width === "wide" ? "max-w-4xl" : "max-w-3xl"}`}>
            <header className="mb-8 flex flex-col gap-3">
                {back && (
                    <Link
                        to={back.to}
                        className="inline-flex w-fit items-center gap-1.5 text-meta text-muted transition-colors hover:text-ink"
                    >
                        <ArrowLeft size={14} aria-hidden />
                        {back.label}
                    </Link>
                )}
                <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                        <h1 className="text-title">{title}</h1>
                        {meta && <div className="mt-1 flex items-center gap-2 text-meta text-muted">{meta}</div>}
                    </div>
                    {actions && <div className="shrink-0">{actions}</div>}
                </div>
            </header>
            <div className="flex flex-col gap-6">{children}</div>
        </div>
    );
}
