import { FormEvent, useEffect, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Button } from "./Button";
import { Notice } from "./Notice";
import { Select } from "./Select";
import { controlClass, labelClass } from "./formStyles";
import { TRANSITION_BASE } from "../lib/motion";
import type { NewDeck } from "../types/Deck";
import type { Language, LanguageVariant } from "../types/Language";

export interface NewDeckFormProps {
    languages: Language[];
    /** The accents of a language. */
    loadVariants: (languageIso6393: string) => Promise<LanguageVariant[]>;
    /** Resolves to `true` when the deck was created. */
    onCreate: (deck: NewDeck) => Promise<boolean>;
    onCancel: () => void;
    busy: boolean;
    error: string | null;
}

/**
 * The form to create a deck, shown in place on the deck list. Creating a deck needs neither an
 * interruption nor a trapped focus, so it is not a modal. Escape cancels.
 */
export function NewDeckForm({ languages, loadVariants, onCreate, onCancel, busy, error }: NewDeckFormProps) {
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [languageIso, setLanguageIso] = useState(languages[0]?.iso6393 ?? "");
    const [variants, setVariants] = useState<LanguageVariant[]>([]);
    const [variantCode, setVariantCode] = useState("");

    useEffect(() => {
        if (!languageIso) return;
        let cancelled = false;
        loadVariants(languageIso).then((loaded) => {
            if (cancelled) return;
            setVariants(loaded);
            const preferred = languages.find((l) => l.iso6393 === languageIso)?.defaultVariantCode;
            setVariantCode(preferred ?? loaded[0]?.bcp47Code ?? "");
        });
        return () => {
            cancelled = true;
        };
    }, [languageIso, languages, loadVariants]);

    const submit = async (e: FormEvent) => {
        e.preventDefault();
        const created = await onCreate({
            name,
            languageIso6393: languageIso,
            variantBcp47Code: variantCode || null,
            description: description.trim() === "" ? null : description,
        });
        if (created) onCancel();
    };

    return (
        <motion.form
            layout
            aria-labelledby="new-deck-title"
            onSubmit={submit}
            onKeyDown={(e) => {
                if (e.key === "Escape") onCancel();
            }}
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={TRANSITION_BASE}
            className="flex flex-col gap-5 rounded-2xl border border-line-strong bg-panel p-5 md:col-span-2 md:p-6"
        >
            <h2 id="new-deck-title" className="text-heading">
                New deck
            </h2>

            <AnimatePresence initial={false}>
                {error && (
                    <Notice key="error" tone="error">
                        {error}
                    </Notice>
                )}
            </AnimatePresence>

            <div>
                <label className={labelClass} htmlFor="deck-name">
                    Name
                </label>
                <input
                    id="deck-name"
                    className={controlClass}
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    maxLength={100}
                    required
                    autoFocus
                />
            </div>

            <div className="grid gap-5 sm:grid-cols-2">
                <div>
                    <label className={labelClass} htmlFor="deck-language">
                        Language
                    </label>
                    <Select
                        id="deck-language"
                        value={languageIso}
                        onChange={setLanguageIso}
                        options={languages.map((l) => ({ value: l.iso6393, label: l.name }))}
                    />
                </div>
                <div>
                    <label className={labelClass} htmlFor="deck-accent">
                        Accent
                    </label>
                    <Select
                        id="deck-accent"
                        value={variantCode}
                        onChange={setVariantCode}
                        disabled={variants.length === 0}
                        options={variants.map((v) => ({ value: v.bcp47Code, label: v.variantName }))}
                    />
                </div>
            </div>

            <div>
                <label className={labelClass} htmlFor="deck-description">
                    Description <span className="font-normal text-subtle">(optional)</span>
                </label>
                <textarea
                    id="deck-description"
                    className={`${controlClass} min-h-20 resize-y`}
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                />
            </div>

            <div className="flex justify-end gap-2">
                <Button variant="ghost" onClick={onCancel}>
                    Cancel
                </Button>
                <Button type="submit" loading={busy} disabled={languages.length === 0}>
                    Create deck
                </Button>
            </div>
        </motion.form>
    );
}
