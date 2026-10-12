import { useCallback, useEffect, useRef, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { answerReviewCard, listReviewOverview, nextReviewCard, undoReviewAnswer } from "../api/review";
import type { Intervals, NextReviewCard, Rating, ReviewCard, ReviewCounts, ReviewType } from "../types/Review";
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
    /**
     * Counts every time a card is put on screen, so a card that comes back at once (answered
     * Again, or the first pass of a new card) is a new presentation and its audio plays again.
     */
    const [presentation, setPresentation] = useState(0);
    /** What the learner wrote, for the types that are answered by typing. */
    const [typed, setTyped] = useState("");
    /**
     * The answers given in this session, in order. The counts of the summary come from it, and
     * undoing takes the last one off, so the numbers always match what is recorded.
     */
    const [answers, setAnswers] = useState<{ uuid: string; rating: Rating }[]>([]);
    /**
     * An answer was recorded but the next card could not be loaded. The card on screen is
     * already answered, so it must not be answered again: only reloading is offered.
     */
    const [needsReload, setNeedsReload] = useState(false);
    const [intervals, setIntervals] = useState<Intervals | null>(null);
    const [languageTag, setLanguageTag] = useState("");
    const [nextDueInSeconds, setNextDueInSeconds] = useState<number | null>(null);
    const [dailyLimitReached, setDailyLimitReached] = useState(false);
    const [autoplayAudio, setAutoplayAudio] = useState(true);
    /** The other types of this deck that have something waiting, for when this one is done. */
    const [siblings, setSiblings] = useState<ReviewType[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const { error, busy, run } = useAsyncRunner();

    // When the card appeared, to tell how long the answer took.
    const shownAt = useRef(performance.now());

    const show = useCallback((next: NextReviewCard) => {
        setCard(next.card);
        setCounts(next.counts);
        setIntervals(next.intervals);
        setLanguageTag(next.languageTag);
        setNextDueInSeconds(next.nextDueInSeconds);
        setDailyLimitReached(next.dailyLimitReached);
        setAutoplayAudio(next.autoplayAudio);
        setRevealed(false);
        setNeedsReload(false);
        setPresentation((n) => n + 1);
        setTyped("");
        shownAt.current = performance.now();
    }, []);

    useEffect(() => {
        let cancelled = false;
        setStatus("loading");
        setAnswers([]);
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

    // When the session ends, find what else is waiting in the same deck.
    const done = status === "ready" && card === null;
    useEffect(() => {
        if (!done) return;
        let cancelled = false;
        listReviewOverview()
            .then((decks) => {
                const deck = decks.find((d) => d.name === deckName);
                const others = (deck?.types ?? []).filter(
                    (t) => t.slug !== typeSlug && t.counts.new + t.counts.learning + t.counts.review > 0,
                );
                if (!cancelled) setSiblings(others);
            })
            .catch(() => undefined);
        return () => {
            cancelled = true;
        };
    }, [done, deckName, typeSlug]);

    /** Shows the back of the card. */
    const showResponse = () => {
        if (card) setRevealed(true);
    };

    /**
     * Records the answer, then moves to the next card. Only works once the response is shown.
     * If recording fails nothing changes and the card stays as it was. If only loading the next
     * card fails, the answer stays recorded and `needsReload` asks for a retry instead of letting
     * the same card be answered twice.
     */
    const answer = (rating: Rating) => {
        if (!card || !revealed || busy || needsReload) return Promise.resolve(false);
        const durationMs = Math.round(performance.now() - shownAt.current);
        return run(async () => {
            await answerReviewCard(card.uuid, rating, durationMs);
            setAnswers((given) => [...given, { uuid: card.uuid, rating }]);
            setNeedsReload(true);
            show(await nextReviewCard(deckName, typeSlug));
        });
    };

    /** Loads the next card again after a failure that came after the answer was recorded. */
    const reload = () => run(async () => show(await nextReviewCard(deckName, typeSlug)));

    /**
     * Takes back the last answer of the session and shows that card again, as it was. It can be
     * repeated to go back further. If it fails, nothing was undone.
     */
    const undo = () => {
        const last = answers[answers.length - 1];
        if (!last || busy) return Promise.resolve(false);
        return run(async () => {
            const previous = await undoReviewAnswer(deckName, typeSlug, last.uuid);
            setAnswers((given) => given.slice(0, -1));
            show(previous);
        });
    };

    return {
        status,
        card,
        counts,
        revealed,
        presentation,
        typed,
        setTyped,
        /** How many answers were given, counting a card each time it came back. */
        answered: answers.length,
        /** How many of the answers so far were Again: each time a card was missed. */
        again: answers.filter((a) => a.rating === 1).length,
        /** How many different cards were answered. */
        cardsReviewed: new Set(answers.map((a) => a.uuid)).size,
        canUndo: answers.length > 0 && !busy,
        needsReload,
        intervals,
        languageTag,
        /** When the session is over: seconds until the next card of the type is due, if any. */
        nextDueInSeconds,
        /** When the session is over: cards are waiting, held back by the deck's daily limit. */
        dailyLimitReached,
        /** The deck's setting: the audio of a card plays by itself. */
        autoplayAudio,
        siblings,
        /** Nothing is waiting: there was nothing to review, or the last card was answered. */
        done,
        error,
        busy,
        showResponse,
        answer,
        undo,
        reload,
    };
}
