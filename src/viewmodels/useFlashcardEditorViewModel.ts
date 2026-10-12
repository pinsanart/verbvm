import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { pickMediaFiles } from "../api/files";
import { playAudio } from "../api/media";
import {
    attachMedia,
    createDraft,
    deleteDraft,
    duplicateDraft,
    generateMedia,
    getFlashcardForm,
    listDrafts,
    moveDraft,
    publishDrafts,
    removeMedia,
    updateDraft,
} from "../api/flashcards";
import { generatedAudio } from "../lib/media";
import type { Draft, FieldDefinition, FlashcardForm, MoveDirection } from "../types/FlashcardForm";
import type { LoadStatus } from "./useDecksViewModel";

/** How long typing pauses before a draft is saved. */
const AUTOSAVE_DELAY_MS = 500;

/** How long the "saved" notice stays. */
const NOTICE_DURATION_MS = 4000;

/** Field errors of one draft, by field key. */
export type FieldErrors = Record<string, string>;

/**
 * The view's API for editing the drafts of a deck. A deck has one list of drafts that mixes
 * flashcard types.
 *
 * The backend is the source of truth. Add, move, duplicate and delete replace the list with the
 * one the backend returns. Typing is applied locally right away and saved after a short pause, and
 * pending saves are flushed before any other action and when the screen closes. The backend
 * enforces every rule (required fields, sanitizing). This only shows what it reports.
 */
export function useFlashcardEditorViewModel(deckName: string) {
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
                const loadedForm = await getFlashcardForm(deckName);
                const loaded = await listDrafts(deckName);
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
    }, [deckName, run, replaceDrafts, flush]);

    // The "saved" notice goes away by itself.
    useEffect(() => {
        if (lastPublished === null) return;
        const timer = setTimeout(() => setLastPublished(null), NOTICE_DURATION_MS);
        return () => clearTimeout(timer);
    }, [lastPublished]);

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

    /** Adds an empty draft of a type at the end of the list. */
    const addDraft = (typeSlug: string) => structural(() => createDraft(deckName, typeSlug));
    const duplicate = (uuid: string) => structural(() => duplicateDraft(uuid));
    const move = (uuid: string, direction: MoveDirection) => structural(() => moveDraft(uuid, direction));
    const remove = (uuid: string) => structural(() => deleteDraft(uuid));

    /** Saves every draft as a flashcard. When the backend reports problems nothing is saved. */
    const saveAll = () =>
        run(async () => {
            await flush();
            const result = await publishDrafts(deckName);
            const grouped: Record<string, FieldErrors> = {};
            for (const issue of result.issues) {
                (grouped[issue.draftUuid] ??= {})[issue.fieldKey ?? ""] = issue.message;
            }
            setErrors(grouped);
            if (result.issues.length > 0) return;

            setLastPublished(result.published);
            replaceDrafts(await listDrafts(deckName));
        });

    /** Takes the media of a draft from the backend and leaves what was typed untouched. */
    const applyMedia = (updated: Draft) =>
        replaceDrafts(
            draftsRef.current.map((d) => (d.uuid === updated.uuid ? { ...d, media: updated.media } : d)),
        );

    /** Opens the file picker and attaches the chosen files to a field of a draft. */
    const attachFiles = (uuid: string, key: string) =>
        run(async () => {
            for (const path of await pickMediaFiles()) {
                applyMedia(await attachMedia(uuid, key, path));
            }
        });

    /**
     * Generates the audio of an audio field from the text of its source field. The backend reads
     * the saved text, so what was typed is saved first. The new audio plays at once.
     */
    const generateAudio = (uuid: string, key: string) =>
        run(async () => {
            await flush();
            const before = draftsRef.current.find((d) => d.uuid === uuid)?.media[key];
            const updated = await generateMedia(uuid, key);
            applyMedia(updated);
            const created = generatedAudio(before, updated.media[key]);
            if (created) playAudio(created).catch(() => undefined);
        });

    const detachFile = (uuid: string, key: string, sha256: string) =>
        run(async () => applyMedia(await removeMedia(uuid, key, sha256)));

    /** The front and back fields of each flashcard type, by slug. */
    const fieldsByType = useMemo(() => {
        const byType: Record<string, { front: FieldDefinition[]; back: FieldDefinition[] }> = {};
        for (const type of form?.types ?? []) {
            byType[type.slug] = {
                front: type.fields.filter((field) => field.side === "FRONT"),
                back: type.fields.filter((field) => field.side === "BACK"),
            };
        }
        return byType;
    }, [form]);

    return {
        status,
        form,
        fieldsByType,
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
        attachFiles,
        generateAudio,
        detachFile,
        saveAll,
    };
}
