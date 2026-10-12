import { useTranslation } from "react-i18next";
import type { ParseKeys } from "i18next";
import type { ReviewCounts } from "../types/Review";

const ROWS = [
    { key: "new", labelKey: "review.new" },
    { key: "learning", labelKey: "review.learning" },
    { key: "review", labelKey: "review.reviewCount" },
] as const satisfies readonly { key: keyof ReviewCounts; labelKey: ParseKeys }[];

/**
 * The three counts of a type. They are neutral on purpose: red and green are the colors of the
 * answers Again and Good, and of errors and success, so a count does not borrow them.
 */
export function ReviewCountList({ counts }: { counts: ReviewCounts }) {
    const { t } = useTranslation();
    return (
        <dl className="flex flex-col gap-1.5 text-ui">
            {ROWS.map(({ key, labelKey }) => (
                <div key={key} className="flex items-baseline justify-between gap-4">
                    <dt className="text-muted">{t(labelKey)}</dt>
                    <dd className="font-semibold tabular-nums text-ink">{counts[key]}</dd>
                </div>
            ))}
        </dl>
    );
}
