import { AnimatePresence, motion } from "motion/react";
import { ArrowLeft, Save } from "lucide-react";
import { Link, useParams } from "react-router";
import { Button } from "../components/Button";
import { CountryFlag } from "../components/CountryFlag";
import { FlashcardInput } from "../components/FlashcardInput";
import { Notice } from "../components/Notice";
import { TypePicker } from "../components/TypePicker";
import { useFlashcardEditorViewModel } from "../viewmodels/useFlashcardEditorViewModel";

/** The editor of a deck: one list of flashcards of any type, with the type buttons at the bottom. */
export default function CreateFlashcards() {
  const { deckName = "" } = useParams();
  const vm = useFlashcardEditorViewModel(deckName);

  return (
    <div className="mx-auto flex w-full flex-col gap-6 p-6 md:max-w-[60%]">
      <div className="flex flex-col gap-2">
        <Link to="/create" className="flex items-center gap-1 self-start text-sm text-white/50 hover:text-white">
          <ArrowLeft size={14} />
          Decks
        </Link>
        <h1 className="text-2xl font-semibold">{deckName}</h1>
        {vm.form && (
          <div className="flex items-center gap-2 text-sm text-white/60">
            <CountryFlag code={vm.form.variant.countryCode} className="h-3.5 w-5" />
            {vm.form.variant.variantName}
          </div>
        )}
      </div>

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

      {vm.status === "loading" && <p className="text-sm text-white/50">Loading...</p>}

      {vm.status === "ready" && vm.form && (
        <>
          {vm.drafts.length === 0 && (
            <p className="rounded-2xl border border-dashed border-white/10 px-4 py-10 text-center text-sm text-white/50">
              No flashcards yet. Pick a type below to add one.
            </p>
          )}

          <div className="flex flex-col gap-6">
            <AnimatePresence initial={false}>
              {vm.drafts.map((draft, index) => {
                const fields = vm.fieldsByType[draft.typeSlug] ?? { front: [], back: [] };
                return (
                  <motion.div
                    key={draft.uuid}
                    layout="position"
                    initial={{ opacity: 0, y: 16 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, scale: 0.96 }}
                    transition={{ duration: 0.2 }}
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

          <div className="sticky bottom-16 z-10 -mx-6 flex flex-wrap items-center justify-between gap-3 border-t border-white/10 bg-panel/90 px-6 py-3 backdrop-blur md:bottom-0">
            <TypePicker onPick={vm.addDraft} disabled={vm.busy} />
            <Button onClick={vm.saveAll} disabled={vm.busy || vm.drafts.length === 0}>
              <Save size={16} />
              Save all ({vm.drafts.length})
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
