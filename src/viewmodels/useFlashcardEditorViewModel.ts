import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import {
    createDraft,
    deleteDraft,
    duplicateDraft,
    getFlashcardForm,
    listDrafts,
    moveDraft,
    publishDrafts,
    updateDraft,
} from "../api/flashcards";
import type { Draft, FieldDefinition, FlashcardForm, MoveDirection } from "../types/FlashcardForm";
import type { LoadStatus } from "./useDecksViewModel";

/** How long typing pauses before a draft is saved. */
const AUTOSAVE_DELAY_MS = 500;

/** Field errors of one draft, by field key. */
export type FieldErrors = Record<string, string>;

/**
 * The view's API for editing the drafts of a flashcard type in a deck.
 *
 * The backend is the source of truth. Add, move, duplicate and delete replace the list with the
 * one the backend returns. Typing is applied locally right away and saved after a short pause, and
 * pending saves are flushed before any other action and when the screen closes. The backend
 * enforces every rule (required fields, sanitizing). This only shows what it reports.
 */
export function useFlashcardEditorViewModel(deckName: string, typeSlug: string) {
    const [form, setForm] = useState<FlashcardForm | null>(null);
    const [drafts, setDrafts] = useState<Draft[]>([]);
    const [status, setStatus] = useState<LoadStatus>("loading");
    const [errors, setErrors] = useState<Record<string, FieldErrors>>({});
    const [lastPublished, setLastPublished] = useState<number | null>(null);
    const { error, busy, run } = useAsyncRunner();

    // Latest drafts and the timers of the saves that have not happened yet.
    const draftsRef = useRef<Draft[]>([]);
    const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());

    const replaceDrafts = useCallback((next: Draft[]) => {
        draftsRef.current = next;
        setDrafts(next);
    }, []);

    const saveNow = useCallback(async (uuid: string) => {
        const timer = timers.current.get(uuid);
        if (timer !== undefined) clearTimeout(timer);
        timers.current.delete(uuid);
        const draft = draftsRef.current.find((d) => d.uuid === uuid);
        if (draft) await updateDraft(uuid, draft.values);
    }, []);

    /** Saves every draft whose typing has not been saved yet. */
    const flush = useCallback(async () => {
        await Promise.all([...timers.current.keys()].map(saveNow));
    }, [saveNow]);

    useEffect(() => {
        let cancelled = false;
        setStatus("loading");
        (async () => {
            const ok = await run(async () => {
                const loadedForm = await getFlashcardForm(deckName, typeSlug);
                let loaded = await listDrafts(deckName, typeSlug);
                if (loaded.length === 0) loaded = await createDraft(deckName, typeSlug);
                if (cancelled) return;
                setForm(loadedForm);
                replaceDrafts(loaded);
            });
            if (!cancelled) setStatus(ok ? "ready" : "error");
        })();
        return () => {
            cancelled = true;
            flush().catch(() => undefined);
        };
    }, [deckName, typeSlug, run, replaceDrafts, flush]);

    const setValue = (uuid: string, key: string, value: string) => {
        replaceDrafts(
            draftsRef.current.map((d) =>
                d.uuid === uuid ? { ...d, values: { ...d.values, [key]: value } } : d,
            ),
        );
        setLastPublished(null);
        setErrors((current) => {
            if (!current[uuid]?.[key]) return current;
            const { [key]: _removed, ...rest } = current[uuid];
            return { ...current, [uuid]: rest };
        });

        const pending = timers.current.get(uuid);
        if (pending !== undefined) clearTimeout(pending);
        timers.current.set(
            uuid,
            setTimeout(() => {
                saveNow(uuid).catch(() => undefined);
            }, AUTOSAVE_DELAY_MS),
        );
    };

    /** Flushes pending saves, runs a backend action that returns the new list, and applies it. */
    const structural = (action: () => Promise<Draft[]>) =>
        run(async () => {
            await flush();
            replaceDrafts(await action());
        });

    const addDraft = () => structural(() => createDraft(deckName, typeSlug));
    const duplicate = (uuid: string) => structural(() => duplicateDraft(uuid));
    const move = (uuid: string, direction: MoveDirection) => structural(() => moveDraft(uuid, direction));
    const remove = (uuid: string) => structural(() => deleteDraft(uuid));

    /** Saves every draft as a flashcard. When the backend reports problems nothing is saved. */
    const saveAll = () =>
        run(async () => {
            await flush();
            const result = await publishDrafts(deckName, typeSlug);
            const grouped: Record<string, FieldErrors> = {};
            for (const issue of result.issues) {
                (grouped[issue.draftUuid] ??= {})[issue.fieldKey ?? ""] = issue.message;
            }
            setErrors(grouped);
            if (result.issues.length > 0) return;

            setLastPublished(result.published);
            const remaining = await listDrafts(deckName, typeSlug);
            replaceDrafts(remaining.length > 0 ? remaining : await createDraft(deckName, typeSlug));
        });

    const fieldsBySide = useMemo(() => {
        const bySide = (side: FieldDefinition["side"]) =>
            (form?.fields ?? []).filter((field) => field.side === side);
        return { front: bySide("FRONT"), back: bySide("BACK") };
    }, [form]);

    return {
        status,
        form,
        fieldsBySide,
        drafts,
        errorsFor: (uuid: string): FieldErrors => errors[uuid] ?? {},
        error,
        busy,
        lastPublished,
        addDraft,
        setValue,
        duplicate,
        move,
        remove,
        saveAll,
    };
}
