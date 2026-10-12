import { AnimatePresence, motion } from "motion/react";
import { Layers, Save } from "lucide-react";
import { useParams } from "react-router";
import { Button } from "../components/Button";
import { CountryFlag } from "../components/CountryFlag";
import { EmptyState } from "../components/EmptyState";
import { FlashcardInput } from "../components/FlashcardInput";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { TypePicker } from "../components/TypePicker";
import { TRANSITION_BASE } from "../lib/motion";
import { useFlashcardEditorViewModel } from "../viewmodels/useFlashcardEditorViewModel";

/** The editor of a deck: one list of flashcards of any type, with the type buttons at the bottom. */
export default function DeckEditor() {
  const { deckName = "" } = useParams();
  const vm = useFlashcardEditorViewModel(deckName);

  return (
    <Page
      title={deckName}
      parents={[{ to: "/decks", label: "Decks" }]}
      meta={
        vm.form && (
          <>
            <CountryFlag code={vm.form.variant.countryCode} className="h-3.5 w-5" />
            {vm.form.variant.variantName}
          </>
        )
      }
    >
      <AnimatePresence initial={false}>
        {vm.error && (
          <Notice key="error" tone="error">
            {vm.error}
          </Notice>
        )}
        {vm.lastPublished !== null && (
          <Notice key="published" tone="success">
            {vm.lastPublished} {vm.lastPublished === 1 ? "flashcard" : "flashcards"} saved.
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="flex flex-col gap-6" aria-busy="true" aria-label="Loading flashcards">
          <div className="h-48 animate-pulse rounded-2xl border border-line bg-panel" />
        </div>
      )}

      {vm.status === "ready" && vm.form && (
        <>
          {vm.drafts.length === 0 && (
            <EmptyState
              icon={Layers}
              title="Add your first flashcard"
              description="Use the button below to add a card. Your cards stay here as drafts until you press Save all."
            />
          )}

          <div className="flex flex-col gap-6">
            <AnimatePresence initial={false}>
              {vm.drafts.map((draft, index) => {
                const fields = vm.fieldsByType[draft.typeSlug] ?? { front: [], back: [] };
                return (
                  <motion.div
                    key={draft.uuid}
                    layout="position"
                    initial={{ opacity: 0, y: 12 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.97 }}
                    transition={TRANSITION_BASE}
                  >
                    <FlashcardInput
                      typeSlug={draft.typeSlug}
                      accent={{ name: vm.form!.variant.variantName, countryCode: vm.form!.variant.countryCode }}
                      frontFields={fields.front}
                      backFields={fields.back}
                      values={draft.values}
                      errors={vm.errorsFor(draft.uuid)}
                      media={draft.media}
                      isFirst={index === 0}
                      isLast={index === vm.drafts.length - 1}
                      onChange={(key, value) => vm.setValue(draft.uuid, key, value)}
                      onAttachMedia={(key) => vm.attachFiles(draft.uuid, key)}
                      onRemoveMedia={(key, sha256) => vm.detachFile(draft.uuid, key, sha256)}
                      onMoveUp={() => vm.move(draft.uuid, "up")}
                      onMoveDown={() => vm.move(draft.uuid, "down")}
                      onDuplicate={() => vm.duplicate(draft.uuid)}
                      onDelete={() => vm.remove(draft.uuid)}
                    />
                  </motion.div>
                );
              })}
            </AnimatePresence>
          </div>

          <div className="sticky bottom-16 z-10 -mx-4 flex flex-wrap items-center justify-between gap-3 border-t border-line bg-canvas/90 px-4 py-3 backdrop-blur md:-mx-8 md:bottom-0 md:px-8">
            <TypePicker onPick={vm.addDraft} disabled={vm.busy} />
            <Button onClick={vm.saveAll} loading={vm.busy} disabled={vm.drafts.length === 0}>
              <Save size={16} aria-hidden />
              Save all ({vm.drafts.length})
            </Button>
          </div>
        </>
      )}
    </Page>
  );
}
