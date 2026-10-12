import { Plus } from "lucide-react";
import { ENABLED_TYPE_SLUGS, styleForType } from "../data/FlashcardTypeStyles";

export interface TypePickerProps {
    /** Called with the slug of the type the user picked. */
    onPick: (typeSlug: string) => void;
    disabled?: boolean;
}

/**
 * The buttons that add a card of a type to the deck. Only the types that can be written today are
 * shown, so every button here does something. The others appear when they are ready.
 */
export function TypePicker({ onPick, disabled }: TypePickerProps) {
    return (
        <div className="flex flex-wrap items-center gap-2" role="group" aria-label="Add a flashcard">
            {ENABLED_TYPE_SLUGS.map((slug) => {
                const { icon: Icon, label, tile } = styleForType(slug);
                return (
                    <button
                        key={slug}
                        type="button"
                        disabled={disabled}
                        onClick={() => onPick(slug)}
                        className={`flex items-center gap-2 rounded-lg border-2 border-dotted px-3 py-2 text-ui font-medium text-ink transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${tile}`}
                    >
                        <Plus size={16} aria-hidden />
                        <Icon size={18} aria-hidden />
                        {label}
                    </button>
                );
            })}
        </div>
    );
}
