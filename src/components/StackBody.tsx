import { Link } from "react-router";
import { useTranslation } from "react-i18next";
import { Plus, Sparkles } from "lucide-react";
import { Button } from "./Button";
import { PlainInputField } from "./PlainInputField";
import { StackLane } from "./StackLane";
import { useTypeLabel } from "../i18n/useTypeLabel";
import type { StackData } from "../lib/stackFields";

export interface StackBodyProps {
    stack: StackData;
    onPhraseChange: (text: string) => void;
    onGenerateAll: () => void;
    onAddMember: (typeSlug: string) => void;
    onChange: (uuid: string, key: string, value: string) => void;
    onAttachMedia: (uuid: string, key: string) => void;
    onGenerate: (uuid: string, key: string) => Promise<unknown>;
    onRemoveMedia: (uuid: string, key: string, sha256: string) => void;
    onToggleLink: (uuid: string) => void;
    onTakeOut: (uuid: string) => void;
    onDeleteCard: (uuid: string) => void;
}

/**
 * The inside of a stack: the phrase the cards share with the button that makes their audio and
 * pronunciation, then one band per card, separated by hairlines. The editor and the saved cards
 * panel both use it.
 */
export function StackBody(props: StackBodyProps) {
    const { t } = useTranslation();
    const typeLabel = useTypeLabel();
    const { lanes, phrase, phraseError, generating, notes, missing } = props.stack;

    return (
        <div className="flex flex-col gap-5">
            <div className="flex flex-col gap-3">
                <PlainInputField
                    label={t("stack.phrase")}
                    value={phrase}
                    onChange={props.onPhraseChange}
                    required
                    error={phraseError}
                />
                <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
                    <Button onClick={props.onGenerateAll} loading={generating} disabled={phrase.trim() === ""} className="w-full md:w-auto">
                        {!generating && <Sparkles size={16} aria-hidden />}
                        {generating ? t("stack.generating") : t("stack.generate")}
                    </Button>
                    <p className="text-meta text-muted">{t("stack.sharedHint")}</p>
                </div>
                <div role="status" className="flex flex-col gap-1.5">
                    {notes.map((note) => (
                        <p key={note} className="text-meta text-muted">
                            {note}
                        </p>
                    ))}
                    {notes.length > 0 && (
                        <Link to="/settings" className="w-fit text-meta text-accent-ink underline">
                            {t("stack.settingsLink")}
                        </Link>
                    )}
                </div>
            </div>

            <div className="flex flex-col divide-y divide-line border-t border-line pt-5">
                {lanes.map((lane) => (
                    <StackLane
                        key={`${lane.uuid}:${lane.revision}`}
                        {...lane}
                        onChange={(key, value) => props.onChange(lane.uuid, key, value)}
                        onAttachMedia={(key) => props.onAttachMedia(lane.uuid, key)}
                        onGenerate={(key) => props.onGenerate(lane.uuid, key)}
                        onRemoveMedia={(key, sha256) => props.onRemoveMedia(lane.uuid, key, sha256)}
                        onToggleLink={() => props.onToggleLink(lane.uuid)}
                        onTakeOut={() => props.onTakeOut(lane.uuid)}
                        onDeleteCard={() => props.onDeleteCard(lane.uuid)}
                    />
                ))}
            </div>

            {missing.length > 0 && (
                <div className="flex flex-wrap gap-1 border-t border-line pt-3">
                    {missing.map((slug) => (
                        <Button key={slug} variant="ghost" onClick={() => props.onAddMember(slug)}>
                            <Plus size={16} aria-hidden />
                            {t("stack.addBack", { type: typeLabel(slug) })}
                        </Button>
                    ))}
                </div>
            )}
        </div>
    );
}
