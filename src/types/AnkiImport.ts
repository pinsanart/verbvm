/** What was brought in for one language of the Anki collection. */
export interface DeckImport {
    name: string;
    /** Stacks of two cards (vocabulary and phonetic). */
    stacks: number;
    /** Vocabulary cards with no pronunciation to pair with. */
    vocabulary: number;
    /** Phonetic cards with no vocabulary to pair with. */
    phonetic: number;
    images: number;
    /** Why nothing was created for this language, if so. */
    skipped: string | null;
}

export interface AnkiImportReport {
    decks: DeckImport[];
    warnings: string[];
    /** Anki had the collection open, so its latest changes may be missing. */
    partial: boolean;
}
