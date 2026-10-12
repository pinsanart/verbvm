import { AnimatePresence, motion } from "motion/react";
import { ActionsMenu } from "./ActionsMenu";
import { Button } from "./Button";
import { TRANSITION_FAST } from "../lib/motion";
import type { Rating, ReviewCounts } from "../types/Review";

const RATINGS: { rating: Rating; label: string; fill: string }[] = [
    { rating: 1, label: "Again", fill: "bg-rating-again" },
    { rating: 2, label: "Hard", fill: "bg-rating-hard" },
    { rating: 3, label: "Good", fill: "bg-rating-good" },
    { rating: 4, label: "Easy", fill: "bg-rating-easy" },
];

export interface RatingBarProps {
    counts: ReviewCounts;
    revealed: boolean;
    busy: boolean;
    onShowResponse: () => void;
    onAnswer: (rating: Rating) => void;
}

/**
 * The bar at the bottom of a review. Before the response is shown it has one button for it, and
 * after, the four answers, each with its keyboard shortcut. The counts of what is still waiting
 * are on the left and the Actions menu on the right.
 */
export function RatingBar({ counts, revealed, busy, onShowResponse, onAnswer }: RatingBarProps) {
    return (
        <div className="sticky bottom-16 z-10 -mx-4 border-t border-line bg-canvas/90 px-4 py-3 backdrop-blur md:-mx-8 md:bottom-0 md:px-8">
            <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-3 md:flex-nowrap">
                <p className="order-1 flex gap-3 text-meta tabular-nums text-muted" aria-label="Waiting in this type">
                    <span className="text-accent-ink">New {counts.new}</span>
                    <span className="text-stack-ink">Learning {counts.learning}</span>
                    <span className="text-orthographic">Review {counts.review}</span>
                </p>

                <div className="order-3 flex w-full justify-center md:order-2 md:w-auto">
                    <AnimatePresence mode="wait" initial={false}>
                        {revealed ? (
                            <motion.div
                                key="answers"
                                className="grid w-full grid-cols-4 gap-2 md:w-auto"
                                initial={{ opacity: 0, y: 6 }}
                                animate={{ opacity: 1, y: 0 }}
                                exit={{ opacity: 0 }}
                                transition={TRANSITION_FAST}
                            >
                                {RATINGS.map(({ rating, label, fill }) => (
                                    <Button
                                        key={rating}
                                        variant="plain"
                                        onClick={() => onAnswer(rating)}
                                        disabled={busy}
                                        aria-keyshortcuts={String(rating)}
                                        className={`min-h-11 flex-col gap-0 px-2 text-white hover:brightness-110 md:min-w-24 ${fill}`}
                                    >
                                        <span>{label}</span>
                                        <span className="text-label font-normal opacity-80" aria-hidden>
                                            {rating}
                                        </span>
                                    </Button>
                                ))}
                            </motion.div>
                        ) : (
                            <motion.div
                                key="show"
                                className="w-full md:w-auto"
                                initial={{ opacity: 0 }}
                                animate={{ opacity: 1 }}
                                exit={{ opacity: 0 }}
                                transition={TRANSITION_FAST}
                            >
                                <Button
                                    onClick={onShowResponse}
                                    aria-keyshortcuts="Space"
                                    className="min-h-11 w-full px-8 md:w-auto"
                                >
                                    Show response
                                </Button>
                            </motion.div>
                        )}
                    </AnimatePresence>
                </div>

                <div className="order-2 md:order-3">
                    <ActionsMenu />
                </div>
            </div>
        </div>
    );
}
