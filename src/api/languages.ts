import { invoke } from "@tauri-apps/api/core";
import type { CommandError, Language, LanguageVariant, NewLanguage, NewLanguageVariant } from "../types/Language";

export const listLanguages = () => invoke<Language[]>("list_languages");

export const createLanguage = (input: NewLanguage) => invoke<Language>("create_language", { input });

export const deleteLanguage = (id: number) => invoke<void>("delete_language", { id });

export const listLanguageVariants = (languageId: number) =>
    invoke<LanguageVariant[]>("list_language_variants", { languageId });

export const createLanguageVariant = (input: NewLanguageVariant) =>
    invoke<LanguageVariant>("create_language_variant", { input });

export const setDefaultLanguageVariant = (languageId: number, variantId: number) =>
    invoke<Language>("set_default_language_variant", { languageId, variantId });

export const deleteLanguageVariant = (id: number) => invoke<void>("delete_language_variant", { id });

/** Turns whatever `invoke` rejected with into a message to show. */
export function errorMessage(err: unknown): string {
    if (err && typeof err === "object" && "message" in err) {
        return (err as CommandError).message;
    }
    if (err instanceof TypeError) {
        return "Tauri is not available here. Run the app with `npm run tauri dev`.";
    }
    return String(err);
}
