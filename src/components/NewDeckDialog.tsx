import { FormEvent, useEffect, useState } from "react";
import { Select } from "./Select";
import { controlClass, labelClass } from "./formStyles";
import type { NewDeck } from "../types/Deck";
import type { Language, LanguageVariant } from "../types/Language";

export interface NewDeckDialogProps {
    languages: Language[];
    /** The accents of a language. */
    loadVariants: (languageIso6393: string) => Promise<LanguageVariant[]>;
    /** Resolves to `true` when the deck was created. */
    onCreate: (deck: NewDeck) => Promise<boolean>;
    onClose: () => void;
    busy: boolean;
    error: string | null;
}

/** A dialog to create a deck: a name, a language and its accent, and a description. */
export function NewDeckDialog({ languages, loadVariants, onCreate, onClose, busy, error }: NewDeckDialogProps) {
    const [name, setName] = useState("");
    const [description, setDescription] = useState("");
    const [languageIso, setLanguageIso] = useState(languages[0]?.iso6393 ?? "");
    const [variants, setVariants] = useState<LanguageVariant[]>([]);
    const [variantCode, setVariantCode] = useState("");

    const language = languages.find((l) => l.iso6393 === languageIso);

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
        if (created) onClose();
    };

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" onClick={onClose}>
            <form
                role="dialog"
                aria-modal="true"
                aria-label="New deck"
                onSubmit={submit}
                onKeyDown={(e) => {
                    if (e.key === "Escape") onClose();
                }}
                onClick={(e) => e.stopPropagation()}
                className="flex w-full max-w-md flex-col gap-4 rounded-2xl border border-white/10 bg-panel p-5 shadow-xl"
            >
                <h2 className="text-lg font-semibold">New deck</h2>

                {error && (
                    <div role="alert" className="rounded-lg border border-stack/70 px-3 py-2 text-sm text-stack">
                        {error}
                    </div>
                )}

                <div>
                    <label className={labelClass} htmlFor="deck-name">Name</label>
                    <input id="deck-name" className={controlClass} value={name} onChange={(e) => setName(e.target.value)}
                        maxLength={100} required autoFocus />
                </div>

                <div className="grid grid-cols-2 gap-3">
                    <div>
                        <label className={labelClass} htmlFor="deck-language">Language</label>
                        <Select id="deck-language" value={languageIso} onChange={setLanguageIso}
                            options={languages.map((l) => ({ value: l.iso6393, label: l.name }))} />
                    </div>
                    <div>
                        <label className={labelClass} htmlFor="deck-accent">Accent</label>
                        <Select id="deck-accent" value={variantCode} onChange={setVariantCode}
                            disabled={variants.length === 0}
                            options={variants.map((v) => ({ value: v.bcp47Code, label: v.variantName }))} />
                    </div>
                </div>

                <div>
                    <label className={labelClass} htmlFor="deck-description">Description</label>
                    <textarea id="deck-description" className={`${controlClass} min-h-20 resize-y`} value={description}
                        onChange={(e) => setDescription(e.target.value)} />
                </div>

                <div className="flex justify-end gap-2">
                    <button type="button" onClick={onClose}
                        className="rounded-lg px-3 py-2 text-sm text-white/60 hover:bg-white/10 hover:text-white">
                        Cancel
                    </button>
                    <button type="submit" disabled={busy || !language}
                        className="rounded-lg bg-vocabulary px-4 py-2 text-sm font-medium disabled:opacity-50">
                        Create deck
                    </button>
                </div>
            </form>
        </div>
    );
}
