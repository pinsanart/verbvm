import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DictionaryStatus } from "../types/Pronunciation";

/** What the pronunciation dictionary holds, and the accents it can hold. */
export const pronunciationStatus = () => invoke<DictionaryStatus>("pronunciation_status");

/**
 * Builds the dictionary from the `ipa-dict` word lists in a folder. It takes a while. A failed
 * import leaves the dictionary that was installed.
 */
export const importPronunciationDictionary = (folder: string) =>
    invoke<DictionaryStatus>("import_pronunciation_dictionary", { folder });

/** Opens the system folder picker. Resolves to the chosen folder, or `null`. */
export async function pickFolder(): Promise<string | null> {
    const picked = await open({ multiple: false, directory: true });
    return typeof picked === "string" ? picked : null;
}
