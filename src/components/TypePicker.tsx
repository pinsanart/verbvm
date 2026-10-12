import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ENABLED_TYPE_SLUGS, styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import { TRANSITION_FAST } from "../lib/motion";

export interface TypePickerProps {
    /** Called with the slug of the type the user picked. */
    onPick: (typeSlug: string) => void;
    disabled?: boolean;
}

/**
 * The control that adds a card of a type to the deck. Only the types that can be written today are
 * offered, so everything in it does something. On a wide screen each type is a button. On a phone
 * a round "+" at the start of the bar opens the list of types above it, where a thumb reaches.
 */
export function TypePicker({ onPick, disabled }: TypePickerProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const [open, setOpen] = useState(false);
    const root = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!open) return;
        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") setOpen(false);
        };
        const onPointerDown = (e: PointerEvent) => {
            if (!root.current?.contains(e.target as Node)) setOpen(false);
        };
        document.addEventListener("keydown", onKeyDown);
        document.addEventListener("pointerdown", onPointerDown);
        return () => {
            document.removeEventListener("keydown", onKeyDown);
            document.removeEventListener("pointerdown", onPointerDown);
        };
    }, [open]);

    const pick = (slug: string) => {
        setOpen(false);
        onPick(slug);
    };

    return (
        <div ref={root} className="relative">
            {/* Wide screens: one button per type. */}
            <div className="hidden flex-wrap items-center gap-2 md:flex" role="group" aria-label={t("dock.add")}>
                {ENABLED_TYPE_SLUGS.map((slug) => {
                    const { icon: Icon, text } = styleForType(slug);
                    return (
                        <button
                            key={slug}
                            type="button"
                            disabled={disabled}
                            onClick={() => pick(slug)}
                            className="flex min-h-10 items-center gap-2 rounded-full px-4 text-ui font-medium text-ink transition-colors hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-50"
                        >
                            <Plus size={16} className="text-muted" aria-hidden />
                            <Icon size={18} className={text} aria-hidden />
                            {typeLabel(slug)}
                        </button>
                    );
                })}
            </div>

            {/* Phones: a round "+" that opens the types. */}
            <div className="md:hidden">
                <AnimatePresence>
                    {open && (
                        <motion.div
                            role="group"
                            aria-label={t("dock.add")}
                            initial={{ opacity: 0, y: 8 }}
                            animate={{ opacity: 1, y: 0 }}
                            exit={{ opacity: 0, y: 8 }}
                            transition={TRANSITION_FAST}
                            className="absolute bottom-full right-0 mb-3 flex min-w-48 flex-col gap-1 rounded-2xl border border-line-strong bg-raised p-1.5 shadow-lg shadow-black/40"
                        >
                            {ENABLED_TYPE_SLUGS.map((slug) => {
                                const { icon: Icon } = styleForType(slug);
                                return (
                                    <button
                                        key={slug}
                                        type="button"
                                        onClick={() => pick(slug)}
                                        className="flex min-h-12 items-center gap-3 rounded-xl px-3 text-ui font-medium text-ink transition-colors hover:bg-white/10"
                                    >
                                        <Icon size={20} aria-hidden />
                                        {typeLabel(slug)}
                                    </button>
                                );
                            })}
                        </motion.div>
                    )}
                </AnimatePresence>
                <button
                    type="button"
                    disabled={disabled}
                    aria-expanded={open}
                    aria-label={t("dock.add")}
                    onClick={() => setOpen((v) => !v)}
                    className="flex size-12 items-center justify-center rounded-full bg-accent text-white shadow-lg shadow-black/30 transition-[filter] hover:brightness-110 disabled:pointer-events-none disabled:opacity-50"
                >
                    <motion.span animate={{ rotate: open ? 45 : 0 }} transition={TRANSITION_FAST} className="flex">
                        <Plus size={24} aria-hidden />
                    </motion.span>
                </button>
            </div>
        </div>
    );
}
