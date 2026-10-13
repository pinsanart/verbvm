import { invoke } from "@tauri-apps/api/core";
import { MediaDto, toMedia } from "./media";
import type { Draft, FlashcardForm, Generated, MoveDirection, PublishResult, SavedCard } from "../types/FlashcardForm";

interface DraftDto {
    uuid: string;
    typeSlug: string;
    position: number;
    values: Record<string, string>;
    media: Record<string, MediaDto[]>;
    stackUuid: string | null;
}

const toDraft = (dto: DraftDto): Draft => ({
    uuid: dto.uuid,
    typeSlug: dto.typeSlug,
    position: dto.position,
    stackUuid: dto.stackUuid,
    values: dto.values,
    media: Object.fromEntries(Object.entries(dto.media).map(([key, items]) => [key, items.map(toMedia)])),
});

interface SavedCardDto extends Omit<SavedCard, "media"> {
    media: Record<string, MediaDto[]>;
}

const toSavedCard = (dto: SavedCardDto): SavedCard => ({
    ...dto,
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

/** Speaks the text of the field an audio field comes from and attaches the audio to it. */
export const generateMedia = async (draftUuid: string, fieldKey: string) =>
    toDraft(await invoke<DraftDto>("generate_media", { draftUuid, fieldKey }));

/**
 * Fills an IPA field with the pronunciation of the words of the field it is made from. The text
 * is the one saved with the draft. The warnings name the words that had no pronunciation.
 */
export const generatePronunciation = async (draftUuid: string, fieldKey: string): Promise<Generated<Draft>> => {
    const result = await invoke<{ draft: DraftDto; warnings: string[] }>("generate_pronunciation", { draftUuid, fieldKey });
    return { value: toDraft(result.draft), warnings: result.warnings };
};

/** Adds a stack of drafts (vocabulary, phonetic and orthographic) and returns the deck's list. */
export const addStack = async (deckName: string) => (await invoke<DraftDto[]>("add_stack", { deckName })).map(toDraft);

/** Adds a card of a type the stack does not have, with the shared phrase copied in. */
export const addStackMember = async (stackUuid: string, typeSlug: string) =>
    (await invoke<DraftDto[]>("add_stack_member", { stackUuid, typeSlug })).map(toDraft);

/** Takes a draft out of its stack. It becomes a regular draft. */
export const unstackDraft = async (draftUuid: string) =>
    (await invoke<DraftDto[]>("unstack_draft", { draftUuid })).map(toDraft);

/** Deletes every draft of a stack. */
export const deleteStack = async (stackUuid: string) =>
    (await invoke<DraftDto[]>("delete_stack", { stackUuid })).map(toDraft);

/**
 * Generates the audio and the pronunciation of a stack, each from the phrase saved with its own
 * card. What works is applied, and what does not is in the warnings. Returns the deck's list.
 */
export const generateStack = async (stackUuid: string): Promise<Generated<Draft[]>> => {
    const result = await invoke<{ drafts: DraftDto[]; warnings: string[] }>("generate_stack", { stackUuid });
    return { value: result.drafts.map(toDraft), warnings: result.warnings };
};

export const removeMedia = async (draftUuid: string, fieldKey: string, sha256: string) =>
    toDraft(await invoke<DraftDto>("remove_media", { draftUuid, fieldKey, sha256 }));

/** Saves every draft of the deck as a flashcard of its own type. */
export const publishDrafts = (deckName: string) => invoke<PublishResult>("publish_drafts", { deckName });

/** The saved flashcards of a deck, newest first. */
export const listSavedCards = async (deckName: string) =>
    (await invoke<SavedCardDto[]>("list_saved_cards", { deckName })).map(toSavedCard);

/** Replaces the text of a saved flashcard. Fields left out are cleared. */
export const updateSavedCard = async (cardUuid: string, values: Record<string, string>) =>
    toSavedCard(await invoke<SavedCardDto>("update_saved_card", { cardUuid, values }));

export const attachSavedCardMedia = async (cardUuid: string, fieldKey: string, sourcePath: string) =>
    toSavedCard(await invoke<SavedCardDto>("attach_saved_card_media", { cardUuid, fieldKey, sourcePath }));

export const generateSavedCardMedia = async (cardUuid: string, fieldKey: string) =>
    toSavedCard(await invoke<SavedCardDto>("generate_saved_card_media", { cardUuid, fieldKey }));

/** Replaces the text of several saved cards at once (a stack). Nothing is saved unless all pass. */
export const updateSavedCards = async (updates: { cardUuid: string; values: Record<string, string> }[]) =>
    (await invoke<SavedCardDto[]>("update_saved_cards", { updates })).map(toSavedCard);

export const generateSavedCardPronunciation = async (cardUuid: string, fieldKey: string): Promise<Generated<SavedCard>> => {
    const result = await invoke<{ card: SavedCardDto; warnings: string[] }>("generate_saved_card_pronunciation", { cardUuid, fieldKey });
    return { value: toSavedCard(result.card), warnings: result.warnings };
};

/** Takes a saved card out of its stack. The card is kept. */
export const unstackSavedCard = async (cardUuid: string) =>
    toSavedCard(await invoke<SavedCardDto>("unstack_saved_card", { cardUuid }));

/** Deletes every card of a stack, with their review history. */
export const deleteSavedStack = (stackUuid: string) => invoke<void>("delete_saved_stack", { stackUuid });

/** Generates the audio and the pronunciation of the saved cards of a stack from the saved text. */
export const generateSavedStack = async (stackUuid: string): Promise<Generated<SavedCard[]>> => {
    const result = await invoke<{ cards: SavedCardDto[]; warnings: string[] }>("generate_saved_stack", { stackUuid });
    return { value: result.cards.map(toSavedCard), warnings: result.warnings };
};

export const removeSavedCardMedia = async (cardUuid: string, fieldKey: string, sha256: string) =>
    toSavedCard(await invoke<SavedCardDto>("remove_saved_card_media", { cardUuid, fieldKey, sha256 }));

/** Deletes a saved flashcard with its review history. */
export const deleteSavedCard = (cardUuid: string) => invoke<void>("delete_saved_card", { cardUuid });
