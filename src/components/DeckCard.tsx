import { motion } from "motion/react";
import { TRANSITION_BASE } from "../lib/motion";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { CountryFlag } from "./CountryFlag";
import { styleForType } from "../data/FlashcardTypeStyles";
import type { Deck } from "../types/Deck";

export interface DeckCardProps {
    deck: Deck;
    onOpen: () => void;
    onDelete: () => void;
}

/** A deck in the deck list: its name, language and accent, description and card counts. */
export function DeckCard({ deck, onOpen, onDelete }: DeckCardProps) {
    return (
        <motion.div
            layout
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0, scale: 0.97 }}
            transition={TRANSITION_BASE}
            className="relative rounded-2xl border border-line bg-panel transition-colors hover:border-line-strong"
        >
            <button type="button" onClick={onOpen} className="flex w-full flex-col gap-3 p-4 text-left">
                <div className="flex items-center gap-3 pr-8">
                    <CountryFlag code={deck.variant.countryCode} className="h-6 w-9 shrink-0" />
                    <div className="min-w-0">
                        <h2 className="truncate text-base font-semibold">{deck.name}</h2>
                        <p className="truncate text-sm text-muted">
                            {deck.language.name} · {deck.variant.variantName}
                        </p>
                    </div>
                </div>

                {deck.description && <p className="line-clamp-2 text-sm text-muted">{deck.description}</p>}

                <div className="flex flex-wrap items-center gap-2 text-xs tabular-nums">
                    <span className="rounded-full bg-white/10 px-2 py-0.5 font-medium text-muted">
                        {deck.stats.totalCards} {deck.stats.totalCards === 1 ? "card" : "cards"}
                    </span>
                    {deck.stats.cardsByType
                        .filter((entry) => entry.count > 0)
                        .map((entry) => (
                            <span
                                key={entry.typeSlug}
                                className={`rounded-full px-2 py-0.5 font-medium ${styleForType(entry.typeSlug).badge}`}
                            >
                                {entry.typeName} {entry.count}
                            </span>
                        ))}
                </div>
            </button>

            <div className="absolute right-2 top-2">
                <ConfirmDeleteButton label={`the deck ${deck.name}`} onConfirm={onDelete} />
            </div>
        </motion.div>
    );
}
