import { useTranslation } from "react-i18next";
import { Page } from "../components/Page";
import { Select } from "../components/Select";
import { labelClass } from "../components/formStyles";
import { LANGUAGES, setUiLanguage, type UiLanguage } from "../i18n";

export default function Settings() {
  const { t, i18n } = useTranslation();

  return (
    <Page title={t("nav.settings")}>
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
    </Page>
  );
}
