import { useEffect, useRef } from "react";
import { keyboard } from "./keyboard";

/**
 * Runs `handler` when one of `keys` is pressed anywhere on the page, while `enabled`. The latest
 * `handler` is always the one called, so it can use current state without resubscribing on every
 * render. Use this for global shortcuts instead of listening to `keydown` on the document.
 */
export function useKeys(keys: readonly string[], handler: (event: KeyboardEvent) => void, enabled = true) {
    const latest = useRef(handler);
    latest.current = handler;
    const wanted = keys.join("\u0000");

    useEffect(() => {
        if (!enabled) return;
        return keyboard().subscribe({
            keys: wanted.split("\u0000"),
            notify: (event) => latest.current(event),
        });
    }, [wanted, enabled]);
}
