import { useTranslation } from "react-i18next";
import type { ParseKeys } from "i18next";
import en from "../locales/en.json";

const hasLabel = (key: string): key is keyof typeof en.fields => key in en.fields;

/**
 * The label of a field in the interface language, looked up by its key. The backend sends its own
 * label, which stays for a field this app has no translation for (for example a language-specific one).
 */
export function useFieldLabel() {
    const { t } = useTranslation();
    return (key: string, fallbackLabel: string) =>
        hasLabel(key) ? t(`fields.${key}` satisfies ParseKeys) : fallbackLabel;
}
