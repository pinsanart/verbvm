import { useEffect } from "react";
import { AnimatePresence, motion } from "motion/react";
import { PartyPopper } from "lucide-react";
import { Link, useParams } from "react-router";
import { buttonClass } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { RatingBar } from "../components/RatingBar";
import { ReviewCardFace } from "../components/ReviewCardFace";
import { TRANSITION_BASE, TRANSITION_FAST } from "../lib/motion";
import { styleForType } from "../data/FlashcardTypeStyles";
import type { Rating } from "../types/Review";
import { useReviewSessionViewModel } from "../viewmodels/useReviewSessionViewModel";

/** Keys that press the focused control themselves, so the shortcuts must leave them alone. */
const isTypingOrPressing = (target: EventTarget | null) =>
  target instanceof HTMLElement && target.closest("input, textarea, select, button, a, [role=menuitem]") !== null;

/** Reviews the flashcards of one type in a deck, one card at a time. */
export default function ReviewSession() {
  const { deckName = "", typeSlug = "" } = useParams();
  const vm = useReviewSessionViewModel(deckName, typeSlug);
  const title = vm.card?.typeName ?? styleForType(typeSlug).label;
  const front = vm.card?.fields.filter((field) => field.side === "FRONT") ?? [];
  const back = vm.card?.fields.filter((field) => field.side === "BACK") ?? [];

  // Space or Enter shows the response and 1 to 4 answer, so a review can run without the mouse.
  const { showResponse, answer, revealed, card } = vm;
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (!card || e.metaKey || e.ctrlKey || e.altKey) return;
      if (!revealed) {
        if ((e.key === " " || e.key === "Enter") && !isTypingOrPressing(e.target)) {
          e.preventDefault();
          showResponse();
        }
      } else if (["1", "2", "3", "4"].includes(e.key) && !isTypingOrPressing(e.target)) {
        e.preventDefault();
        answer(Number(e.key) as Rating);
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [card, revealed, showResponse, answer]);

  return (
    <Page title={title} parents={[{ to: "/review", label: "Review" }]} meta={deckName}>
      <AnimatePresence initial={false}>
        {vm.error && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="h-64 animate-pulse rounded-2xl border border-line bg-panel" aria-busy="true" aria-label="Loading the card" />
      )}

      {vm.done && (
        <EmptyState
          icon={PartyPopper}
          title={vm.answered > 0 ? "All done for now" : `Nothing to review in ${title}`}
          description={
            vm.answered > 0
              ? `You answered ${vm.answered} ${vm.answered === 1 ? "card" : "cards"}. Cards you missed come back in a few minutes.`
              : "Everything here is up to date. Cards come back when they are due."
          }
          action={
            <Link to="/review" className={buttonClass("primary")}>
              Back to Review
            </Link>
          }
        />
      )}

      {vm.card && (
        <>
          <AnimatePresence mode="wait" initial={false}>
            <motion.article
              key={vm.card.uuid}
              aria-label="Flashcard"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={TRANSITION_FAST}
              className="mx-auto flex w-full max-w-2xl flex-col gap-6 rounded-2xl border border-line bg-panel p-6 md:p-10"
            >
              <ReviewCardFace fields={front} />

              <AnimatePresence initial={false}>
                {vm.revealed && back.length > 0 && (
                  <motion.div
                    key="back"
                    aria-live="polite"
                    initial={{ opacity: 0, y: 8 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0 }}
                    transition={TRANSITION_BASE}
                    className="border-t border-line-strong pt-6"
                  >
                    <ReviewCardFace fields={back} />
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.article>
          </AnimatePresence>

          <RatingBar
            counts={vm.counts}
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
