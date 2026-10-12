import { useId } from "react";
import { Trans, useTranslation } from "react-i18next";
import { Link } from "react-router";
import { AnimatePresence, motion } from "motion/react";
import { Check, ChevronDown } from "lucide-react";
import { buttonClass } from "./Button";
import { CountryFlag } from "./CountryFlag";
import { ReviewCountList } from "./ReviewCountList";
import { styleForType } from "../data/FlashcardTypeStyles";
import { TRANSITION_BASE } from "../lib/motion";
import { useLanguageNames } from "../i18n/useLanguageNames";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { ReviewDeck } from "../types/Review";

export interface ReviewDeckRowProps {
    deck: ReviewDeck;
    collapsed: boolean;
    onToggle: () => void;
}

/**
 * A deck in the review overview: its flag, name and accent, then one column for each flashcard
 * type it has cards of, with what is waiting and the way into reviewing it. The header collapses
 * the columns, and then shows how many cards are waiting in all, so nothing is hidden.
 */
export function ReviewDeckRow({ deck, collapsed, onToggle }: ReviewDeckRowProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const names = useLanguageNames();
    const titleId = useId();
    const deckPath = encodeURIComponent(deck.name);
    const bodyId = `${titleId}-body`;
    const waiting = deck.types.reduce((sum, type) => sum + type.counts.new + type.counts.learning + type.counts.review, 0);

    return (
        <section aria-labelledby={titleId} className="overflow-hidden rounded-2xl border border-line bg-panel">
            <header className={collapsed ? "" : "border-b border-line"}>
                <button
                    type="button"
                    onClick={onToggle}
                    aria-expanded={!collapsed}
                    aria-controls={bodyId}
                    aria-label={t(collapsed ? "review.expandDeck" : "review.collapseDeck", { deck: deck.name })}
                    className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-white/5 md:px-5"
                >
                    <ChevronDown
                        size={18}
                        aria-hidden
                        className={`shrink-0 text-muted transition-transform ${collapsed ? "-rotate-90" : ""}`}
                    />
                    <CountryFlag code={deck.variant.countryCode} className="h-5 w-7 shrink-0" />
                    <h2 id={titleId} className="min-w-0 truncate text-body font-semibold">
                        {deck.name}
                    </h2>
                    <p className="shrink-0 text-meta text-muted">{names.variant(deck.variant)}</p>
                    {collapsed && (
                        <p className="ml-auto shrink-0 text-meta tabular-nums text-muted">
                            {waiting > 0 ? t("review.waitingCount", { count: waiting }) : t("review.completed")}
                        </p>
                    )}
                </button>
            </header>

            <AnimatePresence initial={false}>
                {!collapsed && (
                    <motion.div
                        key="body"
                        id={bodyId}
                        initial={{ height: 0, opacity: 0 }}
                        animate={{ height: "auto", opacity: 1 }}
                        exit={{ height: 0, opacity: 0 }}
                        transition={TRANSITION_BASE}
                        className="overflow-hidden"
                    >
                    {deck.types.length === 0 ? (
                        <p className="px-4 py-5 text-ui text-muted md:px-5">
                            <Trans
                                i18nKey="review.noFlashcards"
                                components={{ add: <Link to={`/decks/${deckPath}`} className="text-accent-ink underline" /> }}
                            />
                        </p>
                    ) : (
                        <ul className="grid divide-y divide-line sm:grid-flow-col sm:auto-cols-fr sm:divide-x sm:divide-y-0">
                            {deck.types.map((type) => {
                                const { icon: Icon, text } = styleForType(type.slug);
                                const name = typeLabel(type.slug, type.name);
                                const completed = type.counts.new + type.counts.learning + type.counts.review === 0;
                                return (
                                    <li key={type.slug} className="flex flex-col gap-4 p-4 md:p-5">
                                        <h3 className={`flex items-center gap-2 text-ui font-semibold ${text}`}>
                                            <Icon size={18} aria-hidden />
                                            {name}
                                        </h3>
                                        <ReviewCountList counts={type.counts} />
                                        {completed ? (
                                            <span className="inline-flex items-center justify-center gap-1.5 rounded-lg bg-orthographic px-3 py-2 text-ui font-semibold text-canvas">
                                                <Check size={16} aria-hidden />
                                                {t("review.completed")}
                                            </span>
                                        ) : (
                                            <Link
                                                to={`/review/${deckPath}/${type.slug}`}
                                                aria-label={t("review.reviewAria", { type: name, deck: deck.name })}
                                                className={buttonClass("primary")}
                                            >
                                                {t("review.reviewAction")}
                                            </Link>
                                        )}
                                    </li>
                                );
                            })}
                        </ul>
                    )}
                    </motion.div>
                )}
            </AnimatePresence>
        </section>
    );
}
