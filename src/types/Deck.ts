import type { LanguageVariant } from "./Language";

export interface LanguageSummary {
    name: string;
    iso6393: string;
}

export interface TypeCount {
    typeSlug: string;
    typeName: string;
    count: number;
}

export interface DeckStats {
    totalCards: number;
    cardsByType: TypeCount[];
}

/** How a deck is studied. */
export interface DeckSettings {
    /** How many cards seen for the first time a day lets in. */
    newCardsPerDay: number;
    /** How many answers a day lets in for cards that had been recalled before. */
    maxReviewsPerDay: number;
    /** The audio of a card plays by itself in a review. */
    autoplayAudio: boolean;
}

/** A deck, identified by its name. */
export interface Deck extends DeckSettings {
    name: string;
    description: string | null;
    /** UTC, `YYYY-MM-DD HH:MM:SS`. */
    createdAt: string | null;
    language: LanguageSummary;
    variant: LanguageVariant;
    stats: DeckStats;
}

/** What can be changed in a deck. Its language and accent are fixed once it is created. */
export interface UpdateDeck extends DeckSettings {
    name: string;
    description: string | null;
}

export interface NewDeck extends DeckSettings {
    name: string;
    languageIso6393: string;
    /** BCP 47 code of the accent. The language's default variant is used when null. */
    variantBcp47Code: string | null;
    description: string | null;
}
