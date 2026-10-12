import { useEffect, useId, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { ChevronUp } from "lucide-react";
import { buttonClass } from "./Button";
import { TRANSITION_FAST } from "../lib/motion";

const ITEMS = ["Edit card", "Delete card"] as const;

/**
 * The Actions menu of a review. For now it only shows what is planned: its items are disabled
 * and marked "Soon". It opens above the button, because the bar it sits in is at the bottom.
 */
export function ActionsMenu() {
    const menuId = useId();
    const [open, setOpen] = useState(false);
    const container = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!open) return;
        const onPointerDown = (e: PointerEvent) => {
            if (!container.current?.contains(e.target as Node)) setOpen(false);
        };
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") setOpen(false);
        };
        document.addEventListener("pointerdown", onPointerDown);
        document.addEventListener("keydown", onKeyDown);
        return () => {
            document.removeEventListener("pointerdown", onPointerDown);
            document.removeEventListener("keydown", onKeyDown);
        };
    }, [open]);

    return (
        <div ref={container} className="relative">
            <button
                type="button"
                aria-haspopup="menu"
                aria-expanded={open}
                aria-controls={menuId}
                onClick={() => setOpen((v) => !v)}
                className={buttonClass("outline")}
            >
                Actions
                <ChevronUp size={16} aria-hidden className={`transition-transform ${open ? "" : "rotate-180"}`} />
            </button>

            <AnimatePresence>
                {open && (
                    <motion.div
                        id={menuId}
                        role="menu"
                        aria-label="Card actions"
                        initial={{ opacity: 0, y: 4 }}
                        animate={{ opacity: 1, y: 0 }}
                        exit={{ opacity: 0, y: 4 }}
                        transition={TRANSITION_FAST}
                        className="absolute bottom-full right-0 z-30 mb-2 w-52 rounded-xl border border-line bg-raised py-1 shadow-xl shadow-black/40"
                    >
                        {ITEMS.map((item) => (
                            <div
                                key={item}
                                role="menuitem"
                                aria-disabled="true"
                                className="flex items-center justify-between gap-3 px-3 py-2 text-ui text-subtle"
                            >
                                {item}
                                <span className="rounded-full bg-white/10 px-1.5 py-0.5 text-label font-medium text-muted">
                                    Soon
                                </span>
                            </div>
                        ))}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    );
}
