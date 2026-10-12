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
    const [selectedIso, setSelectedIso] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);

    const selected = languages.find((language) => language.iso6393 === selectedIso) ?? null;

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
        (languageIso: string) => run(async () => setVariants(await listLanguageVariants(languageIso))),
        [run],
    );

    useEffect(() => {
        refreshLanguages();
    }, [refreshLanguages]);

    useEffect(() => {
        if (selectedIso === null) {
            setVariants([]);
        } else {
            refreshVariants(selectedIso);
        }
    }, [selectedIso, refreshVariants]);

    const addLanguage = (input: NewLanguage) =>
        run(async () => {
            const created = await createLanguage(input);
            await refreshLanguages();
            setSelectedIso(created.iso6393);
        });

    const removeLanguage = (iso: string) =>
        run(async () => {
            await deleteLanguage(iso);
            if (selectedIso === iso) setSelectedIso(null);
            await refreshLanguages();
        });

    const addVariant = (input: Omit<NewLanguageVariant, "languageIso6393">) => {
        if (selectedIso === null) return Promise.resolve(false);
        return run(async () => {
            await createLanguageVariant({ ...input, languageIso6393: selectedIso });
            await Promise.all([refreshVariants(selectedIso), refreshLanguages()]);
        });
    };

    const makeDefaultVariant = (bcp47Code: string) => {
        if (selectedIso === null) return Promise.resolve(false);
        return run(async () => {
            await setDefaultLanguageVariant(selectedIso, bcp47Code);
            await refreshLanguages();
        });
    };

    const removeVariant = (bcp47Code: string) => {
        if (selectedIso === null) return Promise.resolve(false);
        return run(async () => {
            await deleteLanguageVariant(bcp47Code);
            await Promise.all([refreshVariants(selectedIso), refreshLanguages()]);
        });
    };

    return {
        languages,
        variants,
        selected,
        error,
        selectLanguage: setSelectedIso,
        addLanguage,
        removeLanguage,
        addVariant,
        makeDefaultVariant,
        removeVariant,
    };
}
