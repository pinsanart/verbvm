import { invoke } from "@tauri-apps/api/core";
import { MediaDto, toMedia } from "./media";
import type { Draft, FlashcardForm, MoveDirection, PublishResult } from "../types/FlashcardForm";

interface DraftDto {
    uuid: string;
    typeSlug: string;
    position: number;
    values: Record<string, string>;
    media: Record<string, MediaDto[]>;
}

const toDraft = (dto: DraftDto): Draft => ({
    uuid: dto.uuid,
    typeSlug: dto.typeSlug,
    position: dto.position,
    values: dto.values,
    media: Object.fromEntries(Object.entries(dto.media).map(([key, items]) => [key, items.map(toMedia)])),
});

export const getFlashcardForm = (deckName: string) =>
    invoke<FlashcardForm>("get_flashcard_form", { deckName });

/** The deck's drafts, of every type, in order. */
export const listDrafts = async (deckName: string) =>
    (await invoke<DraftDto[]>("list_drafts", { deckName })).map(toDraft);

/** Adds an empty draft of a type at the end of the deck's list and returns the updated list. */
export const createDraft = async (deckName: string, typeSlug: string) =>
    (await invoke<DraftDto[]>("create_draft", { deckName, typeSlug })).map(toDraft);

export const updateDraft = async (draftUuid: string, values: Record<string, string>) =>
    toDraft(await invoke<DraftDto>("update_draft", { draftUuid, values }));

export const duplicateDraft = async (draftUuid: string) =>
    (await invoke<DraftDto[]>("duplicate_draft", { draftUuid })).map(toDraft);

export const moveDraft = async (draftUuid: string, direction: MoveDirection) =>
    (await invoke<DraftDto[]>("move_draft", { draftUuid, direction })).map(toDraft);

export const deleteDraft = async (draftUuid: string) =>
    (await invoke<DraftDto[]>("delete_draft", { draftUuid })).map(toDraft);

/** Copies the file at `sourcePath` into the app's media folder and attaches it to a field. */
export const attachMedia = async (draftUuid: string, fieldKey: string, sourcePath: string) =>
    toDraft(await invoke<DraftDto>("attach_media", { draftUuid, fieldKey, sourcePath }));

export const removeMedia = async (draftUuid: string, fieldKey: string, sha256: string) =>
    toDraft(await invoke<DraftDto>("remove_media", { draftUuid, fieldKey, sha256 }));

/** Saves every draft of the deck as a flashcard of its own type. */
export const publishDrafts = (deckName: string) => invoke<PublishResult>("publish_drafts", { deckName });
