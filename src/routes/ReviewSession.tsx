import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { AnimatePresence, motion } from "motion/react";
import { useParams } from "react-router";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { RatingBar } from "../components/RatingBar";
import { ReviewCardFace } from "../components/ReviewCardFace";
import { ReviewSummary } from "../components/ReviewSummary";
import { TRANSITION_BASE, TRANSITION_FAST } from "../lib/motion";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { Rating } from "../types/Review";
import { useReviewSessionViewModel } from "../viewmodels/useReviewSessionViewModel";

/** Keys that press the focused control themselves, so the shortcuts must leave them alone. */
const isTypingOrPressing = (target: EventTarget | null) =>
  target instanceof HTMLElement && target.closest("input, textarea, select, button, a") !== null;

/** Plays the first audio of the card from the start. */
const replayAudio = () => {
  const audio = document.querySelector<HTMLAudioElement>("audio[data-review-audio]");
  if (!audio) return;
  audio.currentTime = 0;
  audio.play().catch(() => undefined);
};

/** Reviews the flashcards of one type in a deck, one card at a time. */
export default function ReviewSession() {
  const { deckName = "", typeSlug = "" } = useParams();
  const { t } = useTranslation();
  const typeLabel = useTypeLabel();
  const vm = useReviewSessionViewModel(deckName, typeSlug);
  const title = typeLabel(typeSlug, vm.card?.typeName);
  const front = vm.card?.fields.filter((field) => field.side === "FRONT") ?? [];
  const back = vm.card?.fields.filter((field) => field.side === "BACK") ?? [];
  const cardRef = useRef<HTMLElement>(null);

  // Space or Enter shows the response, 1 to 4 answer and R plays the audio again, so a review can
  // run without the mouse.
  const { showResponse, answer, revealed, card } = vm;
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!card || e.metaKey || e.ctrlKey || e.altKey || isTypingOrPressing(e.target)) return;
      if (e.key.toLowerCase() === "r") {
        e.preventDefault();
        replayAudio();
      } else if (!revealed && (e.key === " " || e.key === "Enter")) {
        e.preventDefault();
        showResponse();
      } else if (revealed && ["1", "2", "3", "4"].includes(e.key)) {
        e.preventDefault();
        answer(Number(e.key) as Rating);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [card, revealed, showResponse, answer]);

  // The button that was pressed goes away when the answers replace it, which would drop the focus
  // to the page. The card takes it instead, so the next Tab goes to the first answer.
  useEffect(() => {
    if (card) cardRef.current?.focus({ preventScroll: true });
  }, [card?.uuid, revealed]);

  return (
    <Page title={title} parents={[{ to: "/review", label: t("nav.review") }]} meta={deckName}>
      <AnimatePresence initial={false}>
        {vm.error && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="h-64 animate-pulse rounded-2xl border border-line bg-panel" aria-busy="true" aria-label={t("review.loadingCard")} />
      )}

      {vm.done && (
        <ReviewSummary
          typeName={title}
          deckName={deckName}
          answered={vm.answered}
          again={vm.again}
          nextDueInSeconds={vm.nextDueInSeconds}
          siblings={vm.siblings}
        />
      )}

      {vm.card && (
        <>
          <AnimatePresence mode="wait" initial={false}>
            <motion.article
              key={vm.card.uuid}
              ref={cardRef}
              tabIndex={-1}
              lang={vm.languageTag || undefined}
              aria-label={t("review.flashcardAria")}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={TRANSITION_FAST}
              className="mx-auto flex min-h-[18rem] w-full max-w-3xl flex-col justify-center gap-8 rounded-2xl border border-line bg-panel p-8 outline-none md:p-14"
            >
              <ReviewCardFace fields={front} size="front" />

              <AnimatePresence initial={false}>
                {vm.revealed && back.length > 0 && (
                  <motion.div
                    key="back"
                    aria-live="polite"
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    transition={TRANSITION_BASE}
                    className="border-t border-line-strong pt-8"
                  >
                    <ReviewCardFace fields={back} size="back" />
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.article>
          </AnimatePresence>

          <RatingBar
            counts={vm.counts}
            intervals={vm.intervals}
            revealed={vm.revealed}
            busy={vm.busy}
            onShowResponse={vm.showResponse}
            onAnswer={vm.answer}
          />
        </>
      )}
    </Page>
  );
}
