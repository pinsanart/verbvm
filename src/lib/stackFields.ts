import { plainText } from "./text";
import type { FieldDefinition } from "../types/FlashcardForm";
import type { Media } from "../types/Media";

/** The field the cards of a stack share, and the one their generated fields are made from. */
export const SHARED_KEY = "phrase";

/** The cards a stack is made of, in the order they are shown. */
export const STACK_TYPE_ORDER = ["vocabulary", "phonetic", "orthographic"] as const;

/** The fields of a flashcard type, front before back. */
export interface TypeFields {
    front: FieldDefinition[];
    back: FieldDefinition[];
}

/** What the stack views need to know about one card of a stack, whether it is a draft or saved. */
export interface StackMember {
    uuid: string;
    typeSlug: string;
    values: Record<string, string>;
}

/** The members in the order of the stack (vocabulary, phonetic, orthographic, then any other). */
export function orderedMembers<T extends { typeSlug: string }>(members: T[]): T[] {
    const rank = (slug: string) => {
        const index = (STACK_TYPE_ORDER as readonly string[]).indexOf(slug);
        return index === -1 ? STACK_TYPE_ORDER.length : index;
    };
    return members
        .map((member, position) => ({ member, position }))
        .sort((a, b) => rank(a.member.typeSlug) - rank(b.member.typeSlug) || a.position - b.position)
        .map(({ member }) => member);
}

/** The types of a stack that no member has, so the view can offer to add them back. */
export function missingTypes(members: { typeSlug: string }[]): string[] {
    return STACK_TYPE_ORDER.filter((slug) => !members.some((m) => m.typeSlug === slug));
}

/** The phrase of a card as plain text: the vocabulary phrase is HTML, the others are text. */
export const phraseOf = (values: Record<string, string>) => plainText(values[SHARED_KEY] ?? "");

/**
 * The cards that already have a phrase of their own when a stack is loaded: the ones whose phrase
 * is not empty and differs from the first phrase in the stack. They start unlinked.
 */
export function initialUnlinked(members: StackMember[]): Set<string> {
    const shared = members.map((m) => phraseOf(m.values)).find((phrase) => phrase !== "") ?? "";
    return new Set(
        members.filter((m) => phraseOf(m.values) !== "" && phraseOf(m.values) !== shared).map((m) => m.uuid),
    );
}

/** The phrase the linked cards share: the one of the first card that is linked. */
export function sharedPhrase(members: StackMember[], unlinked: ReadonlySet<string>): string {
    const linked = members.filter((m) => !unlinked.has(m.uuid));
    return phraseOf((linked[0] ?? members[0])?.values ?? {});
}

/** The key a field is generated from, when its text is missing there is nothing to generate. */
export function canGenerate(definition: FieldDefinition, values: Record<string, string>): boolean {
    if (definition.sourceKey === null) return false;
    return plainText(values[definition.sourceKey] ?? "").trim() !== "";
}

/** One card of a stack as the view shows it. */
export interface StackLaneData {
    /** The card's uuid. */
    uuid: string;
    /** Changes when the card's text was replaced from outside (generated), so its editors reload. */
    revision: number;
    typeSlug: string;
    fields: TypeFields;
    values: Record<string, string>;
    media: Record<string, Media[]>;
    errors: Record<string, string>;
    /** The card's phrase follows the stack's. Unlinked, the card shows and edits its own. */
    linked: boolean;
}

/** A stack as the view shows it: the shared phrase and its cards in order. */
export interface StackData {
    stackUuid: string;
    phrase: string;
    phraseError?: string;
    lanes: StackLaneData[];
    /** The card types the stack does not have. */
    missing: string[];
    generating: boolean;
    /** What the last generation reported. */
    notes: string[];
}
