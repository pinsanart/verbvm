import { Media } from "./Media";

export interface Field {
    content: string;
    media?: Media[];
}

interface FlashcardBase {
    id: string;
}

export interface VocabularyFlashcard extends FlashcardBase {
    type: "vocabulary";
    front: { phrase: Field };
    back: { wordOrExpression: Field; meaning: Field };
}

export interface PhoneticFlashcard extends FlashcardBase {
    type: "phonetic";
    front: { phrase: Field };
    back: { ipaPhonetics: Field };
}

export interface OrthographicFlashcard extends FlashcardBase {
    type: "orthographic";
    front: { phrase: Field };
}

export type Flashcard = VocabularyFlashcard | PhoneticFlashcard | OrthographicFlashcard;
