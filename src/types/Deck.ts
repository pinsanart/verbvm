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

/** A deck, identified by its name. */
export interface Deck {
    name: string;
    description: string | null;
    /** UTC, `YYYY-MM-DD HH:MM:SS`. */
    createdAt: string | null;
    language: LanguageSummary;
    variant: LanguageVariant;
    stats: DeckStats;
}

export interface NewDeck {
    name: string;
    languageIso6393: string;
    /** BCP 47 code of the accent. The language's default variant is used when null. */
    variantBcp47Code: string | null;
    description: string | null;
}
