import { CaseSensitive, Layers, LucideIcon, Mic, PencilLine } from "lucide-react";
import type { ParseKeys } from "i18next";

export interface FlashcardTypeStyle {
    icon: LucideIcon;
    /** The translation key of the type's name. */
    labelKey: ParseKeys;
    /** Solid background, for accent bars. */
    accent: string;
    /** Text color. */
    text: string;
    /** Translucent background and text, for badges. */
    badge: string;
    /** Dotted border that becomes solid on hover, for tiles. */
    tile: string;
}

// The colors are the theme colors in `index.css`. Class names are written out in full so
// Tailwind can find them.
export const FLASHCARD_TYPE_STYLES: Record<string, FlashcardTypeStyle> = {
    stack: {
        icon: Layers,
        labelKey: "types.stack",
        accent: "bg-stack",
        text: "text-stack-ink",
        badge: "bg-stack/15 text-stack-ink",
        tile: "border-stack/40 hover:border-stack text-stack-ink",
    },
    vocabulary: {
        icon: CaseSensitive,
        labelKey: "types.vocabulary",
        accent: "bg-vocabulary",
        text: "text-vocabulary-ink",
        badge: "bg-vocabulary/15 text-vocabulary-ink",
        tile: "border-vocabulary/40 hover:border-vocabulary text-vocabulary-ink",
    },
    phonetic: {
        icon: Mic,
        labelKey: "types.phonetic",
        accent: "bg-phonetic",
        text: "text-phonetic",
        badge: "bg-phonetic/15 text-phonetic",
        tile: "border-phonetic/40 hover:border-phonetic text-phonetic",
    },
    orthographic: {
        icon: PencilLine,
        labelKey: "types.orthographic",
        accent: "bg-orthographic",
        text: "text-orthographic",
        badge: "bg-orthographic/15 text-orthographic",
        tile: "border-orthographic/40 hover:border-orthographic text-orthographic",
    },
};

/**
 * What the editor lets you add, in the order of the buttons: the three card types, and the stack,
 * which is not a type but makes one card of each type at once.
 */
export const ENABLED_TYPE_SLUGS: readonly string[] = ["stack", "vocabulary", "phonetic", "orthographic"];

const NEUTRAL: FlashcardTypeStyle = {
    icon: CaseSensitive,
    labelKey: "types.flashcard",
    accent: "bg-white/40",
    text: "text-muted",
    badge: "bg-white/10 text-muted",
    tile: "border-line-strong hover:border-white/50 text-muted",
};

/** The style of a flashcard type, or a neutral one for a type this app has no color for. */
export function styleForType(slug: string): FlashcardTypeStyle {
    return FLASHCARD_TYPE_STYLES[slug] ?? NEUTRAL;
}

/** The translation key of a known flashcard type's name, or `null` for a type this app does not know. */
export function typeLabelKey(slug: string): ParseKeys | null {
    return FLASHCARD_TYPE_STYLES[slug]?.labelKey ?? null;
}
