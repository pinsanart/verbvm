import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";

export interface NewDeckTileProps {
    onClick: () => void;
    /** True when there are no decks, so the tile also explains what a deck is. */
    first?: boolean;
    ref?: React.Ref<HTMLButtonElement>;
}

/**
 * A deck-shaped slot at the end of the list that starts a new deck, so adding one reads as
 * adding to the list.
 */
export function NewDeckTile({ onClick, first, ref }: NewDeckTileProps) {
    const { t } = useTranslation();
    return (
        <button
            ref={ref}
            type="button"
            onClick={onClick}
            className="flex min-h-32 w-full flex-col items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-line px-6 py-6 text-center text-muted transition-colors hover:border-line-strong hover:text-ink"
        >
            <Plus size={28} aria-hidden />
            <span className="text-body font-medium">{t("decks.newDeck")}</span>
            {first && (
                <span className="max-w-xs text-ui text-subtle">
                    {t("decks.newDeckFirstHint")}
                </span>
            )}
        </button>
    );
}
