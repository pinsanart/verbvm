import { ArrowLeft, Save } from "lucide-react";
import { Link, useParams } from "react-router";
import { CountryFlag } from "../components/CountryFlag";
import { FlashcardInput } from "../components/FlashcardInput";
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

      {vm.error && (
        <div role="alert" className="rounded-lg border border-stack/70 px-3 py-2 text-sm text-stack">
          {vm.error}
        </div>
      )}

      {vm.lastPublished !== null && (
        <div role="status" className="rounded-lg border border-orthographic/50 px-3 py-2 text-sm text-orthographic">
          {vm.lastPublished} {vm.lastPublished === 1 ? "flashcard" : "flashcards"} saved.
        </div>
      )}

      {vm.status === "loading" && <p className="text-sm text-white/50">Loading...</p>}

      {vm.status === "ready" && vm.form && (
        <>
          {vm.drafts.length === 0 && (
            <p className="rounded-2xl border border-dashed border-white/10 px-4 py-10 text-center text-sm text-white/50">
              No flashcards yet. Pick a type below to add one.
            </p>
          )}

          <div className="flex flex-col gap-6">
            {vm.drafts.map((draft, index) => {
              const fields = vm.fieldsByType[draft.typeSlug] ?? { front: [], back: [] };
              return (
                <FlashcardInput
                  key={draft.uuid}
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
              );
            })}
          </div>

          <div className="sticky bottom-16 z-10 -mx-6 flex flex-wrap items-center justify-between gap-3 border-t border-white/10 bg-panel/90 px-6 py-3 backdrop-blur md:bottom-0">
            <TypePicker onPick={vm.addDraft} disabled={vm.busy} />
            <button
              type="button"
              onClick={vm.saveAll}
              disabled={vm.busy || vm.drafts.length === 0}
              className="flex items-center gap-1.5 rounded-lg bg-vocabulary px-4 py-2 text-sm font-medium disabled:opacity-50"
            >
              <Save size={16} />
              Save all ({vm.drafts.length})
            </button>
          </div>
        </>
      )}
    </div>
  );
}
