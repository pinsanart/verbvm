import { useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useNavigate } from "react-router";
import { DeckCard } from "../components/DeckCard";
import { DeckEditForm } from "../components/DeckEditForm";
import { NewDeckForm } from "../components/NewDeckForm";
import { NewDeckTile } from "../components/NewDeckTile";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { useDecksViewModel } from "../viewmodels/useDecksViewModel";

/** The first step of creating flashcards: pick a deck, edit one, or add a new one at the end. */
export default function Create() {
  const vm = useDecksViewModel();
  const navigate = useNavigate();
  const [creating, setCreating] = useState(false);
  /** The name of the deck being edited, if any. */
  const [editing, setEditing] = useState<string | null>(null);
  const newDeckTile = useRef<HTMLButtonElement>(null);

  // Closing the form hands the focus back to the tile that opened it.
  const closeForm = () => {
    setCreating(false);
    requestAnimationFrame(() => newDeckTile.current?.focus());
  };

  const openDeck = (name: string) => navigate(`/create/${encodeURIComponent(name)}`);
  const formOpen = creating || editing !== null;

  return (
    <Page title="Create flashcards" width="wide">
      <AnimatePresence initial={false}>
        {vm.error && !formOpen && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2" aria-busy="true" aria-label="Loading decks">
          {[0, 1].map((n) => (
            <div key={n} className="h-32 animate-pulse rounded-2xl border border-line bg-panel" />
          ))}
        </div>
      )}

      {vm.status !== "loading" && (
        <motion.div layout="position" className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <AnimatePresence initial={false}>
            {vm.decks.map((deck) =>
              editing === deck.name ? (
                <DeckEditForm
                  key={`edit-${deck.name}`}
                  deck={deck}
                  onSave={(changes) => vm.editDeck(deck.name, changes)}
                  onCancel={() => setEditing(null)}
                  busy={vm.busy}
                  error={vm.error}
                />
              ) : (
                <DeckCard
                  key={deck.name}
                  deck={deck}
                  onOpen={() => openDeck(deck.name)}
                  onEdit={() => {
                    setCreating(false);
                    setEditing(deck.name);
                  }}
                  onDelete={() => vm.removeDeck(deck.name)}
                />
              ),
            )}

            {vm.status === "ready" &&
              (creating ? (
                <NewDeckForm
                  key="new-deck"
                  languages={vm.languages}
                  loadVariants={vm.loadVariants}
                  onCreate={vm.addDeck}
                  onCancel={closeForm}
                  busy={vm.busy}
                  error={vm.error}
                />
              ) : (
                <NewDeckTile
                  key="new-deck-tile"
                  ref={newDeckTile}
                  first={vm.decks.length === 0}
                  onClick={() => {
                    setEditing(null);
                    setCreating(true);
                  }}
                />
              ))}
          </AnimatePresence>
        </motion.div>
      )}
    </Page>
  );
}
