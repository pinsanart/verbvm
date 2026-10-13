import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { pickMediaFiles } from "../api/files";
import { playAudio } from "../api/media";
import {
    addStack,
    addStackMember,
    attachMedia,
    createDraft,
    deleteDraft,
    deleteStack,
    duplicateDraft,
    generateMedia,
    generatePronunciation,
    generateStack,
    getFlashcardForm,
    listDrafts,
    moveDraft,
    publishDrafts,
    removeMedia,
    unstackDraft,
    updateDraft,
} from "../api/flashcards";
import { generatedAudio } from "../lib/media";
import { initialUnlinked, missingTypes, orderedMembers, SHARED_KEY, sharedPhrase } from "../lib/stackFields";
import type { StackData, StackLaneData, TypeFields } from "../lib/stackFields";
import type { Draft, FlashcardForm, MoveDirection } from "../types/FlashcardForm";
import type { LoadStatus } from "./useDecksViewModel";

/** How long typing pauses before a draft is saved. */
const AUTOSAVE_DELAY_MS = 500;

/** How long the "saved" notice stays. */
const NOTICE_DURATION_MS = 4000;

/** Field errors of one draft, by field key. */
export type FieldErrors = Record<string, string>;

/** What the editor lists: a card on its own, or a stack of cards shown as one. */
export type EditorItem =
    | { kind: "card"; draft: Draft }
    | { kind: "stack"; stack: StackData; drafts: Draft[] };

