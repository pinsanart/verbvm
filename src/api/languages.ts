import { invoke } from "@tauri-apps/api/core";
import type { Language, LanguageVariant } from "../types/Language";

export const listLanguages = () => invoke<Language[]>("list_languages");

export const listLanguageVariants = (languageIso: string) =>
    invoke<LanguageVariant[]>("list_language_variants", { languageIso });
