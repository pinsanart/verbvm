import { Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import { plainText } from "../lib/text";
import type { FieldDefinition, SavedCard } from "../types/FlashcardForm";

export interface SavedCardListProps {
    /** Every saved card, to count the types for the filter. */
    all: SavedCard[];
    /** The cards that match the search and the filter. */
    cards: SavedCard[];
    /** The first front and back field of each type, for the row preview. */
    previewKeys: Record<string, { front?: FieldDefinition; back?: FieldDefinition }>;
    query: string;
    typeFilter: string | null;
    onQuery: (query: string) => void;
    onTypeFilter: (typeSlug: string | null) => void;
    onOpen: (uuid: string) => void;
}

/** The saved cards of a deck with a search box and a filter by type. A row opens the edit panel. */
export function SavedCardList({ all, cards, previewKeys, query, typeFilter, onQuery, onTypeFilter, onOpen }: SavedCardListProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const typeSlugs = [...new Set(all.map((card) => card.typeSlug))];

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-wrap items-center gap-3">
                <label className="relative min-w-48 flex-1">
                    <span className="sr-only">{t("savedList.search")}</span>
                    <Search size={16} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-subtle" aria-hidden />
                    <input
                        type="search"
                        value={query}
                        onChange={(e) => onQuery(e.target.value)}
                        placeholder={t("savedList.search")}
                        className="w-full rounded-lg border border-line bg-panel py-2 pl-9 pr-3 text-ui text-ink placeholder:text-subtle"
                    />
                </label>
                {typeSlugs.length > 1 && (
                    <div className="flex items-center gap-1.5" role="group" aria-label={t("savedList.filterAria")}>
                        {[null, ...typeSlugs].map((slug) => {
                            const active = slug === typeFilter;
                            return (
                                <button
                                    key={slug ?? "all"}
                                    type="button"
                                    aria-pressed={active}
                                    onClick={() => onTypeFilter(slug)}
                                    className={`min-h-10 rounded-full border px-3 text-meta font-medium transition-colors md:min-h-8 ${
                                        active ? "border-ink bg-ink text-canvas" : "border-line text-muted hover:border-line-strong hover:text-ink"
                                    }`}
                                >
                                    {slug ? typeLabel(slug) : t("savedList.all")}
                                </button>
                            );
                        })}
                    </div>
                )}
            </div>

            <p className="text-meta text-muted" role="status">
                {cards.length === all.length
                    ? t("savedList.count", { count: all.length })
                    : t("savedList.countOf", { shown: cards.length, total: all.length })}
            </p>

            {cards.length === 0 ? (
                <p className="rounded-2xl border border-dashed border-line px-6 py-10 text-center text-ui text-muted">
                    {t("savedList.noMatch")}
                </p>
            ) : (
                <ul className="flex flex-col gap-2">
                    {cards.map((card) => {
                        const { icon: Icon, badge } = styleForType(card.typeSlug);
                        const label = typeLabel(card.typeSlug);
                        const keys = previewKeys[card.typeSlug] ?? {};
                        const front = plainText(card.values[keys.front?.key ?? ""] ?? "");
                        const back = plainText(card.values[keys.back?.key ?? ""] ?? "");
                        return (
                            <li key={card.uuid}>
                                <button
                                    type="button"
                                    onClick={() => onOpen(card.uuid)}
                                    className="flex w-full items-center gap-3 rounded-xl border border-line bg-panel p-3 text-left transition-colors hover:border-line-strong"
                                >
                                    <span className={`flex shrink-0 items-center gap-1.5 rounded-full px-2.5 py-1 text-label font-medium ${badge}`}>
                                        <Icon size={14} aria-hidden />
                                        {label}
                                    </span>
                                    <span className="min-w-0 flex-1">
                                        <span className="block truncate text-ui font-medium text-ink">
                                            {front || <span className="text-muted">{t("savedList.mediaOnly")}</span>}
                                        </span>
                                        {back && <span className="block truncate text-meta text-muted">{back}</span>}
                                    </span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
            )}
        </div>
    );
}
