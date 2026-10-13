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
import { StackInput } from "../components/StackInput";
import { Notice } from "../components/Notice";
import { Page } from "../components/Page";
import { SavedCardList } from "../components/SavedCardList";
import { SavedCardPanel } from "../components/SavedCardPanel";
import { SavedStackPanel } from "../components/SavedStackPanel";
import { TRANSITION_BASE } from "../lib/motion";
import type { StackData } from "../lib/stackFields";
import { useLanguageNames } from "../i18n/useLanguageNames";
import { useFlashcardEditorViewModel } from "../viewmodels/useFlashcardEditorViewModel";
import { useSavedCardsViewModel } from "../viewmodels/useSavedCardsViewModel";

type Tab = "new" | "saved";

/** What the stack panel holds while it is closed. */
const EMPTY_STACK: StackData = { stackUuid: "", phrase: "", lanes: [], missing: [], generating: false, notes: [] };

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
  const saved = useSavedCardsViewModel(deckName, vm.fieldsByType);
  const [tab, setTab] = useState<Tab>("new");

  // Cards published from the other tab show up when this one is opened, and the count in the tab
  // follows every save.
  const { reload } = saved;
  useEffect(() => {
    if (tab === "saved") reload();
  }, [tab, reload]);
  const { lastPublished } = vm;
  useEffect(() => {
    if (lastPublished !== null) reload();
  }, [lastPublished, reload]);

  // The row of a saved card previews its first text field: the audio of an orthographic card has none.
  const previewKeys = Object.fromEntries(
    Object.entries(vm.fieldsByType).map(([slug, fields]) => [
      slug,
      { front: fields.front.find((f) => f.kind !== "AUDIO"), back: fields.back[0] },
    ]),
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
        {vm.warnings.map((warning) => (
          <Notice key={warning} tone="info">
            {warning}
          </Notice>
        ))}
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
              rows={saved.rows}
              previewKeys={previewKeys}
              query={saved.query}
              typeFilter={saved.typeFilter}
              onQuery={saved.setQuery}
              onTypeFilter={saved.setTypeFilter}
              onOpen={saved.open}
              onOpenStack={saved.openStack}
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
              {vm.items.map((item, index) => {
                const accent = { name: names.variant(vm.form!.variant), countryCode: vm.form!.variant.countryCode };
                const isFirst = index === 0;
                const isLast = index === vm.items.length - 1;
                if (item.kind === "stack") {
                  const { stack } = item;
                  const first = item.drafts[0].uuid;
                  return (
                    <motion.div
                      key={stack.stackUuid}
                      layout="position"
                      initial={{ opacity: 0, y: 12 }}
                      animate={{ opacity: 1, y: 0 }}
                      exit={{ opacity: 0, scale: 0.97 }}
                      transition={TRANSITION_BASE}
                    >
                      <StackInput
                        stack={stack}
                        accent={accent}
                        isFirst={isFirst}
                        isLast={isLast}
                        onMoveUp={() => vm.move(first, "up")}
                        onMoveDown={() => vm.move(first, "down")}
                        onDuplicate={() => vm.duplicate(first)}
                        onDeleteStack={() => vm.removeStack(stack.stackUuid)}
                        onPhraseChange={(text) => vm.setStackPhrase(stack.stackUuid, text)}
                        onGenerateAll={() => vm.generateStack(stack.stackUuid)}
                        onAddMember={(slug) => vm.addToStack(stack.stackUuid, slug)}
                        onChange={vm.setValue}
                        onAttachMedia={vm.attachFiles}
                        onGenerate={vm.generate}
                        onRemoveMedia={vm.detachFile}
                        onToggleLink={vm.toggleLink}
                        onTakeOut={vm.takeOutOfStack}
                        onDeleteCard={vm.remove}
                      />
                    </motion.div>
                  );
                }
                const { draft } = item;
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
                      key={`${draft.uuid}:${vm.revisionOf(draft.uuid)}`}
                      typeSlug={draft.typeSlug}
                      accent={accent}
                      frontFields={fields.front}
                      backFields={fields.back}
                      values={draft.values}
                      errors={vm.errorsFor(draft.uuid)}
                      media={draft.media}
                      isFirst={isFirst}
                      isLast={isLast}
                      onChange={(key, value) => vm.setValue(draft.uuid, key, value)}
                      onAttachMedia={(key) => vm.attachFiles(draft.uuid, key)}
                      onGenerate={(key) => vm.generate(draft.uuid, key)}
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
        onGenerate={saved.generate}
        onRemoveMedia={saved.detachFile}
        revision={saved.selected ? saved.revisionOf(saved.selected.uuid) : 0}
        onSave={saved.save}
        onDelete={saved.remove}
        onClose={saved.close}
      />

      {/* Always rendered, so the panel can slide in and out. */}
      {(
        <SavedStackPanel
          stackUuid={saved.stack?.stackUuid ?? null}
          stack={saved.stack ?? EMPTY_STACK}
          dirty={saved.stackDirty}
          justSaved={saved.justSaved}
          error={saved.error}
          busy={saved.busy}
          onSave={saved.saveStack}
          onDeleteStack={saved.removeStack}
          onClose={saved.closeStack}
          onPhraseChange={saved.setStackPhrase}
          onGenerateAll={saved.generateAll}
          onAddMember={() => undefined}
          onChange={saved.setStackValue}
          onAttachMedia={saved.attachStackFiles}
          onGenerate={saved.generateField}
          onRemoveMedia={saved.detachStackFile}
          onToggleLink={saved.toggleLink}
          onTakeOut={saved.takeOut}
          onDeleteCard={saved.deleteStackCard}
        />
      )}
    </Page>
  );
}
