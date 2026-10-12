import i18n from "../i18n";

/** Seconds until the next UTC midnight, when a deck's daily limits start over. */
export function secondsUntilNextUtcDay(now: Date = new Date()): number {
    const next = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1);
    return Math.max(0, Math.round((next - now.getTime()) / 1000));
}

/** A number with one decimal at most, as the days and months of an interval. */
const rounded = (value: number) => Math.round(value * 10) / 10;

/**
 * A short duration for a button or a sentence in the interface language: "<1 min", "10 min",
 * "3 h", "1 day", "1.3 days", "6 days", "2 months".
 */
export function formatInterval(seconds: number): string {
    const t = i18n.t.bind(i18n);
    if (seconds < 60) return t("interval.lessThanMinute");
    const minutes = seconds / 60;
    if (minutes < 60) return t("interval.minutes", { count: Math.round(minutes) });
    const hours = minutes / 60;
    if (hours < 24) return t("interval.hours", { count: Math.round(hours) });
    const days = hours / 24;
    if (days < 30) return t("interval.days", { count: rounded(days) });
    return t("interval.months", { count: rounded(days / 30) });
}
