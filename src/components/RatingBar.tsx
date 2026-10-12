import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Button } from "./Button";
import { TRANSITION_FAST } from "../lib/motion";
import { formatInterval } from "../lib/format";
import type { ParseKeys } from "i18next";
import type { Intervals, Rating } from "../types/Review";

// Solid fills in the color of each answer, with full white text (white at 80% falls under 4.5:1 on
// Again, Hard and Good). Each class is written out in full so
// Tailwind can find it.
const RATINGS: { rating: Rating; labelKey: ParseKeys; key: keyof Intervals; tone: string }[] = [
    { rating: 1, labelKey: "review.again", key: "again", tone: "bg-rating-again" },
    { rating: 2, labelKey: "review.hard", key: "hard", tone: "bg-rating-hard" },
    { rating: 3, labelKey: "review.good", key: "good", tone: "bg-rating-good" },
    { rating: 4, labelKey: "review.easy", key: "easy", tone: "bg-rating-easy" },
];

export interface RatingBarProps {
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
 * what is still waiting are in the top bar of the screen.
 */
export function RatingBar({ intervals, revealed, busy, onShowResponse, onAnswer }: RatingBarProps) {
    const { t } = useTranslation();
    return (
        <div className="sticky bottom-0 z-10 -mx-4 border-t border-line bg-canvas/90 px-4 py-3 backdrop-blur md:-mx-8 md:px-8">
            <div className="flex justify-center">
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
                            {RATINGS.map(({ rating, labelKey, key, tone }) => (
                                <Button
                                    key={rating}
                                    variant="plain"
                                    onClick={() => onAnswer(rating)}
                                    disabled={busy}
                                    aria-keyshortcuts={String(rating)}
                                    className={`min-h-14 flex-col gap-0.5 px-2 text-white hover:brightness-110 md:min-w-28 ${tone}`}
                                >
                                    <span>
                                        <span className="mr-1.5 font-normal text-white" aria-hidden>
                                            {rating}
                                        </span>
                                        {t(labelKey)}
                                    </span>
                                    {intervals && (
                                        <span className="text-label font-normal text-white">
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
                                {t("review.showResponse")}
                                <kbd className="hidden rounded border border-white/40 px-1.5 py-0.5 font-sans text-label font-normal md:inline">
                                    {t("review.space")}
                                </kbd>
                            </Button>
                        </motion.div>
                    )}
                </AnimatePresence>
            </div>
        </div>
    );
}
