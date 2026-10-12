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
        text: "text-stack",
        badge: "bg-stack/15 text-stack",
        tile: "border-stack/40 hover:border-stack text-stack",
    },
    vocabulary: {
        icon: CaseSensitive,
        label: "Vocabulary",
        accent: "bg-vocabulary",
        text: "text-vocabulary",
        badge: "bg-vocabulary/15 text-vocabulary",
        tile: "border-vocabulary/40 hover:border-vocabulary text-vocabulary",
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

const NEUTRAL: FlashcardTypeStyle = {
    icon: CaseSensitive,
    label: "Flashcard",
    accent: "bg-white/40",
    text: "text-white/70",
    badge: "bg-white/10 text-white/70",
    tile: "border-white/20 hover:border-white/50 text-white/70",
};

/** The style of a flashcard type, or a neutral one for a type this app has no color for. */
export function styleForType(slug: string): FlashcardTypeStyle {
    return FLASHCARD_TYPE_STYLES[slug] ?? NEUTRAL;
}
