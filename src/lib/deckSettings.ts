import type { DeckSettings } from "../types/Deck";

/** The settings a new deck starts with. They match the backend's defaults. */
export const DEFAULT_DECK_SETTINGS: DeckSettings = {
    newCardsPerDay: 20,
    maxReviewsPerDay: 200,
    autoplayAudio: true,
};

/** The largest daily limit the backend accepts. */
export const MAX_PER_DAY = 9999;

/** The settings as a form holds them: the numbers stay text while they are being typed. */
export interface DeckSettingsDraft {
    newCardsPerDay: string;
    maxReviewsPerDay: string;
    autoplayAudio: boolean;
}

export const toDraft = (settings: DeckSettings): DeckSettingsDraft => ({
    newCardsPerDay: String(settings.newCardsPerDay),
    maxReviewsPerDay: String(settings.maxReviewsPerDay),
    autoplayAudio: settings.autoplayAudio,
});

/** The settings from a form, with an empty or invalid number read as the default for it. */
export const fromDraft = (draft: DeckSettingsDraft): DeckSettings => {
    const read = (text: string, fallback: number) => {
        const value = Number.parseInt(text, 10);
        return Number.isInteger(value) ? Math.min(Math.max(value, 0), MAX_PER_DAY) : fallback;
    };
    return {
        newCardsPerDay: read(draft.newCardsPerDay, DEFAULT_DECK_SETTINGS.newCardsPerDay),
        maxReviewsPerDay: read(draft.maxReviewsPerDay, DEFAULT_DECK_SETTINGS.maxReviewsPerDay),
        autoplayAudio: draft.autoplayAudio,
    };
};
