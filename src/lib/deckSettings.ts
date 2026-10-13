import type { DeckSettings } from "../types/Deck";

/** The settings a new deck starts with. They match the backend's defaults. */
export const DEFAULT_DECK_SETTINGS: DeckSettings = {
    newCardsPerDay: 20,
    maxReviewsPerDay: 200,
    autoplayAudio: true,
    // 03:00 in São Paulo.
    dayStartsAt: "06:00",
};

/** The largest daily limit the backend accepts. */
export const MAX_PER_DAY = 9999;

/** The settings as a form holds them: the numbers stay text while they are being typed. */
export interface DeckSettingsDraft {
    newCardsPerDay: string;
    maxReviewsPerDay: string;
    autoplayAudio: boolean;
    dayStartsAt: string;
}

export const toDraft = (settings: DeckSettings): DeckSettingsDraft => ({
    newCardsPerDay: String(settings.newCardsPerDay),
    maxReviewsPerDay: String(settings.maxReviewsPerDay),
    autoplayAudio: settings.autoplayAudio,
    dayStartsAt: settings.dayStartsAt,
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
        dayStartsAt: /^([01]\d|2[0-3]):[0-5]\d$/.test(draft.dayStartsAt) ? draft.dayStartsAt : DEFAULT_DECK_SETTINGS.dayStartsAt,
    };
};

/**
 * What an `HH:MM` time in UTC is on this machine, for the hint under the field. It uses today's
 * date, so a daylight saving offset is the current one.
 */
export function localTimeOfUtc(utcTime: string, locale: string): string | null {
    const match = /^(\d{2}):(\d{2})$/.exec(utcTime);
    if (!match) return null;
    const now = new Date();
    const at = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate(), Number(match[1]), Number(match[2])));
    return new Intl.DateTimeFormat(locale, { hour: "2-digit", minute: "2-digit", hour12: false }).format(at);
}
