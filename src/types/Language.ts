// Languages are identified by their ISO 639-3 code and variants by their BCP 47 code.
// No database id crosses the boundary between the backend and the view.

export interface Language {
    iso6391: string | null;
    iso6393: string;
    name: string;
    /** BCP 47 code of the default variant. */
    defaultVariantCode: string | null;
}

export interface LanguageVariant {
    bcp47Code: string;
    variantName: string;
    countryCode: string | null;
}

export type CommandErrorKind = "not_found" | "invalid" | "conflict" | "internal";

export interface CommandError {
    kind: CommandErrorKind;
    message: string;
}
