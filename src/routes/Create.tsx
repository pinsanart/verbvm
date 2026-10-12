import { useState } from "react";
import { Plus } from "lucide-react";
import { useNavigate } from "react-router";
import { DeckCard } from "../components/DeckCard";
import { NewDeckDialog } from "../components/NewDeckDialog";
import { useDecksViewModel } from "../viewmodels/useDecksViewModel";

/** The first step of creating flashcards: pick a deck, or create one. */
export default function Create() {
  const vm = useDecksViewModel();
  const navigate = useNavigate();
  const [dialogOpen, setDialogOpen] = useState(false);

  const openDeck = (name: string) => navigate(`/create/${encodeURIComponent(name)}`);

  return (
    <div className="mx-auto flex w-full flex-col gap-6 p-6 md:max-w-[60%]">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Create flashcards</h1>
        <button
          type="button"
          onClick={() => setDialogOpen(true)}
          disabled={vm.status !== "ready"}
          className="flex items-center gap-1.5 rounded-lg bg-vocabulary px-3 py-2 text-sm font-medium disabled:opacity-50"
        >
          <Plus size={16} />
          New deck
        </button>
      </div>

      {vm.error && !dialogOpen && (
        <div role="alert" className="rounded-lg border border-stack/70 px-3 py-2 text-sm text-stack">
          {vm.error}
        </div>
      )}

      {vm.status === "loading" && <p className="text-sm text-white/50">Loading decks...</p>}

      {vm.status === "ready" && vm.decks.length === 0 && (
        <p className="text-sm text-white/50">No decks yet. Create one to start adding flashcards.</p>
      )}

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        {vm.decks.map((deck) => (
          <DeckCard
            key={deck.name}
            deck={deck}
            onOpen={() => openDeck(deck.name)}
            onDelete={() => vm.removeDeck(deck.name)}
          />
        ))}
      </div>

      {dialogOpen && (
        <NewDeckDialog
          languages={vm.languages}
          loadVariants={vm.loadVariants}
          onCreate={vm.addDeck}
          onClose={() => setDialogOpen(false)}
          busy={vm.busy}
          error={vm.error}
        />
      )}
    </div>
  );
}
