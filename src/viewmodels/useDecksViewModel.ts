import { useCallback, useEffect, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { createDeck, deleteDeck, listDecks, updateDeck } from "../api/decks";
import { listLanguages, listLanguageVariants } from "../api/languages";
import type { Deck, NewDeck, UpdateDeck } from "../types/Deck";
import type { Language, LanguageVariant } from "../types/Language";

export type LoadStatus = "loading" | "ready" | "error";

/**
 * The view's API for the deck list: the decks, the languages a new deck can use, and the actions
 * to create and delete decks. Actions resolve to `true` when they succeeded.
 */
export function useDecksViewModel() {
    const [decks, setDecks] = useState<Deck[]>([]);
    const [languages, setLanguages] = useState<Language[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const { error, busy, run } = useAsyncRunner();

    const refresh = useCallback(async () => {
        const ok = await run(async () => {
            const [loadedDecks, loadedLanguages] = await Promise.all([listDecks(), listLanguages()]);
            setDecks(loadedDecks);
            setLanguages(loadedLanguages);
        });
        setStatus(ok ? "ready" : "error");
    }, [run]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    const addDeck = (input: NewDeck) =>
        run(async () => {
            await createDeck(input);
            setDecks(await listDecks());
        });

    /** Renames a deck and replaces its description. */
    const editDeck = (name: string, input: UpdateDeck) =>
        run(async () => {
            await updateDeck(name, input);
            setDecks(await listDecks());
        });

    const removeDeck = (name: string) =>
        run(async () => {
            await deleteDeck(name);
            setDecks(await listDecks());
        });

    /** The accents of a language, or an empty list if they could not be loaded. */
    const loadVariants = useCallback(
        async (languageIso6393: string): Promise<LanguageVariant[]> => {
            let variants: LanguageVariant[] = [];
            await run(async () => {
                variants = await listLanguageVariants(languageIso6393);
            });
            return variants;
        },
        [run],
    );

    return { decks, languages, status, error, busy, addDeck, editDeck, removeDeck, loadVariants, refresh };
}