/** The slug that adds a stack instead of a card of one type. */
export const STACK_SLUG = "stack";

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
    /** The cards of a stack whose phrase is their own and no longer follows the stack's. */
    const [unlinked, setUnlinked] = useState<ReadonlySet<string>>(new Set());
    /** Bumped when a card's text was replaced by a generation, so its editors reload. */
    const [revisions, setRevisions] = useState<Record<string, number>>({});
    /** What the last generation of each stack reported, by stack. */
    const [stackNotes, setStackNotes] = useState<Record<string, string[]>>({});
    const [generatingStack, setGeneratingStack] = useState<string | null>(null);
    /** What the last generation for a single card reported (words with no pronunciation). */
    const [warnings, setWarnings] = useState<string[]>([]);
    const { error, busy, run } = useAsyncRunner();

    // Latest drafts and the timers of the saves that have not happened yet.
    const draftsRef = useRef<Draft[]>([]);
    const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());

    /** Keeps the drafts as they are now, without looking at which cards have a phrase of their own. */
    const storeDrafts = useCallback((next: Draft[]) => {
        draftsRef.current = next;
        setDrafts(next);
    }, []);

    /** Takes the list the backend returned. */
    const replaceDrafts = useCallback((next: Draft[]) => {
        storeDrafts(next);
        // A card that came back with a phrase of its own (a reload, a copy of a stack) starts unlinked.
        const stacks = new Map<string, Draft[]>();
        for (const draft of next) {
            if (draft.stackUuid) stacks.set(draft.stackUuid, [...(stacks.get(draft.stackUuid) ?? []), draft]);
        }
        const derived = [...stacks.values()].flatMap((members) => [...initialUnlinked(members)]);
        if (derived.length > 0) setUnlinked((current) => new Set([...current, ...derived]));
    }, [storeDrafts]);

    const bump = useCallback((uuids: string[]) => {
        setRevisions((current) => {
            const next = { ...current };
            for (const uuid of uuids) next[uuid] = (next[uuid] ?? 0) + 1;
            return next;
        });
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
        // While a stack's phrase is typed its cards are updated one after the other, so a card must
        // not be taken for one with its own phrase in between.
        storeDrafts(
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

    /** Adds an empty draft of a type at the end of the list, or a stack of three. */
    const addDraft = (typeSlug: string) =>
        typeSlug === STACK_SLUG ? structural(() => addStack(deckName)) : structural(() => createDraft(deckName, typeSlug));
    /** Duplicating or moving a card of a stack does it to the whole stack. */
    const duplicate = (uuid: string) => structural(() => duplicateDraft(uuid));
    const move = (uuid: string, direction: MoveDirection) => structural(() => moveDraft(uuid, direction));
    /** Deletes only this card. A stack left with fewer than two cards stops being a stack. */
    const remove = (uuid: string) => structural(() => deleteDraft(uuid));
    const takeOutOfStack = (uuid: string) => structural(() => unstackDraft(uuid));
    const addToStack = (stackUuid: string, typeSlug: string) => structural(() => addStackMember(stackUuid, typeSlug));
    const removeStack = (stackUuid: string) => structural(() => deleteStack(stackUuid));

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
        storeDrafts(
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

    /** Replaces a draft with the backend's version, text included, and reloads its editors. */
    const applyDraft = (updated: Draft) => {
        replaceDrafts(draftsRef.current.map((d) => (d.uuid === updated.uuid ? updated : d)));
        bump([updated.uuid]);
    };

    /**
     * Fills an IPA field with the pronunciation of the words of its phrase. The backend reads the
     * saved text, so what was typed is saved first. The text replaces what the field had.
     */
    const generatePronunciationOf = (uuid: string, key: string) =>
        run(async () => {
            setWarnings([]);
            await flush();
            const result = await generatePronunciation(uuid, key);
            applyDraft(result.value);
            setWarnings(result.warnings);
        });

    /** Generates what a field is made of: the pronunciation of an IPA field, the audio of an audio field. */
    const generateField = (uuid: string, key: string) => {
        const draft = draftsRef.current.find((d) => d.uuid === uuid);
        const type = form?.types.find((t) => t.slug === draft?.typeSlug);
        const kind = type?.fields.find((f) => f.key === key)?.kind;
        return kind === "IPA" ? generatePronunciationOf(uuid, key) : generateAudio(uuid, key);
    };

    const detachFile = (uuid: string, key: string, sha256: string) =>
        run(async () => applyMedia(await removeMedia(uuid, key, sha256)));

    /**
     * Makes the audio and the pronunciation of a stack from the phrase of each card. What works is
     * applied and what does not is reported in the stack's notes. The new audio plays at once.
     */
    const generateAll = (stackUuid: string) =>
        run(async () => {
            setGeneratingStack(stackUuid);
            setStackNotes((current) => ({ ...current, [stackUuid]: [] }));
            try {
                await flush();
                const before = draftsRef.current.filter((d) => d.stackUuid === stackUuid);
                const result = await generateStack(stackUuid);
                replaceDrafts(result.value);
                const members = result.value.filter((d) => d.stackUuid === stackUuid);
                bump(members.map((d) => d.uuid));
                setStackNotes((current) => ({ ...current, [stackUuid]: result.warnings }));
                for (const member of members) {
                    const old = before.find((d) => d.uuid === member.uuid);
                    const created = generatedAudio(old?.media.audio, member.media.audio);
                    if (created && member.typeSlug === "orthographic") playAudio(created).catch(() => undefined);
                }
            } finally {
                setGeneratingStack(null);
            }
        });

    /** Types in the phrase of a stack: every card that is still linked gets it. */
    const setStackPhrase = (stackUuid: string, text: string) => {
        for (const draft of draftsRef.current) {
            if (draft.stackUuid === stackUuid && !unlinked.has(draft.uuid)) setValue(draft.uuid, SHARED_KEY, text);
        }
    };

    /** Gives a card its own phrase, or links it back to the stack's (copying the stack's phrase in). */
    const toggleLink = (uuid: string) => {
        const draft = draftsRef.current.find((d) => d.uuid === uuid);
        if (!draft?.stackUuid) return;
        if (unlinked.has(uuid)) {
            const members = draftsRef.current.filter((d) => d.stackUuid === draft.stackUuid && d.uuid !== uuid);
            setValue(uuid, SHARED_KEY, sharedPhrase(orderedMembers(members), unlinked));
            setUnlinked((current) => {
                const next = new Set(current);
                next.delete(uuid);
                return next;
            });
        } else {
            setUnlinked((current) => new Set(current).add(uuid));
        }
    };

    /** The front and back fields of each flashcard type, by slug. */
    const fieldsByType = useMemo(() => {
        const byType: Record<string, TypeFields> = {};
        for (const type of form?.types ?? []) {
            byType[type.slug] = {
                front: type.fields.filter((field) => field.side === "FRONT"),
                back: type.fields.filter((field) => field.side === "BACK"),
            };
        }
        return byType;
    }, [form]);

    /** The list as the editor shows it: stacks as one item, in the order of their first card. */
    const items = useMemo<EditorItem[]>(() => {
        const result: EditorItem[] = [];
        const stacks = new Map<string, Draft[]>();
        for (const draft of drafts) {
            if (!draft.stackUuid) {
                result.push({ kind: "card", draft });
                continue;
            }
            const members = stacks.get(draft.stackUuid);
            if (members) members.push(draft);
            else {
                const created = [draft];
                stacks.set(draft.stackUuid, created);
                result.push({ kind: "stack", stack: { stackUuid: draft.stackUuid, phrase: "", lanes: [], missing: [], generating: false, notes: [] }, drafts: created });
            }
        }
        return result.map((item) => {
            if (item.kind === "card") return item;
            const ordered = orderedMembers(item.drafts);
            const phraseErrors = ordered
                .filter((d) => !unlinked.has(d.uuid))
                .map((d) => errors[d.uuid]?.[SHARED_KEY])
                .filter(Boolean);
            const lanes: StackLaneData[] = ordered.map((draft) => {
                const linked = !unlinked.has(draft.uuid);
                const own = errors[draft.uuid] ?? {};
                const { [SHARED_KEY]: _shared, ...withoutPhrase } = own;
                return {
                    uuid: draft.uuid,
                    revision: revisions[draft.uuid] ?? 0,
                    typeSlug: draft.typeSlug,
                    fields: fieldsByType[draft.typeSlug] ?? { front: [], back: [] },
                    values: draft.values,
                    media: draft.media,
                    // A linked card's phrase error shows on the stack's phrase, not on the card.
                    errors: linked ? withoutPhrase : own,
                    linked,
                };
            });
            return {
                kind: "stack" as const,
                drafts: ordered,
                stack: {
                    stackUuid: item.stack.stackUuid,
                    phrase: sharedPhrase(ordered, unlinked),
                    phraseError: phraseErrors[0],
                    lanes,
                    missing: missingTypes(ordered),
                    generating: generatingStack === item.stack.stackUuid,
                    notes: stackNotes[item.stack.stackUuid] ?? [],
                },
            };
        });
    }, [drafts, unlinked, revisions, errors, fieldsByType, generatingStack, stackNotes]);

    return {
        status,
        form,
        fieldsByType,
        items,
        warnings,
        revisionOf: (uuid: string) => revisions[uuid] ?? 0,
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
        generate: generateField,
        detachFile,
        takeOutOfStack,
        addToStack,
        removeStack,
        setStackPhrase,
        toggleLink,
        generateStack: generateAll,
        saveAll,
    };
}
