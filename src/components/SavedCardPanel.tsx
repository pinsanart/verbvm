import { useTranslation } from "react-i18next";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { FlashcardFaces } from "./FlashcardInput";
import { SidePanel } from "./SidePanel";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { FieldDefinition, SavedCard } from "../types/FlashcardForm";

export interface SavedCardPanelProps {
    /** The card being edited. The panel is closed when this is null. */
    card: SavedCard | null;
    frontFields: FieldDefinition[];
    backFields: FieldDefinition[];
    values: Record<string, string>;
    dirty: boolean;
    justSaved: boolean;
    error: string | null;
    busy: boolean;
    /** Changes when the card's text was replaced from outside (generated), so its editors reload. */
    revision: number;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onGenerate: (key: string) => Promise<unknown>;
    onRemoveMedia: (key: string, sha256: string) => void;
    onSave: () => void;
    onDelete: () => void;
    onClose: () => void;
}

/** The panel that edits one saved card, even when it belongs to a stack: the others are not touched. */
export function SavedCardPanel(props: SavedCardPanelProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const { card, frontFields, backFields, values, revision } = props;
    const style = card ? styleForType(card.typeSlug) : null;

    return (
        <SidePanel
            id={card?.uuid ?? null}
            ariaLabel={card ? t("panel.editAria", { type: typeLabel(card.typeSlug) }) : ""}
            badge={
                card &&
                style && (
                    <span className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${style.badge}`}>
                        <style.icon size={16} aria-hidden />
                        {typeLabel(card.typeSlug)}
                    </span>
                )
            }
            title={t("panel.title")}
            headerActions={
                <ConfirmDeleteButton
                    deleteLabel={t("card.deleteAria")}
                    confirmLabel={t("card.confirmDeleteAria")}
                    onConfirm={props.onDelete}
                />
            }
            dirty={props.dirty}
            justSaved={props.justSaved}
            error={props.error}
            busy={props.busy}
            onSave={props.onSave}
            onClose={props.onClose}
        >
            {card && (
                <FlashcardFaces
                    key={`${card.uuid}:${revision}`}
                    frontFields={frontFields}
                    backFields={backFields}
                    values={values}
                    errors={{}}
                    media={card.media}
                    onChange={props.onChange}
                    onAttachMedia={props.onAttachMedia}
                    onGenerate={props.onGenerate}
                    onRemoveMedia={props.onRemoveMedia}
                />
            )}
        </SidePanel>
    );
}
