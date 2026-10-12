import { useRef, useState } from "react";
import { AnimatePresence } from "motion/react";
import { Plus } from "lucide-react";
import { useNavigate } from "react-router";
import { Button } from "../components/Button";
import { DeckCard } from "../components/DeckCard";
import { NewDeckForm } from "../components/NewDeckForm";
import { Notice } from "../components/Notice";
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
    <div className="mx-auto flex w-full flex-col gap-6 p-6 md:max-w-[60%]">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Create flashcards</h1>
        <Button
          ref={newDeckButton}
          onClick={() => setCreating(true)}
          disabled={vm.status !== "ready" || creating}
          aria-expanded={creating}
        >
          <Plus size={16} />
          New deck
        </Button>
      </div>

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
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2" aria-busy="true" aria-label="Loading decks">
          {[0, 1].map((n) => (
            <div key={n} className="h-32 animate-pulse rounded-2xl border border-line bg-white/5" />
          ))}
        </div>
      )}

      {vm.status === "ready" && vm.decks.length === 0 && (
        <p className="rounded-2xl border border-dashed border-line px-4 py-10 text-center text-sm text-muted">
          No decks yet. Create one to start adding flashcards.
        </p>
      )}

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
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
      </div>

    </div>
  );
}
