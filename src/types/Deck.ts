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

/** How the cards of one flashcard type of a deck are studied. */
export interface DeckSettings {
    /** How many cards seen for the first time a day lets in. */
    newCardsPerDay: number;
    /** How many answers a day lets in for cards that had been recalled before. */
    maxReviewsPerDay: number;
    /** The audio of a card plays by itself in a review. */
    autoplayAudio: boolean;
}

/** The study options of one flashcard type of a deck. */
export interface TypeSettings extends DeckSettings {
    typeSlug: string;
}

/** A deck, identified by its name. */
export interface Deck {
    name: string;
    description: string | null;
    /** UTC, `YYYY-MM-DD HH:MM:SS`. */
    createdAt: string | null;
    language: LanguageSummary;
    variant: LanguageVariant;
    stats: DeckStats;
    /** The options of each flashcard type of the deck. */
    typeSettings: TypeSettings[];
}

/** What can be changed in a deck. Its language and accent are fixed once it is created. */
export interface UpdateDeck {
    name: string;
    description: string | null;
    /** The options of the types to change. A type that is left out keeps its own. */
    typeSettings: TypeSettings[];
}

export interface NewDeck {
    name: string;
    languageIso6393: string;
    /** BCP 47 code of the accent. The language's default variant is used when null. */
    variantBcp47Code: string | null;
    description: string | null;
}
