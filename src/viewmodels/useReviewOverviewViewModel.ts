import { useCallback, useEffect, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { listReviewOverview } from "../api/review";
import type { ReviewDeck } from "../types/Review";
import type { LoadStatus } from "./useDecksViewModel";

/** The view's API for the review overview: the decks, the types each has and what is waiting. */
export function useReviewOverviewViewModel() {
    const [decks, setDecks] = useState<ReviewDeck[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const { error, run } = useAsyncRunner();

    const refresh = useCallback(async () => {
        const ok = await run(async () => setDecks(await listReviewOverview()));
        setStatus(ok ? "ready" : "error");
    }, [run]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    return { decks, status, error, refresh };
}
