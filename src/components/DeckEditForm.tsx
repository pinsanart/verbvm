import { FormEvent, useId, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Button } from "./Button";
import { DeckSettingsFields } from "./DeckSettingsFields";
import { Notice } from "./Notice";
import { controlClass, labelClass } from "./formStyles";
import { fromDraft, toDraft } from "../lib/deckSettings";
import { TRANSITION_BASE } from "../lib/motion";
import { useLanguageNames } from "../i18n/useLanguageNames";
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
 * Edits a deck in the place where its card was. The name, the description and the study options
 * can change: the language and accent are fixed, because the fields of its flashcards depend on
 * them.
 */
export function DeckEditForm({ deck, onSave, onCancel, busy, error }: DeckEditFormProps) {
    const { t } = useTranslation();
    const names = useLanguageNames();
    const id = useId();
    const [name, setName] = useState(deck.name);
    const [description, setDescription] = useState(deck.description ?? "");
    const [settings, setSettings] = useState(toDraft(deck));

    const submit = async (e: FormEvent) => {
        e.preventDefault();
        const saved = await onSave({
            name,
            description: description.trim() === "" ? null : description,
            ...fromDraft(settings),
        });
        if (saved) onCancel();
    };

    return (
        <motion.form
            layout
            aria-label={t("decks.editAria", { name: deck.name })}
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
            <h2 className="text-heading">{t("decks.editTitle")}</h2>

            <AnimatePresence initial={false}>
                {error && (
                    <Notice key="error" tone="error">
                        {error}
                    </Notice>
                )}
            </AnimatePresence>

            <div>
                <label className={labelClass} htmlFor={`${id}-name`}>
                    {t("common.name")}
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
                    {t("common.description")} <span className="font-normal text-subtle">{t("common.optional")}</span>
                </label>
                <textarea
                    id={`${id}-description`}
                    className={`${controlClass} min-h-20 resize-y`}
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                />
            </div>

            <DeckSettingsFields value={settings} onChange={setSettings} />

            <p className="text-meta text-muted">
                {t("decks.fixedNote", { language: names.language(deck.variant.bcp47Code, deck.language.name), accent: names.variant(deck.variant) })}
            </p>

            <div className="flex justify-end gap-2">
                <Button variant="ghost" onClick={onCancel}>
                    {t("common.cancel")}
                </Button>
                <Button type="submit" loading={busy}>
                    {t("common.saveChanges")}
                </Button>
            </div>
        </motion.form>
    );
}
