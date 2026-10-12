import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Save, X } from "lucide-react";
import { Button } from "./Button";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { FlashcardFaces } from "./FlashcardInput";
import { IconButton } from "./IconButton";
import { Notice } from "./Notice";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import { EASE_OUT } from "../lib/motion";
import type { FieldDefinition, SavedCard } from "../types/FlashcardForm";

export interface SavedCardPanelProps {
    /** The card being edited. The panel is closed when this is null. */
    card: SavedCard | null;
    frontFields: FieldDefinition[];
    backFields: FieldDefinition[];
    values: Record<string, string>;
    dirty: boolean;
    justSaved: boolean;
    error: string | null;
    busy: boolean;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onRemoveMedia: (key: string, sha256: string) => void;
    onSave: () => void;
    onDelete: () => void;
    onClose: () => void;
}

/**
 * The panel that edits one saved card. It slides in from the right on a wide screen and fills the
 * screen on a phone. Closing with unsaved text asks first, in the panel, and Escape closes it.
 */
export function SavedCardPanel(props: SavedCardPanelProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const { card, frontFields, backFields, values, dirty, justSaved, error, busy } = props;
    const { onChange, onAttachMedia, onRemoveMedia, onSave, onDelete, onClose } = props;
    const [confirmingDiscard, setConfirmingDiscard] = useState(false);
    const panel = useRef<HTMLDivElement>(null);
    const opener = useRef<Element | null>(null);

    // Remember what had the focus, move it into the panel, and give it back when the panel closes.
    const uuid = card?.uuid ?? null;
    useEffect(() => {
        if (!uuid) return;
        opener.current = document.activeElement;
        setConfirmingDiscard(false);
        panel.current?.focus();
        return () => {
            if (opener.current instanceof HTMLElement) opener.current.focus();
        };
    }, [uuid]);

    const requestClose = () => {
        if (dirty && !confirmingDiscard) setConfirmingDiscard(true);
        else onClose();
    };

    useEffect(() => {
        if (!uuid) return;
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") requestClose();
        };
        document.addEventListener("keydown", onKeyDown);
        return () => document.removeEventListener("keydown", onKeyDown);
    });

    const style = card ? styleForType(card.typeSlug) : null;

    return (
        <AnimatePresence>
            {card && style && (
                <>
                    <motion.div
                        key="backdrop"
                        className="fixed inset-0 z-40 hidden bg-black/50 md:block"
                        initial={{ opacity: 0 }}
                        animate={{ opacity: 1 }}
                        exit={{ opacity: 0 }}
                        transition={{ duration: 0.15, ease: EASE_OUT }}
                        onClick={requestClose}
                        aria-hidden
                    />
                    <motion.div
                        key="panel"
                        ref={panel}
                        role="dialog"
                        aria-modal="true"
                        aria-label={t("panel.editAria", { type: typeLabel(card.typeSlug) })}
                        tabIndex={-1}
                        className="fixed inset-y-0 right-0 z-50 flex w-full flex-col border-l border-line bg-panel outline-none md:max-w-xl"
                        initial={{ x: "100%" }}
                        animate={{ x: 0 }}
                        exit={{ x: "100%" }}
                        transition={{ duration: 0.22, ease: EASE_OUT }}
                    >
                        <header className="flex items-center gap-3 border-b border-line px-4 py-3">
                            <span className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${style.badge}`}>
                                <style.icon size={16} aria-hidden />
                                {typeLabel(card.typeSlug)}
                            </span>
                            <h2 className="text-ui font-semibold">{t("panel.title")}</h2>
                            <div className="ml-auto flex items-center gap-1">
                                <ConfirmDeleteButton
                                    deleteLabel={t("card.deleteAria")}
                                    confirmLabel={t("card.confirmDeleteAria")}
                                    onConfirm={onDelete}
                                />
                                <IconButton label={t("common.close")} icon={X} iconSize={20} onClick={requestClose} />
                            </div>
                        </header>

                        <div className="flex flex-1 flex-col gap-5 overflow-y-auto p-4 md:p-5">
                            <AnimatePresence initial={false}>
                                {error && (
                                    <Notice key="error" tone="error">
                                        {error}
                                    </Notice>
                                )}
                                {justSaved && !dirty && (
                                    <Notice key="saved" tone="success">
                                        {t("panel.saved")}
                                    </Notice>
                                )}
                            </AnimatePresence>

                            <FlashcardFaces
                                key={card.uuid}
                                frontFields={frontFields}
                                backFields={backFields}
                                values={values}
                                errors={{}}
                                media={card.media}
                                onChange={onChange}
                                onAttachMedia={onAttachMedia}
                                onRemoveMedia={onRemoveMedia}
                            />
                        </div>

                        <footer className="flex flex-wrap items-center justify-end gap-3 border-t border-line px-4 py-3">
                            {confirmingDiscard ? (
                                <>
                                    <p role="alert" className="mr-auto text-ui text-muted">
                                        {t("panel.discardPrompt")}
                                    </p>
                                    <Button variant="outline" onClick={() => setConfirmingDiscard(false)}>
                                        {t("panel.keepEditing")}
                                    </Button>
                                    <Button variant="danger" onClick={onClose}>
                                        {t("panel.discard")}
                                    </Button>
                                </>
                            ) : (
                                <>
                                    <Button variant="outline" onClick={requestClose}>
                                        {dirty ? t("common.cancel") : t("common.close")}
                                    </Button>
                                    <Button onClick={onSave} loading={busy} disabled={!dirty}>
                                        <Save size={16} aria-hidden />
                                        {t("common.saveChanges")}
                                    </Button>
                                </>
                            )}
                        </footer>
                    </motion.div>
                </>
            )}
        </AnimatePresence>
    );
}
