import { useId } from "react";
import { useTranslation } from "react-i18next";
import { controlClass, labelClass } from "./formStyles";
import { MAX_PER_DAY, localTimeOfUtc } from "../lib/deckSettings";
import type { DeckSettingsDraft } from "../lib/deckSettings";

export interface DeckSettingsFieldsProps {
    /** What the options are for: the name of the flashcard type. */
    legend: string;
    value: DeckSettingsDraft;
    onChange: (value: DeckSettingsDraft) => void;
}

/**
 * The study options of one flashcard type of a deck. The daily limits count per UTC day. The audio option is worded as an opt-out, since playing
 * automatically is the default.
 */
export function DeckSettingsFields({ legend, value, onChange }: DeckSettingsFieldsProps) {
    const { t, i18n } = useTranslation();
    const id = useId();
    const localTime = localTimeOfUtc(value.dayStartsAt, i18n.language);
    const number = (key: "newCardsPerDay" | "maxReviewsPerDay", label: string, hint: string) => (
        <div>
            <label className={labelClass} htmlFor={`${id}-${key}`}>
                {label}
            </label>
            <input
                id={`${id}-${key}`}
                type="number"
                inputMode="numeric"
                min={0}
                max={MAX_PER_DAY}
                step={1}
                required
                className={controlClass}
                value={value[key]}
                onChange={(e) => onChange({ ...value, [key]: e.target.value })}
                aria-describedby={`${id}-${key}-hint`}
            />
            <p id={`${id}-${key}-hint`} className="mt-1.5 text-meta text-muted">
                {hint}
            </p>
        </div>
    );

    return (
        <fieldset className="flex flex-col gap-4 rounded-xl border border-line p-4">
            <legend className="px-1 text-ui font-medium text-muted">{legend}</legend>
            <div className="grid gap-4 sm:grid-cols-2">
                {number("newCardsPerDay", t("decks.newCardsPerDay"), t("decks.newCardsPerDayHint"))}
                {number("maxReviewsPerDay", t("decks.maxReviewsPerDay"), t("decks.maxReviewsPerDayHint"))}
            </div>
            <div>
                <label className={labelClass} htmlFor={`${id}-dayStartsAt`}>
                    {t("decks.dayStartsAt")}
                </label>
                <input
                    id={`${id}-dayStartsAt`}
                    type="time"
                    required
                    className={`${controlClass} sm:max-w-40`}
                    value={value.dayStartsAt}
                    onChange={(e) => onChange({ ...value, dayStartsAt: e.target.value })}
                    aria-describedby={`${id}-dayStartsAt-hint`}
                />
                <p id={`${id}-dayStartsAt-hint`} className="mt-1.5 text-meta text-muted">
                    {localTime ? t("decks.dayStartsAtLocal", { local: localTime }) + " " : ""}
                    {t("decks.dayStartsAtHint")}
                </p>
            </div>
            <label className="flex items-center gap-2.5 text-ui text-ink">
                <input
                    type="checkbox"
                    className="size-4"
                    checked={!value.autoplayAudio}
                    onChange={(e) => onChange({ ...value, autoplayAudio: !e.target.checked })}
                />
                {t("decks.noAutoplay")}
            </label>
        </fieldset>
    );
}
