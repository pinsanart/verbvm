import { useId } from "react";
import { Trans, useTranslation } from "react-i18next";
import type { CardState, ReviewCounts } from "../types/Review";

export interface ReviewCountsInlineProps {
    counts: ReviewCounts;
    /** The state of the card on screen, so its count is the one that stands out. */
    current: CardState | null;
}

/**
 * What is still waiting in the session, on one line for the top bar. The count of the card on
 * screen stands out, since that is the one answering it brings down. The counts are neutral so
 * red and green stay with the answers. What they mean appears on hover and on keyboard focus,
 * and is also the group's description for screen readers.
 */
export function ReviewCountsInline({ counts, current }: ReviewCountsInlineProps) {
    const { t } = useTranslation();
    const legendId = useId();
    const items = [
        { state: "new", label: t("review.new"), value: counts.new },
        { state: "learning", label: t("review.learning"), value: counts.learning },
        { state: "review", label: t("review.reviewCount"), value: counts.review },
    ] as const;

    return (
        <div
            role="group"
            tabIndex={0}
            aria-label={t("review.waitingLabel")}
            aria-describedby={legendId}
            className="group relative rounded-lg outline-offset-2"
        >
            <dl className="flex items-baseline gap-1 text-meta tabular-nums text-muted">
                {items.map(({ state, label, value }) => (
                    <div
                        key={state}
                        aria-current={current === state ? "true" : undefined}
                        className={`flex items-baseline gap-1.5 rounded-md px-2 py-0.5 ${current === state ? "bg-white/10 text-ink" : ""}`}
                    >
                        <dt>{label}</dt>
                        <dd className="font-semibold text-ink">{value}</dd>
                    </div>
                ))}
            </dl>
            <p
                id={legendId}
                role="tooltip"
                className="absolute right-0 top-full z-30 mt-2 hidden w-72 rounded-lg border border-line-strong bg-raised p-3 text-ui text-muted shadow-lg shadow-black/30 group-focus-within:block group-hover:block"
            >
                <Trans i18nKey="review.legend" components={{ b: <b className="font-medium text-ink" /> }} />
            </p>
        </div>
    );
}
