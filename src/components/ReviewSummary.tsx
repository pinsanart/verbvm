import { Link } from "react-router";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { Check } from "lucide-react";
import { buttonClass } from "./Button";
import { TRANSITION_BASE } from "../lib/motion";
import { formatInterval } from "../lib/format";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { ReviewType } from "../types/Review";

export interface ReviewSummaryProps {
    typeName: string;
    deckName: string;
    answered: number;
    /** How many of the answers were Again. */
    again: number;
    /** Seconds until the next card of this type is due, if one is scheduled. */
    nextDueInSeconds: number | null;
    /** The other types of the deck that still have cards waiting. */
    siblings: ReviewType[];
}

/** What a session leaves you with: what you did, when the next card comes back, and what is next. */
export function ReviewSummary({ typeName, deckName, answered, again, nextDueInSeconds, siblings }: ReviewSummaryProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const stats = [
        { label: t("review.answered"), value: answered },
        { label: t("review.remembered"), value: answered - again },
        { label: t("review.missed"), value: again },
    ];

    return (
        <motion.section
            aria-labelledby="review-summary-title"
            initial={{ opacity: 0, scale: 0.98 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={TRANSITION_BASE}
            className="mx-auto flex w-full max-w-2xl flex-col items-center gap-6 rounded-2xl border border-line bg-panel p-8 text-center md:p-12"
        >
            <span className="flex size-14 items-center justify-center rounded-full bg-orthographic/15 text-orthographic">
                <Check size={28} aria-hidden />
            </span>

            <div className="flex flex-col gap-2">
                <h2 id="review-summary-title" className="text-title">
                    {answered > 0 ? t("review.allDone") : t("review.nothingToReview", { type: typeName })}
                </h2>
                <p className="text-ui text-muted">
                    {nextDueInSeconds !== null
                        ? t("review.nextCard", { interval: formatInterval(nextDueInSeconds) })
                        : t("review.noneScheduled")}
                </p>
            </div>

            {answered > 0 && (
                <dl className="grid w-full max-w-sm grid-cols-3 gap-4">
                    {stats.map(({ label, value }) => (
                        <div key={label} className="flex flex-col gap-1">
                            <dd className="text-title font-semibold tabular-nums">{value}</dd>
                            <dt className="text-meta text-muted">{label}</dt>
                        </div>
                    ))}
                </dl>
            )}

            <div className="flex flex-wrap items-center justify-center gap-2">
                {siblings.map((type) => (
                    <Link
                        key={type.slug}
                        to={`/review/${encodeURIComponent(deckName)}/${type.slug}`}
                        className={buttonClass("primary")}
                    >
                        {t("review.reviewType", {
                            type: typeLabel(type.slug, type.name),
                            count: type.counts.new + type.counts.learning + type.counts.review,
                        })}
                    </Link>
                ))}
                <Link to="/review" className={buttonClass(siblings.length > 0 ? "outline" : "primary")}>
                    {t("review.backToReview")}
                </Link>
            </div>
        </motion.section>
    );
}
