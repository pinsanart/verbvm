import { invoke } from "@tauri-apps/api/core";
import { MediaDto, toMedia } from "./media";
import type { NextReviewCard, Rating, ReviewCard, ReviewDeck, ReviewField } from "../types/Review";

interface ReviewFieldDto extends Omit<ReviewField, "media"> {
    media: MediaDto[];
}

interface ReviewCardDto extends Omit<ReviewCard, "fields"> {
    fields: ReviewFieldDto[];
}

interface NextReviewCardDto extends Omit<NextReviewCard, "card"> {
    card: ReviewCardDto | null;
}

const toCard = (dto: ReviewCardDto): ReviewCard => ({
    ...dto,
    fields: dto.fields.map((field) => ({ ...field, media: field.media.map(toMedia) })),
});

/** Every deck with the types it has flashcards of and what is waiting in each. */
export const listReviewOverview = () => invoke<ReviewDeck[]>("list_review_overview");

/** The next flashcard to review in a deck's type, or none when nothing is waiting. */
export const nextReviewCard = async (deckName: string, typeSlug: string): Promise<NextReviewCard> => {
    const next = await invoke<NextReviewCardDto>("next_review_card", { deckName, typeSlug });
    return { ...next, card: next.card ? toCard(next.card) : null };
};

/**
 * Takes back the latest answer given to a flashcard and returns that flashcard again, as it was
 * before the answer. Each call goes one answer further back for that card.
 */
export const undoReviewAnswer = async (deckName: string, typeSlug: string, cardUuid: string): Promise<NextReviewCard> => {
    const next = await invoke<NextReviewCardDto>("undo_review_answer", { deckName, typeSlug, cardUuid });
    return { ...next, card: next.card ? toCard(next.card) : null };
};

/** Records the answer to a flashcard. `durationMs` is how long the answer took. */
export const answerReviewCard = (cardUuid: string, rating: Rating, durationMs: number) =>
    invoke<void>("answer_review_card", { cardUuid, rating, durationMs });
