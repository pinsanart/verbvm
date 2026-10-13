import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Copy } from "lucide-react";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { CountryFlag } from "./CountryFlag";
import { IconButton } from "./IconButton";
import { StackBody } from "./StackBody";
import type { StackBodyProps } from "./StackBody";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";

export interface StackInputProps extends StackBodyProps {
    /** The deck's accent, shown next to the stack. */
    accent: { name: string; countryCode: string | null };
    isFirst: boolean;
    isLast: boolean;
    onMoveUp: () => void;
    onMoveDown: () => void;
    onDuplicate: () => void;
    /** Deletes the whole stack, with every card in it. */
    onDeleteStack: () => void;
}

/**
 * A stack in the editor: one surface with the stack's accent on top and the controls of the whole
 * stack, around the phrase and the cards (see `StackBody`). Moving, duplicating and deleting act
 * on every card of the stack. Each card keeps its own controls.
 */
export function StackInput({ accent, isFirst, isLast, onMoveUp, onMoveDown, onDuplicate, onDeleteStack, ...body }: StackInputProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const { icon: Icon, badge, accent: bar } = styleForType("stack");

    return (
        <article aria-label={t("stack.aria")} className="overflow-hidden rounded-2xl border border-line bg-panel shadow-lg shadow-black/20">
            <div className={`h-1 w-full ${bar}`} />
            <div className="flex flex-col gap-5 p-5 md:p-6">
                <div className="flex items-center gap-3">
                    <div className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${badge}`}>
                        <Icon size={16} aria-hidden />
                        {typeLabel("stack")}
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
                            deleteLabel={t("stack.deleteAria")}
                            confirmLabel={t("stack.confirmDeleteAria")}
                            onConfirm={onDeleteStack}
                        />
                    </div>
                </div>
                <StackBody {...body} />
            </div>
        </article>
    );
}
