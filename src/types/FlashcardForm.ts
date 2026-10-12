import type { LanguageVariant } from "./Language";
import type { LanguageSummary } from "./Deck";

export type FieldSide = "FRONT" | "BACK";

/** One field of a flashcard type, as defined by the backend for the deck's language. */
export interface FieldDefinition {
    key: string;
    label: string;
    side: FieldSide;
    position: number;
    isRequired: boolean;
    /** The content is HTML and is edited in the rich editor. */
    isRich: boolean;
    mediaAllowed: boolean;
}

export interface FlashcardForm {
    deckName: string;
    language: LanguageSummary;
    variant: LanguageVariant;
    flashcardType: { slug: string; name: string };
    fields: FieldDefinition[];
}

/** A flashcard being edited, identified by its uuid. `values` maps a field key to its content. */
export interface Draft {
    uuid: string;
    position: number;
    values: Record<string, string>;
}

export interface DraftIssue {
    draftUuid: string;
    fieldKey: string | null;
    message: string;
}

/** When `issues` is not empty nothing was saved. */
export interface PublishResult {
    published: number;
    issues: DraftIssue[];
}

export type MoveDirection = "up" | "down";
