import type en from "../locales/en.json";

// Types the keys of `t()` from the English file, so a missing or misspelled key fails `tsc`.
declare module "i18next" {
    interface CustomTypeOptions {
        defaultNS: "translation";
        resources: { translation: typeof en };
    }
}
