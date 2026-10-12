import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Check, CircleAlert } from "lucide-react";
import { ReviewCardFace } from "./ReviewCardFace";
import { controlClass } from "./formStyles";
import { replayAudio } from "../lib/audio";
import { diffText } from "../lib/textDiff";
import type { DiffKind, DiffSegment, TextDiff } from "../lib/textDiff";
import type { ReviewField } from "../types/Review";

// A color is never the only signal: wrong letters are struck through and missing ones underlined.
const KIND_CLASS: Record<DiffKind, string> = {
    equal: "bg-rating-good/30",
    wrong: "bg-danger/40 line-through",
    missing: "bg-white/15 text-muted underline decoration-dotted underline-offset-4",
};

function Segments({ segments }: { segments: DiffSegment[] }) {
    return (
        <p className="font-mono text-heading whitespace-pre-wrap break-words">
            {segments.map((segment, index) => (
                <span key={index} className={`rounded-sm ${KIND_CLASS[segment.kind]}`}>
                    {segment.text}
                </span>
            ))}
        </p>
    );
}

export interface DictationReviewProps {
    /** The fields of the front: the audio to write down. */
    front: ReviewField[];
    /** The text that was expected, from the back of the card. */
    expected: string;
    typed: string;
    revealed: boolean;
    onTypedChange: (value: string) => void;
    /** Called when the learner submits the answer with Enter. */
    onSubmit: () => void;
    /** The audio plays by itself when the card appears. */
    autoPlay?: boolean;
}

/**
 * The review of an orthographic card: the audio, a box to write what was heard and, once the
 * response is shown, what was written beside what was expected, letter by letter.
 */
export function DictationReview({
    front,
    expected,
    typed,
    revealed,
    onTypedChange,
    onSubmit,
    autoPlay,
}: DictationReviewProps) {
    const { t } = useTranslation();
    const input = useRef<HTMLInputElement>(null);

    // The learner starts typing right away. Writing does not need a click first.
    useEffect(() => {
        if (!revealed) input.current?.focus({ preventScroll: true });
    }, [revealed]);

    const diff = revealed ? diffText(typed, expected) : null;

    /** One sentence for how the answer compares, so the learner does not have to compare strings. */
    const verdict = (result: TextDiff): string => {
        if (result.exact) return t("review.exactMatch");
        if (typed.trim() === "") return t("review.nothingTyped");
        if (result.extra > 0 && result.missing > 0) {
            return t("review.diffBoth", { extra: result.extra, missing: result.missing });
        }
        return result.extra > 0
            ? t("review.diffExtra", { count: result.extra })
            : t("review.diffMissing", { count: result.missing });
    };

    return (
        <div className="flex flex-col gap-8">
            <ReviewCardFace fields={front} size="front" autoPlay={autoPlay} />

            {diff ? (
                <div aria-live="polite" className="flex flex-col gap-5 border-t border-line-strong pt-8">
                    <p className="flex items-center gap-2 text-body font-medium text-ink">
                        {diff.exact ? <Check size={18} aria-hidden /> : <CircleAlert size={18} aria-hidden />}
                        {verdict(diff)}
                    </p>
                    {/* The two lines share a label column, so each letter can be compared with the one
                        above it. */}
                    <dl className="grid grid-cols-[auto_1fr] items-baseline gap-x-4 gap-y-3">
                        <dt className="text-meta text-muted">{t("review.yourAnswer")}</dt>
                        <dd>
                            {diff.typed.length > 0 ? (
                                <Segments segments={diff.typed} />
                            ) : (
                                <span className="text-body text-muted">{t("review.nothingTyped")}</span>
                            )}
                        </dd>
                        <dt className="text-meta text-muted">{t("review.expectedAnswer")}</dt>
                        <dd>
                            <Segments segments={diff.expected} />
                        </dd>
                    </dl>
                </div>
            ) : (
                <div className="mx-auto flex w-full max-w-md flex-col gap-2">
                    <label htmlFor="dictation-answer" className="sr-only">
                        {t("review.yourAnswer")}
                    </label>
                    <input
                        id="dictation-answer"
                        ref={input}
                        type="text"
                        value={typed}
                        onChange={(e) => onTypedChange(e.target.value)}
                        onKeyDown={(e) => {
                            if (e.key === "Enter" && !e.nativeEvent.isComposing) {
                                e.preventDefault();
                                onSubmit();
                            } else if (e.altKey && e.key.toLowerCase() === "r") {
                                // R types a letter in this box, so replaying has its own chord.
                                e.preventDefault();
                                replayAudio();
                            }
                        }}
                        placeholder={t("review.typeWhatYouHear")}
                        aria-describedby="dictation-hint"
                        autoComplete="off"
                        autoCorrect="off"
                        autoCapitalize="off"
                        spellCheck={false}
                        className={`${controlClass} text-center text-lead`}
                    />
                    <p id="dictation-hint" className="hidden text-center text-meta text-muted md:block">
                        {t("review.dictationHint")}
                    </p>
                </div>
            )}
        </div>
    );
}
