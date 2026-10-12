import { motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { TRANSITION_BASE } from "../lib/motion";
import { Pencil } from "lucide-react";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { CountryFlag } from "./CountryFlag";
import { IconButton } from "./IconButton";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useLanguageNames } from "../i18n/useLanguageNames";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { Deck } from "../types/Deck";

export interface DeckCardProps {
    deck: Deck;
    onOpen: () => void;
    onEdit: () => void;
    onDelete: () => void;
}

/** A deck in the deck list: its name, language and accent, description and card counts. */
export function DeckCard({ deck, onOpen, onEdit, onDelete }: DeckCardProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const names = useLanguageNames();
    return (
        <motion.div
            layout
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0, scale: 0.97 }}
            transition={TRANSITION_BASE}
            className="relative rounded-2xl border border-line bg-panel transition-colors hover:border-line-strong"
        >
            <div className="flex flex-col gap-3 p-4">
                <div className="flex items-center gap-3 pr-24">
                    <CountryFlag code={deck.variant.countryCode} className="h-6 w-9 shrink-0" />
                    <div className="min-w-0">
                        <h2 className="text-body font-semibold">
                            {/* The name is the control, stretched over the whole card, so the heading stays a heading. */}
                            <button
                                type="button"
                                onClick={onOpen}
                                className="block max-w-full truncate text-left after:absolute after:inset-0 after:rounded-2xl focus-visible:outline-none focus-visible:after:outline focus-visible:after:outline-2 focus-visible:after:outline-offset-2 focus-visible:after:outline-accent-ink"
                            >
                                {deck.name}
                            </button>
                        </h2>
                        <p className="truncate text-meta text-muted">
                            {names.language(deck.variant.bcp47Code, deck.language.name)} · {names.variant(deck.variant)}
                        </p>
                    </div>
                </div>

                {deck.description && <p className="line-clamp-2 text-ui text-muted">{deck.description}</p>}

                <div className="flex flex-wrap items-center gap-2 text-label tabular-nums">
                    <span className="rounded-full bg-white/10 px-2 py-0.5 font-medium text-muted">
                        {t("decks.cards", { count: deck.stats.totalCards })}
                    </span>
                    {deck.stats.cardsByType
                        .filter((entry) => entry.count > 0)
                        .map((entry) => (
                            <span
                                key={entry.typeSlug}
                                className={`rounded-full px-2 py-0.5 font-medium ${styleForType(entry.typeSlug).badge}`}
                            >
                                {typeLabel(entry.typeSlug, entry.typeName)} {entry.count}
                            </span>
                        ))}
                </div>
            </div>

            <div className="absolute right-2 top-2 z-10 flex items-center rounded-lg bg-panel">
                <IconButton label={t("decks.editAria", { name: deck.name })} icon={Pencil} iconSize={16} onClick={onEdit} />
                <ConfirmDeleteButton
                    deleteLabel={t("decks.deleteAria", { name: deck.name })}
                    confirmLabel={t("decks.confirmDeleteAria", { name: deck.name })}
                    onConfirm={onDelete}
                />
            </div>
        </motion.div>
    );
}
