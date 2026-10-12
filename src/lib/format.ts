/** A number with one decimal at most, without a trailing ".0". */
const compact = (value: number) => String(Math.round(value * 10) / 10);

const plural = (value: number, unit: string) => `${compact(value)} ${unit}${value === 1 ? "" : "s"}`;

/**
 * A short duration for a button or a sentence: "<1 min", "10 min", "3 h", "1 day", "1.3 days",
 * "6 days", "2 months".
 */
export function formatInterval(seconds: number): string {
    if (seconds < 60) return "<1 min";
    const minutes = seconds / 60;
    if (minutes < 60) return `${Math.round(minutes)} min`;
    const hours = minutes / 60;
    if (hours < 24) return `${Math.round(hours)} h`;
    const days = hours / 24;
    if (days < 30) return plural(Math.round(days * 10) / 10, "day");
    return plural(Math.round((days / 30) * 10) / 10, "month");
}
