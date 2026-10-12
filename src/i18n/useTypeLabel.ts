import { useTranslation } from "react-i18next";
import { typeLabelKey } from "../data/FlashcardTypeStyles";

/**
 * The name of a flashcard type in the interface language, looked up by its slug. The backend sends
 * its own name, which stays for a type this app has no translation for.
 */
export function useTypeLabel() {
    const { t } = useTranslation();
    return (slug: string, fallbackName?: string) => {
        const key = typeLabelKey(slug);
        return key ? t(key) : (fallbackName ?? t("types.flashcard"));
    };
}
