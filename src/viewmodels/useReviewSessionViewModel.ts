import { useCallback, useEffect, useRef, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { answerReviewCard, nextReviewCard } from "../api/review";
import type { Rating, ReviewCard, ReviewCounts } from "../types/Review";
import type { LoadStatus } from "./useDecksViewModel";

const NO_COUNTS: ReviewCounts = { new: 0, learning: 0, review: 0 };

/**
 * The view's API for reviewing the flashcards of one type in a deck, one card at a time.
 *
 * The backend keeps the queue. After every answer the next card is asked for again, so a card
 * answered Again comes back by itself when its interval has passed. The viewmodel only knows the
 * card on screen, whether its response is shown and how long the answer took.
 */
export function useReviewSessionViewModel(deckName: string, typeSlug: string) {
    const [card, setCard] = useState<ReviewCard | null>(null);
    const [counts, setCounts] = useState<ReviewCounts>(NO_COUNTS);
    const [revealed, setRevealed] = useState(false);
    const [answered, setAnswered] = useState(0);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const { error, busy, run } = useAsyncRunner();

    // When the card appeared, to tell how long the answer took.
    const shownAt = useRef(performance.now());

    const show = useCallback((next: { card: ReviewCard | null; counts: ReviewCounts }) => {
        setCard(next.card);
        setCounts(next.counts);
        setRevealed(false);
        shownAt.current = performance.now();
    }, []);

    useEffect(() => {
        let cancelled = false;
        setStatus("loading");
        setAnswered(0);
        (async () => {
            const ok = await run(async () => {
                const next = await nextReviewCard(deckName, typeSlug);
                if (!cancelled) show(next);
            });
            if (!cancelled) setStatus(ok ? "ready" : "error");
        })();
        return () => {
            cancelled = true;
        };
    }, [deckName, typeSlug, run, show]);

    /** Shows the back of the card. */
    const showResponse = () => {
        if (card) setRevealed(true);
    };

    /** Records the answer, then moves to the next card. Only works once the response is shown. */
    const answer = (rating: Rating) => {
        if (!card || !revealed || busy) return Promise.resolve(false);
        const durationMs = Math.round(performance.now() - shownAt.current);
        return run(async () => {
            await answerReviewCard(card.uuid, rating, durationMs);
            setAnswered((n) => n + 1);
            show(await nextReviewCard(deckName, typeSlug));
        });
    };

    return {
        status,
        card,
        counts,
        revealed,
        answered,
        /** Nothing is waiting: there was nothing to review, or the last card was answered. */
        done: status === "ready" && card === null,
        error,
        busy,
        showResponse,
        answer,
    };
}
