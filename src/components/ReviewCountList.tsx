import type { ReviewCounts } from "../types/Review";

const ROWS = [
    { key: "new", label: "New cards", tone: "text-accent-ink" },
    { key: "learning", label: "Learning cards", tone: "text-stack-ink" },
    { key: "review", label: "Review cards", tone: "text-orthographic" },
] as const;

/** The three counts of a type, each with its label and its color. */
export function ReviewCountList({ counts }: { counts: ReviewCounts }) {
    return (
        <dl className="flex flex-col gap-1.5 text-ui">
            {ROWS.map(({ key, label, tone }) => (
                <div key={key} className="flex items-baseline justify-between gap-4">
                    <dt className="text-muted">{label}</dt>
                    <dd className={`font-semibold tabular-nums ${tone}`}>{counts[key]}</dd>
                </div>
            ))}
        </dl>
    );
}
