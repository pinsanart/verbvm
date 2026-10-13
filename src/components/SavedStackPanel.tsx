import { useTranslation } from "react-i18next";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { SidePanel } from "./SidePanel";
import { StackBody } from "./StackBody";
import type { StackBodyProps } from "./StackBody";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";

export interface SavedStackPanelProps extends StackBodyProps {
    /** The stack being edited. The panel is closed when this is null. */
    stackUuid: string | null;
    dirty: boolean;
    justSaved: boolean;
    error: string | null;
    busy: boolean;
    onSave: () => void;
    /** Deletes the whole stack, with every card in it. */
    onDeleteStack: () => void;
    onClose: () => void;
}

/**
 * The panel that edits the saved cards of a stack together: the shared phrase and one band per
 * card, with one Save for all of them. Each card can also be taken out, deleted, or given its own
 * phrase, and a single card can be opened alone from the list.
 */
export function SavedStackPanel({ stackUuid, dirty, justSaved, error, busy, onSave, onDeleteStack, onClose, ...body }: SavedStackPanelProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const { icon: Icon, badge } = styleForType("stack");

    return (
        <SidePanel
            id={stackUuid}
            ariaLabel={t("panel.stackAria")}
            badge={
                <span className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${badge}`}>
                    <Icon size={16} aria-hidden />
                    {typeLabel("stack")}
                </span>
            }
            title={t("panel.stackTitle")}
            headerActions={
                <ConfirmDeleteButton
                    deleteLabel={t("stack.deleteAria")}
                    confirmLabel={t("stack.confirmDeleteAria")}
                    onConfirm={onDeleteStack}
                />
            }
            dirty={dirty}
            justSaved={justSaved}
            error={error}
            busy={busy}
            onSave={onSave}
            onClose={onClose}
        >
            {stackUuid && <StackBody {...body} />}
        </SidePanel>
    );
}
