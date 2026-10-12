import type { ReviewCounts } from "../types/Review";

const ROWS = [
    { key: "new", label: "New" },
    { key: "learning", label: "Learning" },
    { key: "review", label: "Review" },
] as const;

/**
 * The three counts of a type. They are neutral on purpose: red and green are the colors of the
 * answers Again and Good, and of errors and success, so a count does not borrow them.
 */
export function ReviewCountList({ counts }: { counts: ReviewCounts }) {
    return (
        <dl className="flex flex-col gap-1.5 text-ui">
            {ROWS.map(({ key, label }) => (
                <div key={key} className="flex items-baseline justify-between gap-4">
                    <dt className="text-muted">{label}</dt>
                    <dd className="font-semibold tabular-nums text-ink">{counts[key]}</dd>
                </div>
            ))}
        </dl>
    );
}
