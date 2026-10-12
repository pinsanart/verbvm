import { useCallback, useState } from "react";

const STORAGE_KEY = "verbvm.review.collapsedDecks";

const load = (): ReadonlySet<string> => {
    try {
        const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "[]");
        return new Set(Array.isArray(stored) ? stored.filter((name) => typeof name === "string") : []);
    } catch {
        return new Set();
    }
};

const save = (names: ReadonlySet<string>) => {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify([...names]));
    } catch {
        // Storage can be unavailable. The choice then lasts until the screen is left.
    }
};

/**
 * Which decks of the review overview are collapsed. It is only how the screen looks, so it is kept
 * in the browser and never reaches the backend.
 */
export function useCollapsedDecks() {
    const [collapsed, setCollapsed] = useState<ReadonlySet<string>>(load);

    const update = useCallback((change: (current: ReadonlySet<string>) => ReadonlySet<string>) => {
        setCollapsed((current) => {
            const next = change(current);
            save(next);
            return next;
        });
    }, []);

    const toggle = useCallback(
        (name: string) =>
            update((current) => {
                const next = new Set(current);
                if (!next.delete(name)) next.add(name);
                return next;
            }),
        [update],
    );

    const collapseAll = useCallback((names: string[]) => update(() => new Set(names)), [update]);
    const expandAll = useCallback(() => update(() => new Set()), [update]);

    return { collapsed, toggle, collapseAll, expandAll };
}
