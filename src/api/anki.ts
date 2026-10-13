import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import i18n from "../i18n";
import type { AnkiImportReport } from "../types/AnkiImport";

/** Imports the content of an Anki collection as new decks. The reviews are not imported. */
export const importAnkiCollection = (path: string) => invoke<AnkiImportReport>("import_anki_collection", { path });

/** Opens the system file picker for an Anki collection. Resolves to the chosen file, or `null`. */
export async function pickAnkiCollection(): Promise<string | null> {
    const picked = await open({
        multiple: false,
        directory: false,
        filters: [{ name: i18n.t("anki.pickerFilter"), extensions: ["anki2", "anki21"] }],
    });
    return typeof picked === "string" ? picked : null;
}
