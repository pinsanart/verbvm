import type { LanguageVariant } from "./Language";
import type { LanguageSummary } from "./Deck";
import type { Media } from "./Media";

export type FieldSide = "FRONT" | "BACK";

/**
 * How a field is filled in. An audio field holds only audio, uploaded or generated. An IPA field
 * holds HTML with the pronunciation of the words of another field, and can be generated.
 */
export type FieldKind = "PLAIN" | "RICH" | "AUDIO" | "IPA";

/** One field of a flashcard type, as defined by the backend for the deck's language. */
export interface FieldDefinition {
    key: string;
    label: string;
    side: FieldSide;
    position: number;
    isRequired: boolean;
    /** The content is HTML and is edited in the rich editor. */
    isRich: boolean;
    kind: FieldKind;
    /** For an audio or IPA field, the key of the field it is generated from. */
    sourceKey: string | null;
    mediaAllowed: boolean;
}

/** A flashcard type with its fields in the deck's language. */
export interface TypeForm {
    slug: string;
    name: string;
    fields: FieldDefinition[];
}

/**
 * What the editor shows for a deck: its language and accent, and the form of every flashcard
 * type, since a deck has one list of drafts that mixes types.
 */
export interface FlashcardForm {
    deckName: string;
    language: LanguageSummary;
    variant: LanguageVariant;
    types: TypeForm[];
}

/**
 * A flashcard being edited, identified by its uuid. `values` maps a field key to its text and
 * `media` to its attached files.
 */
export interface Draft {
    uuid: string;
    /** The slug of the draft's flashcard type. */
    typeSlug: string;
    position: number;
    values: Record<string, string>;
    media: Record<string, Media[]>;
    /** The drafts that share this value are one stack: shown as one, linked once published. */
    stackUuid: string | null;
}

/** A flashcard that is already saved, shaped like a draft so the same form edits both. */
export interface SavedCard {
    uuid: string;
    typeSlug: string;
    values: Record<string, string>;
    media: Record<string, Media[]>;
    /** UTC, `YYYY-MM-DD HH:MM:SS`. */
    createdAt: string | null;
    updatedAt: string | null;
    /** The cards that share this value were made together and are edited together. */
    stackUuid: string | null;
}

/** What a generation produced, with what did not work or was left out (for example the words with no pronunciation). */
export interface Generated<T> {
    value: T;
    warnings: string[];
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
