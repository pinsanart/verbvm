import { ChevronLeft, ChevronRight } from "lucide-react";
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
 * where you are. On a phone the title is large, uppercase and centered, with a back arrow on
 * one side and the page's actions on the other. From the medium breakpoint up it is a
 * breadcrumb at the start of the bar. Either way the content starts right below.
 */
export function Page({ title, parents = [], meta, actions, width = "reading", children }: PageProps) {
    const column = `mx-auto w-full px-4 md:px-8 ${width === "wide" ? "max-w-4xl" : "max-w-3xl"}`;
    const lastParent = parents.length - 1;

    return (
        <div>
            <header className="sticky top-0 z-20 border-b border-line bg-canvas/85 backdrop-blur">
                <div
                    className={`${column} grid min-h-16 grid-cols-[1fr_auto_1fr] items-center gap-2 py-2 md:flex md:min-h-14 md:gap-1.5`}
                >
                    <nav aria-label="Breadcrumb" className="justify-self-start md:min-w-0">
                        <ol className="flex items-center gap-1.5 text-ui">
                            {parents.map((parent, index) => (
                                // Phones show only a back arrow to the closest parent.
                                <li
                                    key={parent.to}
                                    className={`shrink-0 items-center gap-1.5 ${index === lastParent ? "flex" : "hidden md:flex"}`}
                                >
                                    <Link
                                        to={parent.to}
                                        className="inline-flex items-center text-muted transition-colors hover:text-ink"
                                    >
                                        <ChevronLeft size={24} className="md:hidden" aria-hidden />
                                        <span className="sr-only md:not-sr-only">{parent.label}</span>
                                    </Link>
                                    <ChevronRight size={14} className="hidden text-subtle md:block" aria-hidden />
                                </li>
                            ))}
                        </ol>
                    </nav>

                    <div className="flex min-w-0 flex-col items-center md:flex-row md:gap-3">
                        <h1 className="max-w-full truncate text-center text-heading font-semibold uppercase tracking-[0.08em] md:text-body">
                            {title}
                        </h1>
                        {meta && (
                            <span className="flex shrink-0 items-center gap-2 text-meta text-muted">{meta}</span>
                        )}
                    </div>

                    <div className="justify-self-end md:ml-auto md:shrink-0 md:pl-4">{actions}</div>
                </div>
            </header>
            <div className={`${column} flex flex-col gap-6 py-6 md:py-8`}>{children}</div>
        </div>
    );
}
