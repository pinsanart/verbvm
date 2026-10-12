import { ENABLED_TYPE_SLUGS, styleForType, TYPE_BUTTON_SLUGS } from "../data/FlashcardTypeStyles";

export interface TypePickerProps {
    /** Called with the slug of the type the user picked. Only enabled types can be picked. */
    onPick: (typeSlug: string) => void;
    disabled?: boolean;
}

/**
 * The buttons that add a card of a type to the deck. Types that are not available yet are shown
 * disabled, with a "Soon" badge.
 */
export function TypePicker({ onPick, disabled }: TypePickerProps) {
    return (
        <div className="flex flex-wrap items-center gap-2" role="group" aria-label="Add a flashcard">
            <span className="mr-1 text-meta font-medium text-muted" aria-hidden>
                Add
            </span>
            {TYPE_BUTTON_SLUGS.map((slug) => {
                const { icon: Icon, label, tile } = styleForType(slug);
                const enabled = ENABLED_TYPE_SLUGS.includes(slug);
                return (
                    <button
                        key={slug}
                        type="button"
                        disabled={!enabled || disabled}
                        onClick={() => onPick(slug)}
                        className={`relative flex items-center gap-2 rounded-lg border-2 border-dotted px-3 py-2 text-ui font-medium text-ink transition-colors disabled:cursor-not-allowed ${tile} ${
                            enabled ? "" : "opacity-40"
                        }`}
                    >
                        <Icon size={18} aria-hidden />
                        {label}
                        {!enabled && (
                            <span className="rounded-full bg-white/10 px-1.5 py-0.5 text-label font-medium text-muted">
                                Soon
                            </span>
                        )}
                    </button>
                );
            })}
        </div>
    );
}
