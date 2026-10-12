import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import en from "../locales/en.json";
import fr from "../locales/fr.json";
import ptBR from "../locales/pt-BR.json";

/** The languages the interface is translated into. English is the default and the fallback. */
export const LANGUAGES = [
    { code: "en", name: "English" },
    { code: "fr", name: "Français" },
    { code: "pt-BR", name: "Português (Brasil)" },
] as const;

export type UiLanguage = (typeof LANGUAGES)[number]["code"];

const DEFAULT_LANGUAGE: UiLanguage = "en";
const STORAGE_KEY = "verbvm.uiLanguage";

const isSupported = (code: string | null): code is UiLanguage => LANGUAGES.some((l) => l.code === code);

/** The language the user chose, or English when none was stored or storage is unavailable. */
function storedLanguage(): UiLanguage {
    try {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (isSupported(stored)) return stored;
    } catch {
        // Storage can be blocked. The default applies.
    }
    return DEFAULT_LANGUAGE;
}

/** Switches the interface language, remembers it and tells the document and the screen readers. */
export function setUiLanguage(code: UiLanguage) {
    try {
        localStorage.setItem(STORAGE_KEY, code);
    } catch {
        // The choice still applies for this session.
    }
    void i18n.changeLanguage(code);
}

i18n.use(initReactI18next).init({
    resources: {
        en: { translation: en },
        fr: { translation: fr },
        "pt-BR": { translation: ptBR },
    },
    lng: storedLanguage(),
    fallbackLng: DEFAULT_LANGUAGE,
    interpolation: { escapeValue: false },
});

document.documentElement.lang = i18n.language;
i18n.on("languageChanged", (lng) => {
    document.documentElement.lang = lng;
});

export default i18n;
