import { useId } from "react";
import { useTranslation } from "react-i18next";
import { controlClass, labelClass } from "./formStyles";
import { MAX_PER_DAY } from "../lib/deckSettings";
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
    const { t } = useTranslation();
    const id = useId();
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
