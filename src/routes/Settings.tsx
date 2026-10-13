import { useTranslation } from "react-i18next";
import { AnkiImportSettings } from "../components/AnkiImportSettings";
import { DictionarySettings } from "../components/DictionarySettings";
import { Page } from "../components/Page";
import { Select } from "../components/Select";
import { labelClass } from "../components/formStyles";
import { LANGUAGES, setUiLanguage, type UiLanguage } from "../i18n";
import { useAnkiImportViewModel } from "../viewmodels/useAnkiImportViewModel";
import { useDictionaryViewModel } from "../viewmodels/useDictionaryViewModel";

export default function Settings() {
  const { t, i18n } = useTranslation();
  const dictionary = useDictionaryViewModel();
  const anki = useAnkiImportViewModel();

  return (
    <Page title={t("nav.settings")}>
      <div className="flex flex-col gap-10">
        <div className="max-w-sm">
          <label className={labelClass} htmlFor="ui-language">
            {t("settings.languageLabel")}
          </label>
          <Select
            id="ui-language"
            value={i18n.resolvedLanguage ?? "en"}
            onChange={(code) => setUiLanguage(code as UiLanguage)}
            options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
          />
          <p className="mt-2 text-meta text-muted">{t("settings.languageHint")}</p>
        </div>

        <DictionarySettings
          status={dictionary.status}
          importing={dictionary.importing}
          imported={dictionary.imported}
          error={dictionary.error}
          onImport={dictionary.importFromFolder}
        />

        <AnkiImportSettings
          report={anki.report}
          importing={anki.importing}
          error={anki.error}
          onImport={anki.importCollection}
        />
      </div>
    </Page>
  );
}
