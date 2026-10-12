import { useId } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Copy, Eye, EyeOff } from "lucide-react";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { CountryFlag } from "./CountryFlag";
import { FieldInput } from "./FieldInput";
import { IconButton } from "./IconButton";
import { useTypeLabel } from "../i18n/useTypeLabel";
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
    const { t } = useTranslation();
    const { icon: TypeIcon, badge } = styleForType(typeSlug);
    const label = useTypeLabel()(typeSlug);

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
                <IconButton label={t("card.moveUp")} icon={ArrowUp} iconSize={16} onClick={onMoveUp} disabled={isFirst} />
                <IconButton label={t("card.moveDown")} icon={ArrowDown} iconSize={16} onClick={onMoveDown} disabled={isLast} />
                <IconButton label={t("card.duplicate")} icon={Copy} iconSize={16} onClick={onDuplicate} />
                <ConfirmDeleteButton
                    deleteLabel={t("card.deleteAria")}
                    confirmLabel={t("card.confirmDeleteAria")}
                    onConfirm={onDelete}
                />
            </div>
        </div>
    );
}

// Front and back share one surface and one palette. They differ in structure: the front has a
// solid top rule and an open eye (what you see first), the back a dashed top rule and a closed one
// (what stays hidden until you flip).
const FACES = {
    front: {
        titleKey: "card.front",
        hintKey: "card.frontHint",
        icon: Eye,
        spine: "border-t-4 border-solid border-ink",
    },
    back: {
        titleKey: "card.back",
        hintKey: "card.backHint",
        icon: EyeOff,
        spine: "border-t-4 border-dashed border-line-strong",
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
    const { t } = useTranslation();
    const titleId = useId();
    if (fields.length === 0) return null;
    const { titleKey, hintKey, icon: Icon, spine } = FACES[face];

    return (
        <section aria-labelledby={titleId} className={`flex flex-col gap-5 pt-4 ${spine}`}>
            <header className="flex flex-wrap items-center gap-x-2.5 gap-y-1">
                <Icon size={18} className="text-muted" aria-hidden />
                <h3 id={titleId} className="text-body font-semibold text-ink">
                    {t(titleKey)}
                </h3>
                <p className="text-meta text-muted">{t(hintKey)}</p>
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

export interface FlashcardFacesProps {
    frontFields: FieldDefinition[];
    backFields: FieldDefinition[];
    values: Record<string, string>;
    errors: Record<string, string>;
    media: Record<string, Media[]>;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onRemoveMedia: (key: string, sha256: string) => void;
}

/** The two faces of a card with the flip divider between them. Drafts and saved cards share it. */
export function FlashcardFaces({ frontFields, backFields, ...faceProps }: FlashcardFacesProps) {
    return (
        <>
            <Face face="front" fields={frontFields} {...faceProps} />
            {frontFields.length > 0 && backFields.length > 0 && (
                <hr className="border-line" aria-hidden />
            )}
            <Face face="back" fields={backFields} {...faceProps} columns />
        </>
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
                <FlashcardFaces
                    frontFields={frontFields}
                    backFields={backFields}
                    {...{ values, errors, media, onChange, onAttachMedia, onRemoveMedia }}
                />
            </div>
        </article>
    );
}
