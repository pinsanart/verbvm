import { useState } from "react";
import { useAsyncRunner } from "./useAsyncRunner";
import { importAnkiCollection, pickAnkiCollection } from "../api/anki";
import type { AnkiImportReport } from "../types/AnkiImport";

/**
 * The view's API for importing an Anki collection: pick the file, run the import, show what it
 * made. The backend creates a deck per language and skips one that already exists.
 */
export function useAnkiImportViewModel() {
    const [report, setReport] = useState<AnkiImportReport | null>(null);
    const [importing, setImporting] = useState(false);
    const { error, run, clearError } = useAsyncRunner();

    /** Asks for the collection file, then imports it. Nothing happens if none is chosen. */
    const importCollection = () =>
        run(async () => {
            const path = await pickAnkiCollection();
            if (path === null) return;
            clearError();
            setReport(null);
            setImporting(true);
            try {
                setReport(await importAnkiCollection(path));
            } finally {
                setImporting(false);
            }
        });

    return { report, importing, error, importCollection };
}
