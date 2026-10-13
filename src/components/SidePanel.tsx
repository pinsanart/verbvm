import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Save, X } from "lucide-react";
import { Button } from "./Button";
import { IconButton } from "./IconButton";
import { Notice } from "./Notice";
import { EASE_OUT } from "../lib/motion";
import { useKeys } from "../lib/useKeys";

export interface SidePanelProps {
    /** What is being edited. The panel is closed when this is null, and reset when it changes. */
    id: string | null;
    ariaLabel: string;
    /** What leads the header, such as the type badge. */
    badge: React.ReactNode;
    title: string;
    /** Controls at the end of the header, next to the close button. */
    headerActions: React.ReactNode;
    dirty: boolean;
    justSaved: boolean;
    error: string | null;
    busy: boolean;
    onSave: () => void;
    onClose: () => void;
    children: React.ReactNode;
}

/**
 * The panel that edits saved cards. It slides in from the right on a wide screen and fills the
 * screen on a phone. Closing with unsaved text asks first, in the panel, and Escape closes it.
 */
export function SidePanel({
    id,
    ariaLabel,
    badge,
    title,
    headerActions,
    dirty,
    justSaved,
    error,
    busy,
    onSave,
    onClose,
    children,
}: SidePanelProps) {
    const { t } = useTranslation();
    const [confirmingDiscard, setConfirmingDiscard] = useState(false);
    const panel = useRef<HTMLDivElement>(null);
    const opener = useRef<Element | null>(null);

    // Remember what had the focus, move it into the panel, and give it back when the panel closes.
    useEffect(() => {
        if (!id) return;
        opener.current = document.activeElement;
        setConfirmingDiscard(false);
        panel.current?.focus();
        return () => {
            if (opener.current instanceof HTMLElement) opener.current.focus();
        };
    }, [id]);

    const requestClose = () => {
        if (dirty && !confirmingDiscard) setConfirmingDiscard(true);
        else onClose();
    };

    useKeys(["Escape"], requestClose, id !== null);

    return (
        <AnimatePresence>
            {id && (
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
                        aria-label={ariaLabel}
                        tabIndex={-1}
                        className="fixed inset-y-0 right-0 z-50 flex w-full flex-col border-l border-line bg-panel outline-none md:max-w-xl"
                        initial={{ x: "100%" }}
                        animate={{ x: 0 }}
                        exit={{ x: "100%" }}
                        transition={{ duration: 0.22, ease: EASE_OUT }}
                    >
                        <header className="flex items-center gap-3 border-b border-line px-4 py-3">
                            {badge}
                            <h2 className="text-ui font-semibold">{title}</h2>
                            <div className="ml-auto flex items-center gap-1">
                                {headerActions}
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

                            {children}
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
