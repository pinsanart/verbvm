import { CaseSensitive, Layers, LucideIcon, Mic, PencilLine } from "lucide-react";

export interface FlashcardTypeStyle {
    icon: LucideIcon;
    label: string;
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
        label: "Stack",
        accent: "bg-stack",
        text: "text-stack-ink",
        badge: "bg-stack/15 text-stack-ink",
        tile: "border-stack/40 hover:border-stack text-stack-ink",
    },
    vocabulary: {
        icon: CaseSensitive,
        label: "Vocabulary",
        accent: "bg-vocabulary",
        text: "text-vocabulary-ink",
        badge: "bg-vocabulary/15 text-vocabulary-ink",
        tile: "border-vocabulary/40 hover:border-vocabulary text-vocabulary-ink",
    },
    phonetic: {
        icon: Mic,
        label: "Phonetic",
        accent: "bg-phonetic",
        text: "text-phonetic",
        badge: "bg-phonetic/15 text-phonetic",
        tile: "border-phonetic/40 hover:border-phonetic text-phonetic",
    },
    orthographic: {
        icon: PencilLine,
        label: "Orthographic",
        accent: "bg-orthographic",
        text: "text-orthographic",
        badge: "bg-orthographic/15 text-orthographic",
        tile: "border-orthographic/40 hover:border-orthographic text-orthographic",
    },
};

/**
 * The types the editor lets you add today, in the order of their buttons. Phonetic and orthographic
 * have their fields but are not offered yet, and stack is not a type but a way to link cards.
 */
export const ENABLED_TYPE_SLUGS: readonly string[] = ["vocabulary"];

const NEUTRAL: FlashcardTypeStyle = {
    icon: CaseSensitive,
    label: "Flashcard",
    accent: "bg-white/40",
    text: "text-muted",
    badge: "bg-white/10 text-muted",
    tile: "border-line-strong hover:border-white/50 text-muted",
};

/** The style of a flashcard type, or a neutral one for a type this app has no color for. */
export function styleForType(slug: string): FlashcardTypeStyle {
    return FLASHCARD_TYPE_STYLES[slug] ?? NEUTRAL;
}
