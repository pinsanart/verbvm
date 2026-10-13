import { useTranslation } from "react-i18next";
import { DatabaseBackup } from "lucide-react";
import { Button } from "./Button";
import { Notice } from "./Notice";
import type { AnkiImportReport } from "../types/AnkiImport";

export interface AnkiImportSettingsProps {
    report: AnkiImportReport | null;
    importing: boolean;
    error: string | null;
    onImport: () => void;
}

/**
 * The Anki import in Settings: what it does, the button that picks the collection, and what it
 * made for each language afterwards.
 */
export function AnkiImportSettings({ report, importing, error, onImport }: AnkiImportSettingsProps) {
    const { t } = useTranslation();

    return (
        <section aria-labelledby="anki-title" className="flex max-w-xl flex-col gap-4">
            <div className="flex flex-col gap-1.5">
                <h2 id="anki-title" className="text-heading">
                    {t("anki.title")}
                </h2>
                <p className="text-ui text-muted">{t("anki.intro")}</p>
            </div>

            {error && <Notice tone="error">{error}</Notice>}

            <div className="flex flex-col gap-2">
                <div>
                    <Button variant="outline" onClick={onImport} loading={importing}>
                        {!importing && <DatabaseBackup size={16} aria-hidden />}
                        {importing ? t("anki.importing") : t("anki.import")}
                    </Button>
                </div>
                {importing && (
                    <div role="status" className="flex flex-col gap-2">
                        <div className="h-1 w-full max-w-xs overflow-hidden rounded-full bg-white/10">
                            <div className="h-full w-1/3 animate-pulse rounded-full bg-accent" />
                        </div>
                        <p className="text-meta text-muted">{t("anki.importingHint")}</p>
                    </div>
                )}
            </div>

            {report && (
                <div className="flex flex-col gap-3">
                    <Notice tone="success">{t("anki.imported")}</Notice>
                    <ul className="divide-y divide-line rounded-xl border border-line bg-panel">
                        {report.decks.map((deck) => (
                            <li key={deck.name} className="flex flex-col gap-0.5 px-4 py-3">
                                <div className="flex items-baseline justify-between gap-4">
                                    <span className="text-ui font-medium text-ink">{deck.name}</span>
                                    {deck.skipped === null && (
                                        <span className="text-meta tabular-nums text-muted">
                                            {t("anki.stacks", { count: deck.stacks })} ·{" "}
                                            {t("decks.cards", { count: deck.stacks * 2 + deck.vocabulary + deck.phonetic })}
                                        </span>
                                    )}
                                </div>
                                {deck.skipped !== null && (
                                    <span className="text-meta text-muted">{t("anki.skipped", { reason: deck.skipped })}</span>
                                )}
                                {deck.skipped === null && deck.images > 0 && (
                                    <span className="text-meta text-subtle">{t("anki.images", { count: deck.images })}</span>
                                )}
                            </li>
                        ))}
                    </ul>
                    {report.partial && <Notice tone="info">{t("anki.partial")}</Notice>}
                    {report.warnings.length > 0 && (
                        <ul className="flex flex-col gap-1 text-meta text-muted">
                            {report.warnings.map((warning) => (
                                <li key={warning}>{warning}</li>
                            ))}
                        </ul>
                    )}
                </div>
            )}
        </section>
    );
}
