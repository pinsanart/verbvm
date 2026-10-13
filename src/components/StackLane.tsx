import { useId } from "react";
import { AnimatePresence, motion } from "motion/react";
import { useTranslation } from "react-i18next";
import { Link2, SquareArrowOutUpRight, Unlink2 } from "lucide-react";
import { Button } from "./Button";
import { ConfirmDeleteButton } from "./ConfirmDeleteButton";
import { FieldInput } from "./FieldInput";
import { IconButton } from "./IconButton";
import { styleForType } from "../data/FlashcardTypeStyles";
import { useTypeLabel } from "../i18n/useTypeLabel";
import { TRANSITION_FAST } from "../lib/motion";
import { canGenerate, SHARED_KEY } from "../lib/stackFields";
import type { TypeFields } from "../lib/stackFields";
import type { Media } from "../types/Media";

export interface StackLaneProps {
    typeSlug: string;
    fields: TypeFields;
    values: Record<string, string>;
    media: Record<string, Media[]>;
    errors: Record<string, string>;
    /** The card's phrase follows the stack's. Unlinked, the card shows and edits its own. */
    linked: boolean;
    onChange: (key: string, value: string) => void;
    onAttachMedia: (key: string) => void;
    onGenerate: (key: string) => Promise<unknown>;
    onRemoveMedia: (key: string, sha256: string) => void;
    onToggleLink: () => void;
    onTakeOut: () => void;
    onDeleteCard: () => void;
}

/**
 * One card of a stack: its type as the heading, then its own fields. It is an ordinary card of
 * its type minus the phrase it shares with the others, so it can be edited, taken out of the stack
 * or deleted on its own. While the phrase is linked the shared phrase above stands for it.
 */
export function StackLane({
    typeSlug,
    fields,
    values,
    media,
    errors,
    linked,
    onChange,
    onAttachMedia,
    onGenerate,
    onRemoveMedia,
    onToggleLink,
    onTakeOut,
    onDeleteCard,
}: StackLaneProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const titleId = useId();
    const { icon: Icon, badge } = styleForType(typeSlug);
    const shown = [...fields.front, ...fields.back].filter((field) => !(linked && field.key === SHARED_KEY));
    // Rich text and audio need the width, short fields can share a row.
    const wide = (kind: string) => kind === "IPA" || kind === "RICH" || kind === "AUDIO";

    return (
        <section aria-labelledby={titleId} className="flex flex-col gap-4 py-5 first:pt-0 last:pb-0">
            <header className="flex flex-wrap items-center gap-x-3 gap-y-2">
                <h3 id={titleId} className={`flex items-center gap-1.5 rounded-full px-3 py-1 text-meta font-medium ${badge}`}>
                    <Icon size={16} aria-hidden />
                    {typeLabel(typeSlug)}
                </h3>
                <div className="ml-auto flex items-center gap-0.5">
                    <IconButton
                        label={linked ? t("stack.linkedAria") : t("stack.unlinkedAria")}
                        icon={linked ? Link2 : Unlink2}
                        iconSize={16}
                        pressed={linked}
                        onClick={onToggleLink}
                    />
                    <IconButton label={t("stack.takeOutAria")} icon={SquareArrowOutUpRight} iconSize={16} onClick={onTakeOut} />
                    <ConfirmDeleteButton
                        deleteLabel={t("card.deleteAria")}
                        confirmLabel={t("card.confirmDeleteAria")}
                        onConfirm={onDeleteCard}
                    />
                </div>
            </header>

            <AnimatePresence initial={false}>
                {!linked && (
                    <motion.div
                        key="own-phrase"
                        initial={{ opacity: 0, height: 0 }}
                        animate={{ opacity: 1, height: "auto" }}
                        exit={{ opacity: 0, height: 0 }}
                        transition={TRANSITION_FAST}
                        className="overflow-hidden"
                    >
                        <Button variant="ghost" onClick={onToggleLink} className="-ml-3">
                            <Link2 size={16} aria-hidden />
                            {t("stack.useStackPhrase")}
                        </Button>
                    </motion.div>
                )}
            </AnimatePresence>

            <div className="grid grid-cols-1 gap-5 md:grid-cols-2">
                {shown.map((field) => (
                    <div key={field.key} className={wide(field.kind) ? "md:col-span-2" : undefined}>
                        <FieldInput
                            definition={field}
                            value={values[field.key] ?? ""}
                            error={errors[field.key]}
                            media={media[field.key]}
                            onChange={(value) => onChange(field.key, value)}
                            onAttachMedia={() => onAttachMedia(field.key)}
                            onGenerate={() => onGenerate(field.key)}
                            canGenerate={canGenerate(field, values)}
                            onRemoveMedia={(sha256) => onRemoveMedia(field.key, sha256)}
                        />
                    </div>
                ))}
            </div>
        </section>
    );
}
