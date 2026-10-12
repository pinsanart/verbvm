import { useCallback, useRef, useState } from "react";
import { errorMessage } from "../api/errors";

/**
 * Runs async actions for a viewmodel, capturing the error message and tracking whether any
 * action is still running. `run` resolves to `true` when the action succeeded.
 */
export function useAsyncRunner() {
    const [error, setError] = useState<string | null>(null);
    const [busy, setBusy] = useState(false);
    const running = useRef(0);

    const run = useCallback(async (action: () => Promise<void>) => {
        running.current += 1;
        setBusy(true);
        try {
            setError(null);
            await action();
            return true;
        } catch (err) {
            setError(errorMessage(err));
            return false;
        } finally {
            running.current -= 1;
            if (running.current === 0) setBusy(false);
        }
    }, []);

    const clearError = useCallback(() => setError(null), []);

    return { error, busy, run, clearError };
}
