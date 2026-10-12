import { useId } from "react";
import { ArrowDown, ArrowUp, Copy, RotateCw } from "lucide-react";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { CountryFlag } from "./CountryFlag";
import { FieldInput } from "./FieldInput";
import { IconButton } from "./IconButton";
import { styleForType } from "../data/FlashcardTypeStyles";
import type { FieldDefinition } from "../types/FlashcardForm";
import type { Media } from "../types/Media";

export interface FlashcardInputProps {
    typeSlug: string;
    /** The deck's accent, shown next to the type. */
    accent: { name: string; countryCode: string | null };
    frontFields: FieldDefinition[];
    backFields: FieldDefinition[];
    /** The typed content by field key. */
    values: Record<string, string>;
    /** Errors by field key. */
    errors: Record<string, string>;
    /** The attached files by field key. */
    media: Record<string, Media[]>;
    isFirst: boolean;
    isLast: boolean;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onRemoveMedia: (key: string, sha256: string) => void;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onDuplicate: () => void;
    onDelete: () => void;
}

function ControlBar({
    typeSlug,
    accent,
    isFirst,
    isLast,
    onMoveUp,
    onMoveDown,
    onDuplicate,
    onDelete,
}: Pick<FlashcardInputProps, "typeSlug" | "accent" | "isFirst" | "isLast" | "onMoveUp" | "onMoveDown" | "onDuplicate" | "onDelete">) {
    const { icon: TypeIcon, label, badge } = styleForType(typeSlug);

    return (
        <div className="flex items-center gap-3">
            <div className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${badge}`}>
                <TypeIcon size={16} aria-hidden />
                {label}
            </div>

            <div className="flex items-center gap-1.5 text-meta text-muted">
                <CountryFlag code={accent.countryCode} className="h-3.5 w-5" />
                {accent.name}
            </div>

            <div className="ml-auto flex items-center gap-0.5 rounded-lg bg-white/5 p-1">
                <IconButton label="Move up" icon={ArrowUp} iconSize={16} onClick={onMoveUp} disabled={isFirst} />
                <IconButton label="Move down" icon={ArrowDown} iconSize={16} onClick={onMoveDown} disabled={isLast} />
                <IconButton label="Duplicate" icon={Copy} iconSize={16} onClick={onDuplicate} />
                <ConfirmDeleteButton label="this flashcard" onConfirm={onDelete} />
            </div>
        </div>
    );
}

const FACES = {
    front: {
        title: "Front",
        hint: "Shown first, as the prompt",
        // A solid light chip on a raised surface: the face you meet first.
        chip: "bg-ink text-canvas",
        surface: "border-line-strong bg-raised",
    },
    back: {
        title: "Back",
        hint: "Revealed after, as the answer",
        // An outlined chip on a sunken surface: the face behind it.
        chip: "border border-line-strong text-ink",
        surface: "border-line bg-canvas",
    },
} as const;

/** One side of the card. Front and back differ in surface, in their tag and in their wording. */
function Face({
    face,
    fields,
    values,
    errors,
    media,
    onChange,
    onAttachMedia,
    onRemoveMedia,
    columns,
}: {
    face: keyof typeof FACES;
    fields: FieldDefinition[];
    values: Record<string, string>;
    errors: Record<string, string>;
    media: Record<string, Media[]>;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onRemoveMedia: (key: string, sha256: string) => void;
    columns?: boolean;
}) {
    const titleId = useId();
    if (fields.length === 0) return null;
    const { title, hint, chip, surface } = FACES[face];

    return (
        <section aria-labelledby={titleId} className={`flex flex-col gap-5 rounded-xl border p-4 md:p-5 ${surface}`}>
            <header className="flex flex-wrap items-center gap-x-3 gap-y-1">
                <h3 id={titleId} className={`rounded-md px-2.5 py-1 text-ui font-semibold ${chip}`}>
                    {title}
                </h3>
                <p className="text-meta text-muted">{hint}</p>
            </header>
            <div className={columns ? "grid grid-cols-1 gap-5 md:grid-cols-2" : "flex flex-col gap-5"}>
                {fields.map((field) => (
                    <FieldInput
                        key={field.key}
                        definition={field}
                        value={values[field.key] ?? ""}
                        error={errors[field.key]}
                        media={media[field.key]}
                        onChange={(value) => onChange(field.key, value)}
                        onAttachMedia={() => onAttachMedia(field.key)}
                        onRemoveMedia={(sha256) => onRemoveMedia(field.key, sha256)}
                    />
                ))}
            </div>
        </section>
    );
}

/**
 * The editor of one flashcard. It renders whatever fields it is given, so a new type or a
 * language-specific field needs no change here. Render it with `key` set to the draft's uuid:
 * the rich editor keeps its own DOM and is not rewritten when `values` changes.
 */
export function FlashcardInput({
    typeSlug,
    accent,
    frontFields,
    backFields,
    values,
    errors,
    media,
    isFirst,
    isLast,
    onChange,
    onAttachMedia,
    onRemoveMedia,
    onMoveUp,
    onMoveDown,
    onDuplicate,
    onDelete,
}: FlashcardInputProps) {
    const faceProps = { values, errors, media, onChange, onAttachMedia, onRemoveMedia };

    return (
        <article className="overflow-hidden rounded-2xl border border-line bg-panel shadow-lg shadow-black/20">
            <div className={`h-1 w-full ${styleForType(typeSlug).accent}`} />
            <div className="flex flex-col gap-5 p-5 md:p-6">
                <ControlBar
                    typeSlug={typeSlug}
                    accent={accent}
                    isFirst={isFirst}
                    isLast={isLast}
                    onMoveUp={onMoveUp}
                    onMoveDown={onMoveDown}
                    onDuplicate={onDuplicate}
                    onDelete={onDelete}
                />
                <Face face="front" fields={frontFields} {...faceProps} />
                {frontFields.length > 0 && backFields.length > 0 && (
                    <div className="flex items-center gap-3 text-subtle" aria-hidden>
                        <span className="h-px flex-1 bg-line" />
                        <RotateCw size={16} />
                        <span className="h-px flex-1 bg-line" />
                    </div>
                )}
                <Face face="back" fields={backFields} {...faceProps} columns />
            </div>
        </article>
    );
}
