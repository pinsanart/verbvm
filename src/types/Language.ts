export interface Language {
    id: number;
    iso6391: string | null;
    iso6393: string;
    name: string;
    defaultVariantId: number | null;
}

export interface LanguageVariant {
    id: number;
    languageId: number;
    bcp47Code: string;
    variantName: string;
    countryCode: string | null;
}

export interface NewLanguage {
    iso6391: string | null;
    iso6393: string;
    name: string;
}

export interface NewLanguageVariant {
    languageId: number;
    bcp47Code: string;
    variantName: string;
    countryCode: string | null;
}

export type CommandErrorKind = "not_found" | "invalid" | "conflict" | "internal";

export interface CommandError {
    kind: CommandErrorKind;
    message: string;
}
