import { invoke } from "@tauri-apps/api/core";
import type { Draft, FlashcardForm, MoveDirection, PublishResult } from "../types/FlashcardForm";

export const getFlashcardForm = (deckName: string, typeSlug: string) =>
    invoke<FlashcardForm>("get_flashcard_form", { deckName, typeSlug });

export const listDrafts = (deckName: string, typeSlug: string) =>
    invoke<Draft[]>("list_drafts", { deckName, typeSlug });

/** Adds an empty draft and returns the updated list. */
export const createDraft = (deckName: string, typeSlug: string) =>
    invoke<Draft[]>("create_draft", { deckName, typeSlug });

export const updateDraft = (draftUuid: string, values: Record<string, string>) =>
    invoke<Draft>("update_draft", { draftUuid, values });

export const duplicateDraft = (draftUuid: string) => invoke<Draft[]>("duplicate_draft", { draftUuid });

export const moveDraft = (draftUuid: string, direction: MoveDirection) =>
    invoke<Draft[]>("move_draft", { draftUuid, direction });

export const deleteDraft = (draftUuid: string) => invoke<Draft[]>("delete_draft", { draftUuid });

export const publishDrafts = (deckName: string, typeSlug: string) =>
    invoke<PublishResult>("publish_drafts", { deckName, typeSlug });
