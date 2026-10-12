import { FormEvent, useId, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "./Button";
import { Notice } from "./Notice";
import { controlClass, labelClass } from "./formStyles";
import { TRANSITION_BASE } from "../lib/motion";
import type { Deck, UpdateDeck } from "../types/Deck";

export interface DeckEditFormProps {
    deck: Deck;
    /** Resolves to `true` when the changes were saved. */
    onSave: (changes: UpdateDeck) => Promise<boolean>;
    onCancel: () => void;
    busy: boolean;
    error: string | null;
}

/**
 * Edits a deck in the place where its card was. Only the name and the description can change:
 * the language and accent are fixed, because the fields of its flashcards depend on them.
 */
export function DeckEditForm({ deck, onSave, onCancel, busy, error }: DeckEditFormProps) {
    const id = useId();
    const [name, setName] = useState(deck.name);
    const [description, setDescription] = useState(deck.description ?? "");

    const submit = async (e: FormEvent) => {
        e.preventDefault();
        const saved = await onSave({
            name,
            description: description.trim() === "" ? null : description,
        });
        if (saved) onCancel();
    };

    return (
        <motion.form
            layout
            aria-label={`Edit ${deck.name}`}
            onSubmit={submit}
            onKeyDown={(e) => {
                if (e.key === "Escape") onCancel();
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={TRANSITION_BASE}
            className="flex flex-col gap-5 rounded-2xl border border-line-strong bg-panel p-5 md:col-span-2 md:p-6"
        >
            <h2 className="text-heading">Edit deck</h2>

            <AnimatePresence initial={false}>
                {error && (
                    <Notice key="error" tone="error">
                        {error}
                    </Notice>
                )}
            </AnimatePresence>

            <div>
                <label className={labelClass} htmlFor={`${id}-name`}>
                    Name
                </label>
                <input
                    id={`${id}-name`}
                    className={controlClass}
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    maxLength={100}
                    required
                    autoFocus
                />
            </div>

            <div>
                <label className={labelClass} htmlFor={`${id}-description`}>
                    Description <span className="font-normal text-subtle">(optional)</span>
                </label>
                <textarea
                    id={`${id}-description`}
                    className={`${controlClass} min-h-20 resize-y`}
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                />
            </div>

            <p className="text-meta text-muted">
                {deck.language.name} · {deck.variant.variantName} stays as it is, because the cards in this
                deck depend on it.
            </p>

            <div className="flex justify-end gap-2">
                <Button variant="ghost" onClick={onCancel}>
                    Cancel
                </Button>
                <Button type="submit" loading={busy}>
                    Save changes
                </Button>
            </div>
        </motion.form>
    );
}
