import { useCallback, useEffect, useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { importPronunciationDictionary, pickFolder, pronunciationStatus } from "../api/pronunciation";
import type { DictionaryStatus } from "../types/Pronunciation";

/**
 * The view's API for the pronunciation dictionary: what is installed and the import from a folder
 * of `ipa-dict` word lists. The import takes a while and cannot be cancelled, and a failed import
 * leaves what was installed.
 */
export function useDictionaryViewModel() {
    const [status, setStatus] = useState<DictionaryStatus | null>(null);
    const [imported, setImported] = useState(false);
    const [importing, setImporting] = useState(false);
    const { error, run, clearError } = useAsyncRunner();

    const refresh = useCallback(() => run(async () => setStatus(await pronunciationStatus())), [run]);

    useEffect(() => {
        refresh();
    }, [refresh]);

    /** Asks for the folder, then builds the dictionary from it. Nothing happens if none is chosen. */
    const importFromFolder = () =>
        run(async () => {
            const folder = await pickFolder();
            if (folder === null) return;
            clearError();
            setImported(false);
            setImporting(true);
            try {
                setStatus(await importPronunciationDictionary(folder));
                setImported(true);
            } finally {
                setImporting(false);
            }
        });

    return { status, imported, importing, error, importFromFolder };
}
