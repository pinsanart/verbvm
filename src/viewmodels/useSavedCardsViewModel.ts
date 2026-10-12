import { useCallback, useEffect, useMemo, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { pickMediaFiles } from "../api/files";
import {
    attachSavedCardMedia,
    deleteSavedCard,
    listSavedCards,
    removeSavedCardMedia,
    updateSavedCard,
} from "../api/flashcards";
import { plainText } from "../lib/text";
import type { SavedCard } from "../types/FlashcardForm";
import type { LoadStatus } from "./useDecksViewModel";

/**
 * The view's API for the saved flashcards of a deck: the list with a search and a type filter,
 * and the edit panel for one card.
 *
 * Text is edited locally and saved with an explicit Save, because it changes what the learner
 * studies. Media and deletion apply right away. The backend enforces every rule and returns the
 * saved card, which replaces the one in the list.
 */
export function useSavedCardsViewModel(deckName: string) {
    const [cards, setCards] = useState<SavedCard[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const [query, setQuery] = useState("");
    const [typeFilter, setTypeFilter] = useState<string | null>(null);
    const [selectedUuid, setSelectedUuid] = useState<string | null>(null);
    const [values, setValues] = useState<Record<string, string>>({});
    const [justSaved, setJustSaved] = useState(false);
    const { error, busy, run, clearError } = useAsyncRunner();

    const reload = useCallback(async () => {
        const ok = await run(async () => setCards(await listSavedCards(deckName)));
        setStatus(ok ? "ready" : "error");
    }, [deckName, run]);

    useEffect(() => {
        setStatus("loading");
        setSelectedUuid(null);
        reload();
    }, [reload]);

    const selected = useMemo(() => cards.find((c) => c.uuid === selectedUuid) ?? null, [cards, selectedUuid]);

    /** Whether the text in the panel differs from what is saved. */
    const dirty = useMemo(() => {
        if (!selected) return false;
        const keys = new Set([...Object.keys(selected.values), ...Object.keys(values)]);
        return [...keys].some((key) => (selected.values[key] ?? "") !== (values[key] ?? ""));
    }, [selected, values]);

    const replace = (updated: SavedCard) => setCards((current) => current.map((c) => (c.uuid === updated.uuid ? updated : c)));

    const open = (uuid: string) => {
        const card = cards.find((c) => c.uuid === uuid);
        if (!card) return;
        clearError();
        setJustSaved(false);
        setValues(card.values);
        setSelectedUuid(uuid);
    };

    const close = () => {
        clearError();
        setSelectedUuid(null);
    };

    const setValue = (key: string, value: string) => {
        setJustSaved(false);
        setValues((current) => ({ ...current, [key]: value }));
    };

    const save = () =>
        run(async () => {
            if (!selected) return;
            const updated = await updateSavedCard(selected.uuid, values);
            replace(updated);
            setValues(updated.values);
            setJustSaved(true);
        });

    /** Opens the file picker and attaches the chosen files. What was typed stays as it is. */
    const attachFiles = (key: string) =>
        run(async () => {
            if (!selected) return;
            for (const path of await pickMediaFiles()) {
                replace({ ...(await attachSavedCardMedia(selected.uuid, key, path)), values: selected.values });
            }
        });

    const detachFile = (key: string, sha256: string) =>
        run(async () => {
            if (!selected) return;
            replace({ ...(await removeSavedCardMedia(selected.uuid, key, sha256)), values: selected.values });
        });

    const remove = () =>
        run(async () => {
            if (!selected) return;
            await deleteSavedCard(selected.uuid);
            setCards((current) => current.filter((c) => c.uuid !== selected.uuid));
            setSelectedUuid(null);
        });

    /** The cards that match the search and the type filter. */
    const visible = useMemo(() => {
        const needle = query.trim().toLowerCase();
        return cards.filter((card) => {
            if (typeFilter && card.typeSlug !== typeFilter) return false;
            if (!needle) return true;
            return Object.values(card.values).some((value) => plainText(value).toLowerCase().includes(needle));
        });
    }, [cards, query, typeFilter]);

    return {
        status,
        cards,
        visible,
        query,
        setQuery,
        typeFilter,
        setTypeFilter,
        selected,
        values,
        dirty,
        justSaved,
        error,
        busy,
        reload,
        open,
        close,
        setValue,
        save,
        attachFiles,
        detachFile,
        remove,
    };
}
