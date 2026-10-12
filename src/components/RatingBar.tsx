import { AnimatePresence, motion } from "motion/react";
import { Button } from "./Button";
import { TRANSITION_FAST } from "../lib/motion";
import { formatInterval } from "../lib/format";
import type { Intervals, Rating, ReviewCounts } from "../types/Review";

// Quiet until hovered: the word on the card leads, the buttons wait. Each class is written out
// in full so Tailwind can find it.
const RATINGS: { rating: Rating; label: string; key: keyof Intervals; tone: string }[] = [
    { rating: 1, label: "Again", key: "again", tone: "border-rating-again/60 bg-rating-again/10 hover:bg-rating-again" },
    { rating: 2, label: "Hard", key: "hard", tone: "border-rating-hard/60 bg-rating-hard/10 hover:bg-rating-hard" },
    { rating: 3, label: "Good", key: "good", tone: "border-rating-good/60 bg-rating-good/10 hover:bg-rating-good" },
    { rating: 4, label: "Easy", key: "easy", tone: "border-rating-easy/60 bg-rating-easy/10 hover:bg-rating-easy" },
];

export interface RatingBarProps {
    counts: ReviewCounts;
    /** What each answer would schedule, shown under its label. */
    intervals: Intervals | null;
    revealed: boolean;
    busy: boolean;
    onShowResponse: () => void;
    onAnswer: (rating: Rating) => void;
}

/**
 * The bar at the bottom of a review. Before the response is shown it has one button for it. After,
 * it has the four answers, each with its shortcut and how long the card would wait. The counts of
 * what is still waiting are on the left and stay neutral, so the colors belong to the answers.
 */
export function RatingBar({ counts, intervals, revealed, busy, onShowResponse, onAnswer }: RatingBarProps) {
    return (
        <div className="sticky bottom-16 z-10 -mx-4 border-t border-line bg-canvas/90 px-4 py-3 backdrop-blur md:-mx-8 md:bottom-0 md:px-8">
            <div className="grid gap-3 md:grid-cols-[1fr_auto_1fr] md:items-center">
                <p
                    className="text-center text-meta tabular-nums text-muted md:text-left"
                    title="New: not reviewed yet. Learning: missed and due again. Review: due to review."
                >
                    New <b className="font-semibold text-ink">{counts.new}</b>
                    <span className="mx-2 text-subtle" aria-hidden>
                        ·
                    </span>
                    Learning <b className="font-semibold text-ink">{counts.learning}</b>
                    <span className="mx-2 text-subtle" aria-hidden>
                        ·
                    </span>
                    Review <b className="font-semibold text-ink">{counts.review}</b>
                </p>

                <AnimatePresence mode="wait" initial={false}>
                    {revealed ? (
                        <motion.div
                            key="answers"
                            className="grid grid-cols-4 gap-2"
                            initial={{ opacity: 0, y: 6 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0 }}
                            transition={TRANSITION_FAST}
                        >
                            {RATINGS.map(({ rating, label, key, tone }) => (
                                <Button
                                    key={rating}
                                    variant="plain"
                                    onClick={() => onAnswer(rating)}
                                    disabled={busy}
                                    aria-keyshortcuts={String(rating)}
                                    className={`group min-h-14 flex-col gap-0.5 border px-2 text-ink hover:text-white md:min-w-24 ${tone}`}
                                >
                                    <span>
                                        <span className="mr-1.5 font-normal text-muted group-hover:text-white/80" aria-hidden>
                                            {rating}
                                        </span>
                                        {label}
                                    </span>
                                    {intervals && (
                                        <span className="text-label font-normal text-muted group-hover:text-white/80">
                                            {formatInterval(intervals[key])}
                                        </span>
                                    )}
                                </Button>
                            ))}
                        </motion.div>
                    ) : (
                        <motion.div
                            key="show"
                            className="flex justify-center"
                            initial={{ opacity: 0 }}
                            animate={{ opacity: 1 }}
                            exit={{ opacity: 0 }}
                            transition={TRANSITION_FAST}
                        >
                            <Button onClick={onShowResponse} aria-keyshortcuts="Space" className="min-h-12 w-full px-8 md:w-auto">
                                Show response
                                <kbd className="hidden rounded border border-white/40 px-1.5 py-0.5 font-sans text-label font-normal md:inline">
                                    Space
                                </kbd>
                            </Button>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    );
}
