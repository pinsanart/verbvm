import { motion } from "motion/react";
import { Check, Loader2, Save } from "lucide-react";
import { TypePicker } from "./TypePicker";

export interface EditorDockProps {
    /** How many cards wait to be saved. */
    pending: number;
    /** How many cards the last save wrote, shown for a moment in place of the count. */
    justSaved: number | null;
    busy: boolean;
    onAdd: (typeSlug: string) => void;
    onSave: () => void;
}

/**
 * The controls of the editor, floating over the bottom of the screen as one rounded dock: add a
 * card, and save the ones waiting. The save button says what it will do ("Save 3 cards"), and
 * after a save it confirms there, where the thumb or the pointer already is. On a phone it is
 * full width with the add button at the right, and on a wide screen it is a compact pill.
 */
export function EditorDock({ pending, justSaved, busy, onAdd, onSave }: EditorDockProps) {
    const confirmed = justSaved !== null && pending === 0;
    const label = confirmed
        ? `${justSaved} ${justSaved === 1 ? "card" : "cards"} saved`
        : pending === 0
          ? "Nothing to save"
          : `Save ${pending} ${pending === 1 ? "card" : "cards"}`;

    return (
        <div className="pointer-events-none sticky bottom-3 z-10 mt-2 flex justify-center">
            <motion.div
                layout
                className="pointer-events-auto flex w-full flex-row-reverse items-center gap-2 rounded-full border border-line-strong bg-raised/90 p-1.5 shadow-xl shadow-black/40 backdrop-blur md:w-auto md:flex-row"
            >
                <TypePicker onPick={onAdd} disabled={busy} />
                <button
                    type="button"
                    onClick={onSave}
                    disabled={busy || pending === 0}
                    aria-busy={busy || undefined}
                    className={`flex min-h-12 flex-1 items-center justify-center gap-2 rounded-full px-6 text-ui font-semibold transition-colors md:min-h-10 md:flex-none ${
                        confirmed
                            ? "bg-orthographic text-canvas"
                            : "bg-accent text-white hover:brightness-110 disabled:bg-white/10 disabled:text-muted"
                    }`}
                >
                    {busy ? (
                        <Loader2 size={16} className="animate-spin" aria-hidden />
                    ) : confirmed ? (
                        <Check size={16} aria-hidden />
                    ) : (
                        <Save size={16} aria-hidden />
                    )}
                    <span aria-live="polite">{label}</span>
                </button>
            </motion.div>
        </div>
    );
}
