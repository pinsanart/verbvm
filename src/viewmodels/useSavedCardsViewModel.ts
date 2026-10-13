import { useCallback, useEffect, useMemo, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { pickMediaFiles } from "../api/files";
import { playAudio } from "../api/media";
import {
    attachSavedCardMedia,
    deleteSavedCard,
    deleteSavedStack,
    generateSavedCardMedia,
    generateSavedCardPronunciation,
    generateSavedStack,
    listSavedCards,
    removeSavedCardMedia,
    unstackSavedCard,
    updateSavedCard,
    updateSavedCards,
} from "../api/flashcards";
import { generatedAudio } from "../lib/media";
import { initialUnlinked, missingTypes, orderedMembers, SHARED_KEY, sharedPhrase } from "../lib/stackFields";
import type { StackData, StackLaneData, TypeFields } from "../lib/stackFields";
import { plainText } from "../lib/text";
import type { SavedCard } from "../types/FlashcardForm";
import type { LoadStatus } from "./useDecksViewModel";

/** What the saved list shows: a card on its own, or the cards of a stack as one row. */
export type SavedRow = { kind: "card"; card: SavedCard } | { kind: "stack"; stackUuid: string; cards: SavedCard[] };

/**
 * The view's API for the saved flashcards of a deck: the list with a search and a type filter,
 * and the panels that edit them.
 *
 * Cards made together (a stack) are one row in the list. Opening the row edits all of them in one
 * panel with one Save, and each card can still be opened and edited alone from its own line. Text
 * is edited locally and saved with an explicit Save, because it changes what the learner studies.
 * Media, generating and deleting apply right away. The backend enforces every rule and returns
 * the saved cards, which replace the ones in the list.
 */
export function useSavedCardsViewModel(deckName: string, fieldsByType: Record<string, TypeFields>) {
    const [cards, setCards] = useState<SavedCard[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const [query, setQuery] = useState("");
    const [typeFilter, setTypeFilter] = useState<string | null>(null);
    const [selectedUuid, setSelectedUuid] = useState<string | null>(null);
    const [values, setValues] = useState<Record<string, string>>({});
    const [justSaved, setJustSaved] = useState(false);
    // The stack being edited, the text of each of its cards as typed, and which cards have their own phrase.
    const [stackUuid, setStackUuid] = useState<string | null>(null);
    const [stackValues, setStackValues] = useState<Record<string, Record<string, string>>>({});
    const [unlinked, setUnlinked] = useState<ReadonlySet<string>>(new Set());
    const [stackNotes, setStackNotes] = useState<string[]>([]);
    const [generatingStack, setGeneratingStack] = useState(false);
    /** Bumped when a card's text was replaced by a generation, so its editors reload. */
    const [revisions, setRevisions] = useState<Record<string, number>>({});
    const { error, busy, run, clearError } = useAsyncRunner();

    const bump = useCallback((uuids: string[]) => {
        setRevisions((current) => {
            const next = { ...current };
            for (const uuid of uuids) next[uuid] = (next[uuid] ?? 0) + 1;
            return next;
        });
    }, []);

    const reload = useCallback(async () => {
        const ok = await run(async () => setCards(await listSavedCards(deckName)));
        setStatus(ok ? "ready" : "error");
    }, [deckName, run]);

    useEffect(() => {
        setStatus("loading");
        setSelectedUuid(null);
        setStackUuid(null);
        reload();
    }, [reload]);

    const selected = useMemo(() => cards.find((c) => c.uuid === selectedUuid) ?? null, [cards, selectedUuid]);
    const stackCards = useMemo(
        () => (stackUuid ? orderedMembers(cards.filter((c) => c.stackUuid === stackUuid)) : []),
        [cards, stackUuid],
    );

    const differs = (saved: Record<string, string>, typed: Record<string, string> | undefined) => {
        if (!typed) return false;
        const keys = new Set([...Object.keys(saved), ...Object.keys(typed)]);
        return [...keys].some((key) => (saved[key] ?? "") !== (typed[key] ?? ""));
    };

    /** Whether the text in the single card panel differs from what is saved. */
    const dirty = useMemo(() => (selected ? differs(selected.values, values) : false), [selected, values]);
    /** The cards of the open stack whose text differs from what is saved. */
    const changedCards = useMemo(
        () => stackCards.filter((card) => differs(card.values, stackValues[card.uuid])),
        [stackCards, stackValues],
    );
    const stackDirty = changedCards.length > 0;

    const replace = (updated: SavedCard) => setCards((current) => current.map((c) => (c.uuid === updated.uuid ? updated : c)));
    const replaceMany = (updated: SavedCard[]) =>
        setCards((current) => current.map((c) => updated.find((u) => u.uuid === c.uuid) ?? c));

    // ---- one card ----

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

    /**
     * Generates what a field is made of: the audio of an audio field or the pronunciation of an
     * IPA field, from the text of its source field. The backend reads the saved text, so the text
     * in the panel is saved first. New audio plays at once.
     */
    const generate = (key: string) =>
        run(async () => {
            if (!selected) return;
            const kind = [...(fieldsByType[selected.typeSlug]?.front ?? []), ...(fieldsByType[selected.typeSlug]?.back ?? [])].find(
                (f) => f.key === key,
            )?.kind;
            replace(await updateSavedCard(selected.uuid, values));
            if (kind === "IPA") {
                const result = await generateSavedCardPronunciation(selected.uuid, key);
                replace(result.value);
                setValues(result.value.values);
                bump([selected.uuid]);
                setStackNotes(result.warnings);
            } else {
                const before = selected.media[key];
                const generated = await generateSavedCardMedia(selected.uuid, key);
                replace(generated);
                setValues(generated.values);
                const created = generatedAudio(before, generated.media[key]);
                if (created) playAudio(created).catch(() => undefined);
            }
            setJustSaved(true);
        });

    const detachFile = (key: string, sha256: string) =>
        run(async () => {
            if (!selected) return;
            replace({ ...(await removeSavedCardMedia(selected.uuid, key, sha256)), values: selected.values });
        });

    /** Deletes the open card. A stack left with fewer than two cards stops being a stack. */
    const remove = () =>
        run(async () => {
            if (!selected) return;
            await deleteSavedCard(selected.uuid);
            setSelectedUuid(null);
            setCards(await listSavedCards(deckName));
        });

    // ---- a stack ----

    const openStack = (uuid: string) => {
        const members = cards.filter((c) => c.stackUuid === uuid);
        if (members.length === 0) return;
        clearError();
        setJustSaved(false);
        setStackNotes([]);
        setStackValues(Object.fromEntries(members.map((c) => [c.uuid, c.values])));
        setUnlinked(initialUnlinked(members));
        setStackUuid(uuid);
    };

    const closeStack = () => {
        clearError();
        setStackUuid(null);
    };

    const setStackValue = (uuid: string, key: string, value: string) => {
        setJustSaved(false);
        setStackValues((current) => ({ ...current, [uuid]: { ...current[uuid], [key]: value } }));
    };

    /** Types in the phrase of the stack: every card that is still linked gets it. */
    const setStackPhrase = (text: string) => {
        setJustSaved(false);
        setStackValues((current) => {
            const next = { ...current };
            for (const card of stackCards) {
                if (!unlinked.has(card.uuid)) next[card.uuid] = { ...next[card.uuid], [SHARED_KEY]: text };
            }
            return next;
        });
    };

    /** Gives a card its own phrase, or links it back to the stack's (copying the stack's phrase in). */
    const toggleLink = (uuid: string) => {
        if (unlinked.has(uuid)) {
            const others = stackCards.filter((c) => c.uuid !== uuid).map((c) => ({ ...c, values: stackValues[c.uuid] ?? c.values }));
            setStackValue(uuid, SHARED_KEY, sharedPhrase(others, unlinked));
            setUnlinked((current) => {
                const next = new Set(current);
                next.delete(uuid);
                return next;
            });
        } else {
            setUnlinked((current) => new Set(current).add(uuid));
        }
    };

    /** Saves what changed in the stack's cards, all or nothing, and returns the saved cards. */
    const saveChanged = async (): Promise<SavedCard[]> => {
        if (changedCards.length === 0) return [];
        const saved = await updateSavedCards(changedCards.map((c) => ({ cardUuid: c.uuid, values: stackValues[c.uuid] })));
        replaceMany(saved);
        setStackValues((current) => ({ ...current, ...Object.fromEntries(saved.map((c) => [c.uuid, c.values])) }));
        return saved;
    };

    const saveStack = () =>
        run(async () => {
            await saveChanged();
            setJustSaved(true);
        });

    /** The audio and the pronunciation of the stack, from the saved phrase of each card. */
    const generateAll = () =>
        run(async () => {
            if (!stackUuid) return;
            setGeneratingStack(true);
            setStackNotes([]);
            try {
                await saveChanged();
                const before = stackCards;
                const result = await generateSavedStack(stackUuid);
                replaceMany(result.value);
                setStackValues((current) => ({ ...current, ...Object.fromEntries(result.value.map((c) => [c.uuid, c.values])) }));
                bump(result.value.map((c) => c.uuid));
                setStackNotes(result.warnings);
                for (const card of result.value) {
                    const old = before.find((c) => c.uuid === card.uuid);
                    const created = generatedAudio(old?.media.audio, card.media.audio);
                    if (created && card.typeSlug === "orthographic") playAudio(created).catch(() => undefined);
                }
                setJustSaved(true);
            } finally {
                setGeneratingStack(false);
            }
        });

    /** Generates one field of one card of the stack (audio or pronunciation), keeping the typed text. */
    const generateField = (uuid: string, key: string) =>
        run(async () => {
            const card = stackCards.find((c) => c.uuid === uuid);
            if (!card) return;
            const kind = [...(fieldsByType[card.typeSlug]?.front ?? []), ...(fieldsByType[card.typeSlug]?.back ?? [])].find(
                (f) => f.key === key,
            )?.kind;
            await saveChanged();
            if (kind === "IPA") {
                const result = await generateSavedCardPronunciation(uuid, key);
                replace(result.value);
                setStackValues((current) => ({ ...current, [uuid]: result.value.values }));
                bump([uuid]);
                setStackNotes(result.warnings);
            } else {
                const generated = await generateSavedCardMedia(uuid, key);
                replace(generated);
                setStackValues((current) => ({ ...current, [uuid]: generated.values }));
                const created = generatedAudio(card.media[key], generated.media[key]);
                if (created) playAudio(created).catch(() => undefined);
            }
        });

    const attachStackFiles = (uuid: string, key: string) =>
        run(async () => {
            for (const path of await pickMediaFiles()) {
                replace(await attachSavedCardMedia(uuid, key, path));
            }
        });

    const detachStackFile = (uuid: string, key: string, sha256: string) =>
        run(async () => replace(await removeSavedCardMedia(uuid, key, sha256)));

    /** After a card left or was deleted: reload, and close the panel when it is no longer a stack. */
    const afterStackChange = async () => {
        const reloaded = await listSavedCards(deckName);
        setCards(reloaded);
        if (stackUuid && reloaded.filter((c) => c.stackUuid === stackUuid).length < 2) setStackUuid(null);
    };

    /** The card keeps existing as a regular card. */
    const takeOut = (uuid: string) =>
        run(async () => {
            await saveChanged();
            await unstackSavedCard(uuid);
            await afterStackChange();
        });

    const deleteStackCard = (uuid: string) =>
        run(async () => {
            await deleteSavedCard(uuid);
            await afterStackChange();
        });

    /** Deletes every card of the open stack. */
    const removeStack = () =>
        run(async () => {
            if (!stackUuid) return;
            await deleteSavedStack(stackUuid);
            setCards((current) => current.filter((c) => c.stackUuid !== stackUuid));
            setStackUuid(null);
        });

    /** The open stack as the shared stack view shows it. */
    const stack = useMemo<StackData | null>(() => {
        if (!stackUuid || stackCards.length === 0) return null;
        const members = stackCards.map((card) => ({ ...card, values: stackValues[card.uuid] ?? card.values }));
        const lanes: StackLaneData[] = members.map((card) => ({
            uuid: card.uuid,
            revision: revisions[card.uuid] ?? 0,
            typeSlug: card.typeSlug,
            fields: fieldsByType[card.typeSlug] ?? { front: [], back: [] },
            values: card.values,
            media: stackCards.find((c) => c.uuid === card.uuid)?.media ?? {},
            errors: {},
            linked: !unlinked.has(card.uuid),
        }));
        return {
            stackUuid,
            phrase: sharedPhrase(members, unlinked),
            lanes,
            missing: [],
            generating: generatingStack,
            notes: stackNotes,
        };
    }, [stackUuid, stackCards, stackValues, unlinked, revisions, fieldsByType, generatingStack, stackNotes]);

    // ---- the list ----

    /** The rows that match the search and the type filter, with a stack's cards together. */
    const rows = useMemo<SavedRow[]>(() => {
        const needle = query.trim().toLowerCase();
        const matches = (members: SavedCard[]) =>
            members.some((card) => {
                if (typeFilter && card.typeSlug !== typeFilter) return false;
                if (!needle) return true;
                return Object.values(card.values).some((value) => plainText(value).toLowerCase().includes(needle));
            });
        const result: SavedRow[] = [];
        const stacks = new Map<string, SavedCard[]>();
        for (const card of cards) {
            if (!card.stackUuid) {
                if (matches([card])) result.push({ kind: "card", card });
                continue;
            }
            const members = stacks.get(card.stackUuid);
            if (members) members.push(card);
            else {
                const created = [card];
                stacks.set(card.stackUuid, created);
                result.push({ kind: "stack", stackUuid: card.stackUuid, cards: created });
            }
        }
        return result
            .filter((row) => row.kind === "card" || matches(row.cards))
            .map((row) => (row.kind === "stack" ? { ...row, cards: orderedMembers(row.cards) } : row));
    }, [cards, query, typeFilter]);

    return {
        status,
        cards,
        rows,
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
        revisionOf: (uuid: string) => revisions[uuid] ?? 0,
        reload,
        open,
        close,
        setValue,
        save,
        attachFiles,
        generate,
        detachFile,
        remove,
        stack,
        stackDirty,
        openStack,
        closeStack,
        setStackValue,
        setStackPhrase,
        toggleLink,
        saveStack,
        generateAll,
        generateField,
        attachStackFiles,
        detachStackFile,
        takeOut,
        deleteStackCard,
        removeStack,
        missingOf: (uuid: string) => missingTypes(cards.filter((c) => c.stackUuid === uuid)),
    };
}
