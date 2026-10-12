import { useCallback, useEffect, useState } from "react";
import {
    createLanguage,
    createLanguageVariant,
    deleteLanguage,
    deleteLanguageVariant,
    errorMessage,
    listLanguages,
    listLanguageVariants,
    setDefaultLanguageVariant,
} from "../api/languages";
import type { Language, LanguageVariant, NewLanguage, NewLanguageVariant } from "../types/Language";

/**
 * The view's API for languages and their variants: state plus actions.
 * Actions resolve to `true` when they succeeded, so the view can reset its forms.
 */
export function useLanguagesViewModel() {
    const [languages, setLanguages] = useState<Language[]>([]);
    const [variants, setVariants] = useState<LanguageVariant[]>([]);
    const [selectedId, setSelectedId] = useState<number | null>(null);
    const [error, setError] = useState<string | null>(null);

    const selected = languages.find((language) => language.id === selectedId) ?? null;

    const run = useCallback(async (action: () => Promise<void>) => {
        try {
            setError(null);
            await action();
            return true;
        } catch (err) {
            setError(errorMessage(err));
            return false;
        }
    }, []);

    const refreshLanguages = useCallback(
        () => run(async () => setLanguages(await listLanguages())),
        [run],
    );

    const refreshVariants = useCallback(
        (languageId: number) => run(async () => setVariants(await listLanguageVariants(languageId))),
        [run],
    );

    useEffect(() => {
        refreshLanguages();
    }, [refreshLanguages]);

    useEffect(() => {
        if (selectedId === null) {
            setVariants([]);
        } else {
            refreshVariants(selectedId);
        }
    }, [selectedId, refreshVariants]);

    const addLanguage = (input: NewLanguage) =>
        run(async () => {
            const created = await createLanguage(input);
            await refreshLanguages();
            setSelectedId(created.id);
        });

    const removeLanguage = (id: number) =>
        run(async () => {
            await deleteLanguage(id);
            if (selectedId === id) setSelectedId(null);
            await refreshLanguages();
        });

    const addVariant = (input: Omit<NewLanguageVariant, "languageId">) => {
        if (selectedId === null) return Promise.resolve(false);
        return run(async () => {
            await createLanguageVariant({ ...input, languageId: selectedId });
            await Promise.all([refreshVariants(selectedId), refreshLanguages()]);
        });
    };

    const makeDefaultVariant = (variantId: number) => {
        if (selectedId === null) return Promise.resolve(false);
        return run(async () => {
            await setDefaultLanguageVariant(selectedId, variantId);
            await refreshLanguages();
        });
    };

    const removeVariant = (variantId: number) => {
        if (selectedId === null) return Promise.resolve(false);
        return run(async () => {
            await deleteLanguageVariant(variantId);
            await Promise.all([refreshVariants(selectedId), refreshLanguages()]);
        });
    };

    return {
        languages,
        variants,
        selected,
        error,
        selectLanguage: setSelectedId,
        addLanguage,
        removeLanguage,
        addVariant,
        makeDefaultVariant,
        removeVariant,
    };
}
