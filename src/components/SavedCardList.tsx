import { useState } from "react";
import { ChevronDown, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import { phraseOf } from "../lib/stackFields";
import { plainText } from "../lib/text";
import type { SavedRow } from "../viewmodels/useSavedCardsViewModel";
import type { FieldDefinition, SavedCard } from "../types/FlashcardForm";

export interface SavedCardListProps {
    /** Every saved card, to count the types for the filter. */
    all: SavedCard[];
    /** The rows that match the search and the filter: cards on their own and stacks as one row. */
    rows: SavedRow[];
    /** The first front and back field of each type, for the row preview. */
    previewKeys: Record<string, { front?: FieldDefinition; back?: FieldDefinition }>;
    query: string;
    typeFilter: string | null;
    onQuery: (query: string) => void;
    onTypeFilter: (typeSlug: string | null) => void;
    /** Opens one card alone, even when it belongs to a stack. */
    onOpen: (uuid: string) => void;
    /** Opens every card of a stack together. */
    onOpenStack: (stackUuid: string) => void;
}

/**
 * The saved cards of a deck with a search box and a filter by type. A card opens its edit panel.
 * The cards of a stack are one row that opens them together, and a chevron shows the cards under
 * it, each of which opens alone.
 */
export function SavedCardList({ all, rows, previewKeys, query, typeFilter, onQuery, onTypeFilter, onOpen, onOpenStack }: SavedCardListProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const typeSlugs = [...new Set(all.map((card) => card.typeSlug))];
    const [expanded, setExpanded] = useState<ReadonlySet<string>>(new Set());
    const shown = rows.reduce((sum, row) => sum + (row.kind === "card" ? 1 : row.cards.length), 0);

    const toggle = (stackUuid: string) =>
        setExpanded((current) => {
            const next = new Set(current);
            if (!next.delete(stackUuid)) next.add(stackUuid);
            return next;
        });

    /** The text a row leads with: the first text field of the type. */
    const preview = (card: SavedCard) => {
        const keys = previewKeys[card.typeSlug] ?? {};
        return {
            front: plainText(card.values[keys.front?.key ?? ""] ?? ""),
            back: plainText(card.values[keys.back?.key ?? ""] ?? ""),
        };
    };

    /** A card on its own, or one card under its stack. */
    const renderCard = (card: SavedCard, inStack: boolean) => {
        const { icon: Icon, badge } = styleForType(card.typeSlug);
        const { front, back } = preview(card);
        return (
            <button
                type="button"
                onClick={() => onOpen(card.uuid)}
                className={`flex w-full items-center gap-3 text-left transition-colors ${
                    inStack ? "px-3 py-2.5 hover:bg-white/5" : "rounded-xl border border-line bg-panel p-3 hover:border-line-strong"
                }`}
            >
                <span className={`flex shrink-0 items-center gap-1.5 rounded-full px-2.5 py-1 text-label font-medium ${badge}`}>
                    <Icon size={14} aria-hidden />
                    {typeLabel(card.typeSlug)}
                </span>
                <span className="min-w-0 flex-1">
                    <span className="block truncate text-ui font-medium text-ink">
                        {front || <span className="text-muted">{t("savedList.mediaOnly")}</span>}
                    </span>
                    {back && <span className="block truncate text-meta text-muted">{back}</span>}
                </span>
            </button>
        );
    };

    /** A stack: one row that opens them all, a chevron that shows the cards under it. */
    const renderStack = (row: Extract<SavedRow, { kind: "stack" }>) => {
        const { icon: StackIcon, badge } = styleForType("stack");
        const open = expanded.has(row.stackUuid);
        const phrase = row.cards.map((card) => phraseOf(card.values)).find((text) => text !== "") ?? "";
        const listId = `stack-${row.stackUuid}`;
        return (
            <li key={row.stackUuid} className="overflow-hidden rounded-xl border border-line bg-panel">
                <div className="flex items-stretch">
                    <button
                        type="button"
                        onClick={() => onOpenStack(row.stackUuid)}
                        aria-label={t("savedList.openStackAria", { phrase })}
                        className="flex min-w-0 flex-1 items-center gap-3 p-3 text-left transition-colors hover:bg-white/5"
                    >
                        <span className={`flex shrink-0 items-center gap-1.5 rounded-full px-2.5 py-1 text-label font-medium ${badge}`}>
                            <StackIcon size={14} aria-hidden />
                            {typeLabel("stack")}
                        </span>
                        <span className="min-w-0 flex-1 truncate text-ui font-medium text-ink">
                            {phrase || <span className="text-muted">{t("savedList.mediaOnly")}</span>}
                        </span>
                        <span className="flex shrink-0 items-center gap-1.5" aria-hidden>
                            {row.cards.map((card) => {
                                const { icon: Icon, text } = styleForType(card.typeSlug);
                                return <Icon key={card.uuid} size={16} className={text} />;
                            })}
                        </span>
                    </button>
                    <button
                        type="button"
                        aria-expanded={open}
                        aria-controls={listId}
                        aria-label={t(open ? "savedList.collapseAria" : "savedList.expandAria")}
                        onClick={() => toggle(row.stackUuid)}
                        className="flex w-11 shrink-0 items-center justify-center border-l border-line text-muted transition-colors hover:bg-white/5 hover:text-ink"
                    >
                        <ChevronDown size={18} aria-hidden className={`transition-transform ${open ? "rotate-180" : ""}`} />
                    </button>
                </div>
                {open && (
                    <ul id={listId} className="divide-y divide-line border-t border-line pl-4">
                        {row.cards.map((card) => (
                            <li key={card.uuid}>{renderCard(card, true)}</li>
                        ))}
                    </ul>
                )}
            </li>
        );
    };

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
                {shown === all.length
                    ? t("savedList.count", { count: all.length })
                    : t("savedList.countOf", { shown, total: all.length })}
            </p>

            {rows.length === 0 ? (
                <p className="rounded-2xl border border-dashed border-line px-6 py-10 text-center text-ui text-muted">
                    {t("savedList.noMatch")}
                </p>
            ) : (
                <ul className="flex flex-col gap-2">
                    {rows.map((row) => {
                        if (row.kind === "stack") return renderStack(row);
                        return <li key={row.card.uuid}>{renderCard(row.card, false)}</li>;
                    })}
                </ul>
            )}
        </div>
    );
}
