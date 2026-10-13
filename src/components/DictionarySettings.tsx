import { useTranslation } from "react-i18next";
import { Check, FolderInput } from "lucide-react";
import { Button } from "./Button";
import { CountryFlag } from "./CountryFlag";
import { Notice } from "./Notice";
import { useLanguageNames } from "../i18n/useLanguageNames";
import type { DictionaryStatus } from "../types/Pronunciation";

export interface DictionarySettingsProps {
    status: DictionaryStatus | null;
    importing: boolean;
    imported: boolean;
    error: string | null;
    onImport: () => void;
}

/**
 * The pronunciation dictionary in Settings: which accents it has and how many words, and the
 * button that imports it from a folder. It is a table of what is installed, not a form.
 */
export function DictionarySettings({ status, importing, imported, error, onImport }: DictionarySettingsProps) {
    const { t } = useTranslation();
    const names = useLanguageNames();
    const installed = new Map((status?.installed ?? []).map((l) => [l.variantCode, l.words]));

    return (
        <section aria-labelledby="dictionary-title" className="flex max-w-xl flex-col gap-4">
            <div className="flex flex-col gap-1.5">
                <h2 id="dictionary-title" className="text-heading">
                    {t("dictionary.title")}
                </h2>
                <p className="text-ui text-muted">{t("dictionary.intro")}</p>
            </div>

            {error && <Notice tone="error">{error}</Notice>}
            {imported && !importing && <Notice tone="success">{t("dictionary.imported")}</Notice>}

            {status && installed.size === 0 && !importing && <p className="text-ui text-muted">{t("dictionary.empty")}</p>}

            {status && (
                <ul className="divide-y divide-line rounded-xl border border-line bg-panel">
                    {status.supported.map((tag) => {
                        const words = installed.get(tag);
                        return (
                            <li key={tag} className="flex items-center gap-3 px-4 py-3">
                                <CountryFlag code={tag.split("-")[1] ?? null} className="h-4 w-6 shrink-0" />
                                <span className="min-w-0 flex-1 truncate text-ui text-ink">
                                    {names.variant({ bcp47Code: tag, variantName: tag })}
                                </span>
                                {words === undefined ? (
                                    <span className="text-meta text-subtle">{t("dictionary.notInstalled")}</span>
                                ) : (
                                    <span className="flex items-center gap-1.5 text-meta tabular-nums text-muted">
                                        <Check size={14} aria-label={t("dictionary.installed")} />
                                        {t("dictionary.words", { count: words })}
                                    </span>
                                )}
                            </li>
                        );
                    })}
                </ul>
            )}

            <div className="flex flex-col gap-2">
                <div>
                    <Button variant="outline" onClick={onImport} loading={importing}>
                        {!importing && <FolderInput size={16} aria-hidden />}
                        {importing ? t("dictionary.importing") : t("dictionary.import")}
                    </Button>
                </div>
                {importing && (
                    <div role="status" className="flex flex-col gap-2">
                        <div className="h-1 w-full max-w-xs overflow-hidden rounded-full bg-white/10">
                            <div className="h-full w-1/3 animate-pulse rounded-full bg-accent" />
                        </div>
                        <p className="text-meta text-muted">{t("dictionary.importingHint")}</p>
                    </div>
                )}
                <p className="text-meta text-subtle">{t("dictionary.licenseNote")}</p>
            </div>
        </section>
    );
}
