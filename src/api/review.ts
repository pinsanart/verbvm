import { invoke } from "@tauri-apps/api/core";
import { MediaDto, toMedia } from "./media";
import type { NextReviewCard, Rating, ReviewCard, ReviewCounts, ReviewDeck, ReviewField } from "../types/Review";

interface ReviewFieldDto extends Omit<ReviewField, "media"> {
    media: MediaDto[];
}

interface ReviewCardDto extends Omit<ReviewCard, "fields"> {
    fields: ReviewFieldDto[];
}

interface NextReviewCardDto {
    card: ReviewCardDto | null;
    counts: ReviewCounts;
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
    return { card: next.card ? toCard(next.card) : null, counts: next.counts };
};

/** Records the answer to a flashcard. `durationMs` is how long the answer took. */
export const answerReviewCard = (cardUuid: string, rating: Rating, durationMs: number) =>
    invoke<void>("answer_review_card", { cardUuid, rating, durationMs });
