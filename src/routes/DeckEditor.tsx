import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Layers } from "lucide-react";
import { useParams } from "react-router";
import { Button } from "../components/Button";
import { CountryFlag } from "../components/CountryFlag";
import { EditorDock } from "../components/EditorDock";
import { EmptyState } from "../components/EmptyState";
import { FlashcardInput } from "../components/FlashcardInput";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { SavedCardList } from "../components/SavedCardList";
import { SavedCardPanel } from "../components/SavedCardPanel";
import { TRANSITION_BASE } from "../lib/motion";
import { useLanguageNames } from "../i18n/useLanguageNames";
import { useFlashcardEditorViewModel } from "../viewmodels/useFlashcardEditorViewModel";
import { useSavedCardsViewModel } from "../viewmodels/useSavedCardsViewModel";

type Tab = "new" | "saved";

/**
 * The screen of a deck. "New cards" is the editor: one list of drafts of any type, with the
 * buttons that add one at the bottom. "Saved cards" lists what is already saved, and a card opens
 * in a panel to be edited or deleted.
 */
export default function DeckEditor() {
  const { t } = useTranslation();
  const names = useLanguageNames();
  const { deckName = "" } = useParams();
  const vm = useFlashcardEditorViewModel(deckName);
  const saved = useSavedCardsViewModel(deckName);
  const [tab, setTab] = useState<Tab>("new");

  // Cards published from the other tab show up when this one is opened.
  const { reload } = saved;
  useEffect(() => {
    if (tab === "saved") reload();
  }, [tab, reload]);

  const previewKeys = Object.fromEntries(
    Object.entries(vm.fieldsByType).map(([slug, fields]) => [slug, { front: fields.front[0], back: fields.back[0] }]),
  );
  const panelFields = vm.fieldsByType[saved.selected?.typeSlug ?? ""] ?? { front: [], back: [] };
  const tabs: { id: Tab; label: string; count: number }[] = [
    { id: "new", label: t("deckEditor.newCards"), count: vm.drafts.length },
    { id: "saved", label: t("deckEditor.savedCards"), count: saved.cards.length },
  ];

  return (
    <Page
      title={deckName}
      parents={[{ to: "/decks", label: t("nav.decks") }]}
      meta={
        vm.form && (
          <>
            <CountryFlag code={vm.form.variant.countryCode} className="h-3.5 w-5" />
            {names.variant(vm.form.variant)}
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
        {saved.error && !saved.selected && (
          <Notice key="saved-error" tone="error">
            {saved.error}
          </Notice>
        )}
      </AnimatePresence>

      {vm.status === "loading" && (
        <div className="flex flex-col gap-6" aria-busy="true" aria-label={t("deckEditor.loading")}>
          <div className="h-48 animate-pulse rounded-2xl border border-line bg-panel" />
        </div>
      )}

      {vm.status === "ready" && vm.form && (
        <div role="tablist" aria-label={t("deckEditor.tabsAria")} className="flex gap-1 border-b border-line">
          {tabs.map(({ id, label, count }) => (
            <button
              key={id}
              type="button"
              role="tab"
              id={`tab-${id}`}
              aria-selected={tab === id}
              aria-controls={`panel-${id}`}
              onClick={() => setTab(id)}
              className={`-mb-px min-h-11 border-b-2 px-4 text-ui font-medium transition-colors ${
                tab === id ? "border-accent-ink text-ink" : "border-transparent text-muted hover:text-ink"
              }`}
            >
              {label} <span className="tabular-nums text-subtle">{count}</span>
            </button>
          ))}
        </div>
      )}

      {vm.status === "ready" && vm.form && tab === "saved" && (
        <div role="tabpanel" id="panel-saved" aria-labelledby="tab-saved">
          {saved.cards.length === 0 && saved.status !== "loading" ? (
            <EmptyState
              icon={Layers}
              title={t("deckEditor.noSavedTitle")}
              description={t("deckEditor.noSavedDescription")}
              action={
                <Button variant="outline" onClick={() => setTab("new")}>
                  {t("deckEditor.goToNewCards")}
                </Button>
              }
            />
          ) : (
            <SavedCardList
              all={saved.cards}
              cards={saved.visible}
              previewKeys={previewKeys}
              query={saved.query}
              typeFilter={saved.typeFilter}
              onQuery={saved.setQuery}
              onTypeFilter={saved.setTypeFilter}
              onOpen={saved.open}
            />
          )}
        </div>
      )}

      {vm.status === "ready" && vm.form && tab === "new" && (
        <div role="tabpanel" id="panel-new" aria-labelledby="tab-new" className="contents">
          {vm.drafts.length === 0 && (
            <EmptyState
              icon={Layers}
              title={t("deckEditor.firstCardTitle")}
              description={t("deckEditor.firstCardDescription")}
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
                      accent={{ name: names.variant(vm.form!.variant), countryCode: vm.form!.variant.countryCode }}
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

          <EditorDock
            pending={vm.drafts.length}
            justSaved={vm.lastPublished}
            busy={vm.busy}
            onAdd={vm.addDraft}
            onSave={vm.saveAll}
          />
        </div>
      )}

      <SavedCardPanel
        card={saved.selected}
        frontFields={panelFields.front}
        backFields={panelFields.back}
        values={saved.values}
        dirty={saved.dirty}
        justSaved={saved.justSaved}
        error={saved.error}
        busy={saved.busy}
        onChange={saved.setValue}
        onAttachMedia={saved.attachFiles}
        onRemoveMedia={saved.detachFile}
        onSave={saved.save}
        onDelete={saved.remove}
        onClose={saved.close}
      />
    </Page>
  );
}
