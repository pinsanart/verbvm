import type { LanguageSummary } from "./Deck";
import type { FieldSide } from "./FlashcardForm";
import type { LanguageVariant } from "./Language";
import type { Media } from "./Media";

/** How many flashcards of a type are waiting. Cards that are not due yet are in none. */
export interface ReviewCounts {
    /** Never reviewed. */
    new: number;
    /** Due, and the last answer was Again. */
    learning: number;
    /** Due, and recalled before. */
    review: number;
}

export interface ReviewType {
    slug: string;
    name: string;
    counts: ReviewCounts;
}

/** A deck in the review overview. Only the types it has flashcards of are listed. */
export interface ReviewDeck {
    name: string;
    language: LanguageSummary;
    variant: LanguageVariant;
    types: ReviewType[];
}

export type CardState = "new" | "learning" | "review";

export interface ReviewField {
    key: string;
    label: string;
    side: FieldSide;
    /** The content is sanitized HTML. */
    isRich: boolean;
    content: string;
    media: Media[];
}

/** A flashcard to review, identified by its uuid. Fields with no text and no media are left out. */
export interface ReviewCard {
    uuid: string;
    typeSlug: string;
    typeName: string;
    state: CardState;
    fields: ReviewField[];
}

/** How long, in seconds, a card waits before it comes back for each answer. */
export interface Intervals {
    again: number;
    hard: number;
    good: number;
    easy: number;
}

/** The next flashcard of a session, if any, and what is still waiting in the type. */
export interface NextReviewCard {
    card: ReviewCard | null;
    counts: ReviewCounts;
    /** BCP 47 code of the deck's accent, for the `lang` of the text on the card. */
    languageTag: string;
    /** What each answer would schedule for the card. Present when there is a card. */
    intervals: Intervals | null;
    /** When there is no card: seconds until the next one is due, if any is scheduled. */
    nextDueInSeconds: number | null;
}

/** 1 is Again, 2 Hard, 3 Good and 4 Easy. */
export type Rating = 1 | 2 | 3 | 4;
