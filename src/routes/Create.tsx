import { useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { FolderPlus, Plus } from "lucide-react";
import { useNavigate } from "react-router";
import { Button } from "../components/Button";
import { DeckCard } from "../components/DeckCard";
import { EmptyState } from "../components/EmptyState";
import { NewDeckForm } from "../components/NewDeckForm";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { useDecksViewModel } from "../viewmodels/useDecksViewModel";

/** The first step of creating flashcards: pick a deck, or create one. */
export default function Create() {
  const vm = useDecksViewModel();
  const navigate = useNavigate();
  const [creating, setCreating] = useState(false);
  const newDeckButton = useRef<HTMLButtonElement>(null);

  // Closing the form hands the focus back to the button that opened it.
  const closeForm = () => {
    setCreating(false);
    requestAnimationFrame(() => newDeckButton.current?.focus());
  };

  const openDeck = (name: string) => navigate(`/create/${encodeURIComponent(name)}`);

  return (
    <Page
      title="Create flashcards"
      width="wide"
      actions={
        <Button
          ref={newDeckButton}
          onClick={() => setCreating(true)}
          disabled={vm.status !== "ready" || creating}
          aria-expanded={creating}
        >
          <Plus size={16} aria-hidden />
          New deck
        </Button>
      }
    >
      <AnimatePresence initial={false}>
        {vm.error && !creating && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      <AnimatePresence initial={false}>
        {creating && (
          <NewDeckForm
            key="new-deck"
            languages={vm.languages}
            loadVariants={vm.loadVariants}
            onCreate={vm.addDeck}
            onCancel={closeForm}
            busy={vm.busy}
            error={vm.error}
          />
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2" aria-busy="true" aria-label="Loading decks">
          {[0, 1].map((n) => (
            <div key={n} className="h-32 animate-pulse rounded-2xl border border-line bg-panel" />
          ))}
        </div>
      )}

      {vm.status === "ready" && vm.decks.length === 0 && !creating && (
        <EmptyState
          icon={FolderPlus}
          title="No decks yet"
          description="A deck holds flashcards in one language and accent. Create your first one to start writing cards."
          action={<Button onClick={() => setCreating(true)}>New deck</Button>}
        />
      )}

      <motion.div layout="position" className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <AnimatePresence>
          {vm.decks.map((deck) => (
            <DeckCard
              key={deck.name}
              deck={deck}
              onOpen={() => openDeck(deck.name)}
              onDelete={() => vm.removeDeck(deck.name)}
            />
          ))}
        </AnimatePresence>
      </motion.div>
    </Page>
  );
}
