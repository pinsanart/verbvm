import { ArrowLeft, Plus, Save } from "lucide-react";
import { Link, useParams } from "react-router";
import { FlashcardInput } from "../components/FlashcardInput";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useFlashcardEditorViewModel } from "../viewmodels/useFlashcardEditorViewModel";

/** The editor of the flashcards of one type in a deck. */
export default function CreateFlashcards() {
  const { deckName = "", typeSlug = "" } = useParams();
  const vm = useFlashcardEditorViewModel(deckName, typeSlug);
  const style = styleForType(typeSlug);

  return (
    <div className="mx-auto flex w-full flex-col gap-6 p-6 md:max-w-[60%]">
      <div className="flex flex-col gap-2">
        <Link
          to={`/create/${encodeURIComponent(deckName)}`}
          className="flex items-center gap-1 self-start text-sm text-white/50 hover:text-white"
        >
          <ArrowLeft size={14} />
          {deckName}
        </Link>
        <h1 className="text-2xl font-semibold">
          <span className={style.text}>{vm.form?.flashcardType.name ?? style.label}</span> flashcards
        </h1>
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
          <div className="flex flex-col gap-6">
            {vm.drafts.map((draft, index) => (
              <FlashcardInput
                key={draft.uuid}
                typeSlug={typeSlug}
                accent={{ name: vm.form!.variant.variantName, countryCode: vm.form!.variant.countryCode }}
                frontFields={vm.fieldsBySide.front}
                backFields={vm.fieldsBySide.back}
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
            ))}
          </div>

          <div className="flex items-center justify-between">
            <button
              type="button"
              onClick={vm.addDraft}
              disabled={vm.busy}
              className="flex items-center gap-1.5 rounded-lg border border-white/10 px-3 py-2 text-sm text-white/70 hover:bg-white/10 hover:text-white disabled:opacity-50"
            >
              <Plus size={16} />
              Add card
            </button>
            <button
              type="button"
              onClick={vm.saveAll}
              disabled={vm.busy}
              className={`flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-50 ${style.accent}`}
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
