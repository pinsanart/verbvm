import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { AnimatePresence, motion } from "motion/react";
import { useNavigate, useParams } from "react-router";
import { Undo2 } from "lucide-react";
import { Button } from "../components/Button";
import { IconButton } from "../components/IconButton";
import { DictationReview } from "../components/DictationReview";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { ReviewCountsInline } from "../components/ReviewCountsInline";
import { RatingBar } from "../components/RatingBar";
import { ReviewCardFace } from "../components/ReviewCardFace";
import { ReviewSummary } from "../components/ReviewSummary";
import { replayAudio } from "../lib/audio";
import { hasModifier, isTyping, isTypingOrPressing } from "../lib/keyboard";
import { TRANSITION_BASE, TRANSITION_FAST } from "../lib/motion";
import { useKeys } from "../lib/useKeys";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { Rating } from "../types/Review";
import { useReviewSessionViewModel } from "../viewmodels/useReviewSessionViewModel";

/** The only keys a review listens to. Any other key is ignored before it reaches this screen. */
// "®" and a non-breaking space are what Alt+R and Alt+Space type on macOS.
const REVIEW_KEYS = ["r", " ", "Enter", "1", "2", "3", "4", "z", "Escape", "®", "\u00a0"] as const;

/** Reviews the flashcards of one type in a deck, one card at a time. */
export default function ReviewSession() {
  const { deckName = "", typeSlug = "" } = useParams();
  const { t } = useTranslation();
  const typeLabel = useTypeLabel();
  const vm = useReviewSessionViewModel(deckName, typeSlug);
  const navigate = useNavigate();
  const title = typeLabel(typeSlug, vm.card?.typeName);
  const front = vm.card?.fields.filter((field) => field.side === "FRONT") ?? [];
  const back = vm.card?.fields.filter((field) => field.side === "BACK") ?? [];
  const cardRef = useRef<HTMLElement>(null);
  // An orthographic card is answered by typing what the audio says.
  const dictation = vm.card?.typeSlug === "orthographic";
  // The audio is what the learner is shown. The text it was spoken from is the answer, kept
  // hidden until the response is shown.
  const audioFields = vm.card?.fields.filter((field) => field.kind === "AUDIO") ?? [];
  const expectedKey = audioFields.find((field) => field.sourceKey !== null)?.sourceKey;
  const expected = vm.card?.fields.find((field) => field.key === expectedKey)?.content ?? "";

  // Space or Enter shows the response, 1 to 4 answer, R plays the audio again, Z takes the last
  // answer back and Escape leaves the session, so a review can run without the mouse. Answers are
  // saved as they are given, so leaving loses nothing. Alt+Space shows the response, Alt+R plays
  // the audio and Ctrl+Z undoes from anywhere, including a text box, where the plain keys write.
  const { showResponse, answer, undo, revealed, card } = vm;
  // Only Space and Enter press a focused button or link themselves, so only they step aside for
  // one. The other keys keep working after a Tab onto the answers.
  useKeys(REVIEW_KEYS, (e) => {
    // Ctrl+Z (Cmd+Z) does what the Undo button does, from anywhere, including the dictation box.
    // With no answer to take back it is left alone, so the box can still undo its own typing.
    if ((e.ctrlKey || e.metaKey) && !e.altKey && !e.shiftKey && e.key.toLowerCase() === "z") {
      if (vm.canUndo) {
        e.preventDefault();
        undo();
      }
      return;
    }
    if (e.altKey && !e.ctrlKey && !e.metaKey) {
      if (!card) return;
      if (e.key === " " || e.key === "\u00a0") {
        e.preventDefault();
        if (!revealed) showResponse();
      } else if (e.key.toLowerCase() === "r" || e.key === "®") {
        e.preventDefault();
        replayAudio();
      }
      return;
    }
    if (e.altKey || isTyping(e.target)) return;
    if (e.key.toLowerCase() === "z" && !hasModifier(e)) {
      e.preventDefault();
      undo();
      return;
    }
    if (!card || hasModifier(e)) return;
    if (e.key === "Escape") {
      navigate("/review");
    } else if (e.key.toLowerCase() === "r") {
      e.preventDefault();
      replayAudio();
    } else if (e.key === " " || e.key === "Enter") {
      if (revealed || isTypingOrPressing(e.target)) return;
      e.preventDefault();
      showResponse();
    } else if (revealed && ["1", "2", "3", "4"].includes(e.key)) {
      e.preventDefault();
      answer(Number(e.key) as Rating);
    }
  });

  // The button that was pressed goes away when the answers replace it, which would drop the focus
  // to the page. The card takes it instead, so the next Tab goes to the first answer.
  useEffect(() => {
    // While a dictation waits for the answer the text box keeps the focus.
    if (card && !(dictation && !revealed)) cardRef.current?.focus({ preventScroll: true });
  }, [card?.uuid, vm.presentation, revealed, dictation]);

  return (
    <Page
      title={title}
      parents={[{ to: "/review", label: t("nav.review") }]}
      meta={deckName}
      actions={
        vm.status === "ready" ? (
          <div className="flex items-center gap-3">
            {/* On a phone the bar has room only for the undo button, so the counts move below it. */}
            {vm.card && (
              <div className="hidden md:block">
                <ReviewCountsInline counts={vm.counts} current={vm.card.state} />
              </div>
            )}
            <IconButton
              label={t("review.undoAria")}
              icon={Undo2}
              iconSize={18}
              onClick={() => undo()}
              disabled={!vm.canUndo}
            />
          </div>
        ) : undefined
      }
    >
      {vm.card && (
        <div className="flex justify-center md:hidden">
          <ReviewCountsInline counts={vm.counts} current={vm.card.state} />
        </div>
      )}

      <AnimatePresence initial={false}>
        {vm.error && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div
          className="mx-auto min-h-[18rem] w-full max-w-3xl animate-pulse rounded-2xl border border-line bg-panel"
          aria-busy="true"
          aria-label={t("review.loadingCard")}
        />
      )}

      {vm.done && (
        <ReviewSummary
          typeName={title}
          typeSlug={typeSlug}
          deckName={deckName}
          answered={vm.answered}
          again={vm.again}
          cardsReviewed={vm.cardsReviewed}
          nextDueInSeconds={vm.nextDueInSeconds}
          dailyLimitReached={vm.dailyLimitReached}
          siblings={vm.siblings}
        />
      )}

      {vm.card && (
        <>
          <AnimatePresence mode="wait" initial={false}>
            <motion.article
              key={`${vm.card.uuid}:${vm.presentation}`}
              ref={cardRef}
              tabIndex={-1}
              lang={vm.languageTag || undefined}
              aria-label={t("review.cardAria", {
                type: title,
                state: t(vm.card.state === "new" ? "review.new" : vm.card.state === "learning" ? "review.learning" : "review.reviewCount"),
              })}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={TRANSITION_FAST}
              className="mx-auto flex min-h-[18rem] w-full max-w-3xl flex-col justify-center gap-8 rounded-2xl border border-line bg-panel p-8 outline-none md:p-14"
            >
              {dictation ? (
                <DictationReview
                  front={audioFields}
                  expected={expected}
                  typed={vm.typed}
                  revealed={vm.revealed}
                  onTypedChange={vm.setTyped}
                  onSubmit={vm.showResponse}
                  autoPlay={vm.autoplayAudio}
                />
              ) : (
                <ReviewCardFace fields={front} size="front" autoPlay={vm.autoplayAudio} />
              )}

              <AnimatePresence initial={false}>
                {vm.revealed && back.length > 0 && !dictation && (
                  <motion.div
                    key="back"
                    aria-live="polite"
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    transition={TRANSITION_BASE}
                    className="border-t border-line-strong pt-8"
                  >
                    <ReviewCardFace fields={back} size="back" autoPlay={vm.autoplayAudio} />
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.article>
          </AnimatePresence>

          {vm.needsReload ? (
            <div className="sticky bottom-0 z-10 -mx-4 flex flex-wrap items-center justify-center gap-3 border-t border-line bg-canvas px-4 py-3 md:-mx-8 md:px-8">
              <p className="text-ui text-muted">{t("review.reloadFailed")}</p>
              <Button onClick={() => vm.reload()} loading={vm.busy}>
                {t("review.retry")}
              </Button>
            </div>
          ) : (
            <RatingBar
              intervals={vm.intervals}
              revealed={vm.revealed}
              busy={vm.busy}
              onShowResponse={vm.showResponse}
              onAnswer={vm.answer}
            />
          )}
        </>
      )}
    </Page>
  );
}
