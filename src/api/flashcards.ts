import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { Draft, FlashcardForm, MoveDirection, PublishResult } from "../types/FlashcardForm";
import type { Media, MediaKind } from "../types/Media";

// What the backend sends: a media file comes with its path on disk.
interface MediaDto {
    sha256: string;
    kind: MediaKind;
    mimeType: string;
    path: string | null;
}

interface DraftDto {
    uuid: string;
    position: number;
    values: Record<string, string>;
    media: Record<string, MediaDto[]>;
}

// The webview cannot read a path directly, so it is turned into an asset protocol URL. The
// backend only allows that protocol for its own media folder.
const toMedia = (dto: MediaDto): Media => ({
    sha256: dto.sha256,
    kind: dto.kind,
    mimeType: dto.mimeType,
    url: dto.path ? convertFileSrc(dto.path) : null,
});

const toDraft = (dto: DraftDto): Draft => ({
    uuid: dto.uuid,
    position: dto.position,
    values: dto.values,
    media: Object.fromEntries(Object.entries(dto.media).map(([key, items]) => [key, items.map(toMedia)])),
});

export const getFlashcardForm = (deckName: string, typeSlug: string) =>
    invoke<FlashcardForm>("get_flashcard_form", { deckName, typeSlug });

export const listDrafts = async (deckName: string, typeSlug: string) =>
    (await invoke<DraftDto[]>("list_drafts", { deckName, typeSlug })).map(toDraft);

/** Adds an empty draft and returns the updated list. */
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

export const publishDrafts = (deckName: string, typeSlug: string) =>
    invoke<PublishResult>("publish_drafts", { deckName, typeSlug });
