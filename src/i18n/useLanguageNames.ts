import { useTranslation } from "react-i18next";

/** The name of a code in the interface language, or `fallback` when the runtime does not know it. */
function displayName(locale: string, code: string | null, fallback: string, style: "standard"): string {
    if (!code) return fallback;
    try {
        const names = new Intl.DisplayNames([locale], { type: "language", languageDisplay: style });
        return names.of(code) ?? fallback;
    } catch {
        return fallback;
    }
}

/**
 * Names of study languages and accents in the interface language, from their ISO 639-1 and BCP 47
 * codes ("French", "English (United States)" become "français", "anglais (États-Unis)"). The
 * backend's seeded English name stays for a code the runtime has no name for.
 */
export function useLanguageNames() {
    const { i18n } = useTranslation();
    const locale = i18n.resolvedLanguage ?? "en";
    return {
        /** A language by its ISO 639-1 code, or by the language part of a BCP 47 code ("en-US" gives "en"). */
        language: (code: string | null, fallback: string) => displayName(locale, code?.split("-")[0] ?? null, fallback, "standard"),
        variant: (variant: { bcp47Code: string; variantName: string }) =>
            displayName(locale, variant.bcp47Code, variant.variantName, "standard"),
    };
}
