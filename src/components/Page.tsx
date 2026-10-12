import { ChevronRight } from "lucide-react";
import { Link } from "react-router";

export interface PageProps {
    /** The name of this screen. It is the last step of the breadcrumb and the page's `h1`. */
    title: string;
    /** The screens this one sits under, outermost first, as links. */
    parents?: { to: string; label: string }[];
    /** Context next to the title, such as the language of a deck. */
    meta?: React.ReactNode;
    /** Controls for the whole page, at the end of the bar. */
    actions?: React.ReactNode;
    /** `reading` suits a single column of forms, `wide` suits a grid of cards. */
    width?: "reading" | "wide";
    children: React.ReactNode;
}

/**
 * The frame every screen shares. A slim bar stays at the top while the page scrolls and says
 * where you are as a breadcrumb, so the title costs one line instead of a large heading, and
 * the content starts right below it.
 */
export function Page({ title, parents = [], meta, actions, width = "reading", children }: PageProps) {
    const column = `mx-auto w-full px-4 md:px-8 ${width === "wide" ? "max-w-4xl" : "max-w-3xl"}`;

    return (
        <div>
            <header className="sticky top-0 z-20 border-b border-line bg-canvas/85 backdrop-blur">
                <div className={`${column} flex h-14 items-center justify-between gap-4`}>
                    <nav aria-label="Breadcrumb" className="min-w-0">
                        <ol className="flex min-w-0 items-center gap-1.5 text-ui">
                            {parents.map((parent) => (
                                <li key={parent.to} className="flex shrink-0 items-center gap-1.5">
                                    <Link to={parent.to} className="text-muted transition-colors hover:text-ink">
                                        {parent.label}
                                    </Link>
                                    <ChevronRight size={14} className="text-subtle" aria-hidden />
                                </li>
                            ))}
                            <li className="flex min-w-0 items-center gap-3">
                                <h1 className="truncate text-body font-semibold">{title}</h1>
                                {meta && (
                                    <span className="flex shrink-0 items-center gap-2 text-meta text-muted">{meta}</span>
                                )}
                            </li>
                        </ol>
                    </nav>
                    {actions && <div className="shrink-0">{actions}</div>}
                </div>
            </header>
            <div className={`${column} flex flex-col gap-6 py-6 md:py-8`}>{children}</div>
        </div>
    );
}
