import { BookOpen, Layers } from "lucide-react";
import { Link } from "react-router";
import { AnimatePresence } from "motion/react";
import { buttonClass } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { ReviewDeckRow } from "../components/ReviewDeckRow";
import { useReviewOverviewViewModel } from "../viewmodels/useReviewOverviewViewModel";

/** The decks you can review, with the types each has and what is waiting in them. */
export default function Review() {
  const vm = useReviewOverviewViewModel();
  const hasCards = vm.decks.some((deck) => deck.types.length > 0);

  return (
    <Page title="Review" width="wide">
      <AnimatePresence initial={false}>
        {vm.error && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="flex flex-col gap-4" aria-busy="true" aria-label="Loading decks">
          {[0, 1].map((n) => (
            <div key={n} className="h-44 animate-pulse rounded-2xl border border-line bg-panel" />
          ))}
        </div>
      )}

      {vm.status === "ready" && vm.decks.length === 0 && (
        <EmptyState
          icon={Layers}
          title="No decks to review yet"
          description="Reviews are built from the flashcards in your decks. Create a deck and add some cards first."
          action={
            <Link to="/decks" className={buttonClass("primary")}>
              Go to Decks
            </Link>
          }
        />
      )}

      {vm.status === "ready" && vm.decks.length > 0 && !hasCards && (
        <EmptyState
          icon={BookOpen}
          title="Your decks have no flashcards yet"
          description="Add flashcards to a deck and it will be ready to review here."
        />
      )}

      {vm.decks.length > 0 && (
        <div className="flex flex-col gap-4">
          {vm.decks.map((deck) => (
            <ReviewDeckRow key={deck.name} deck={deck} />
          ))}
        </div>
      )}
    </Page>
  );
}
