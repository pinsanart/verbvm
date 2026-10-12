import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "./IconButton";
import { keyboard } from "../lib/keyboard";
import { TRANSITION_FAST } from "../lib/motion";

/** How long the confirmation waits before it goes back to the trash button. */
const CONFIRM_TIMEOUT_MS = 8000;

export interface ConfirmDeleteButtonProps {
    /** Names the trash button for screen readers, for example "Delete this flashcard". */
    deleteLabel: string;
    /** Names the Keep and Delete group for screen readers. */
    confirmLabel: string;
    onConfirm: () => void;
    iconSize?: number;
}

/**
 * A trash button that asks for confirmation inside the view, never with a system dialog.
 * Pressing it swaps it for a Keep and a Delete button. Keep (focused first, the safe choice),
 * Escape, clicking elsewhere and a short timeout all go back to the trash button.
 */
export function ConfirmDeleteButton({ deleteLabel, confirmLabel, onConfirm, iconSize = 15 }: ConfirmDeleteButtonProps) {
    const { t } = useTranslation();
    const [confirming, setConfirming] = useState(false);
    const container = useRef<HTMLDivElement>(null);
    const keepButton = useRef<HTMLButtonElement>(null);
    const trashButton = useRef<HTMLButtonElement>(null);
    // Going back to the trash button should keep the keyboard user where they were. Clicking
    // elsewhere should not pull the focus away from what they clicked.
    const restoreFocus = useRef(false);

    useEffect(() => {
        if (confirming) return;
        if (restoreFocus.current) {
            restoreFocus.current = false;
            trashButton.current?.focus();
        }
    }, [confirming]);

    useEffect(() => {
        if (!confirming) return;
        keepButton.current?.focus();

        const cancel = (keepFocus: boolean) => {
            // Only when the focus is still in the group (or lost), so it is not taken from a click elsewhere.
            const active = document.activeElement;
            restoreFocus.current = keepFocus && (active === document.body || container.current?.contains(active) === true);
            setConfirming(false);
        };
        const onPointerDown = (e: PointerEvent) => {
            if (!container.current?.contains(e.target as Node)) cancel(false);
        };
        const timer = setTimeout(() => cancel(true), CONFIRM_TIMEOUT_MS);

        const stopKeys = keyboard().subscribe({ keys: ["Escape"], notify: () => cancel(true) });
        document.addEventListener("pointerdown", onPointerDown);
        return () => {
            clearTimeout(timer);
            stopKeys();
            document.removeEventListener("pointerdown", onPointerDown);
        };
    }, [confirming]);

    return (
        <div ref={container} className="flex items-center">
            <AnimatePresence mode="wait" initial={false}>
                {confirming ? (
                    <motion.div
                        key="confirm"
                        role="group"
                        aria-label={confirmLabel}
                        className="flex items-center gap-1"
                        initial={{ opacity: 0, x: 8 }}
                        animate={{ opacity: 1, x: 0 }}
                        exit={{ opacity: 0, x: 8 }}
                        transition={TRANSITION_FAST}
                    >
                        <button
                            ref={keepButton}
                            type="button"
                            onClick={() => {
                                restoreFocus.current = true;
                                setConfirming(false);
                            }}
                            className="min-h-10 rounded-md px-3 py-1.5 md:min-h-8 text-meta font-medium text-muted hover:bg-white/10 hover:text-ink"
                        >
                            {t("common.keep")}
                        </button>
                        <button
                            type="button"
                            onClick={() => {
                                setConfirming(false);
                                onConfirm();
                            }}
                            className="min-h-10 rounded-md bg-danger px-3 py-1.5 md:min-h-8 text-meta font-medium text-white hover:brightness-110"
                        >
                            {t("common.delete")}
                        </button>
                    </motion.div>
                ) : (
                    <motion.div
                        key="trash"
                        initial={{ opacity: 0, scale: 0.9 }}
                        animate={{ opacity: 1, scale: 1 }}
                        exit={{ opacity: 0, scale: 0.9 }}
                        transition={TRANSITION_FAST}
                    >
                        <IconButton
                            ref={trashButton}
                            label={deleteLabel}
                            icon={Trash2}
                            iconSize={iconSize}
                            onClick={() => setConfirming(true)}
                        />
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
