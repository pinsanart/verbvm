import type { CommandError } from "../types/Language";

/** Turns whatever `invoke` rejected with into a message to show. */
export function errorMessage(err: unknown): string {
    if (err && typeof err === "object" && "message" in err) {
        return (err as CommandError).message;
    }
    if (err instanceof TypeError) {
        return "Tauri is not available here. Run the app with `npm run tauri dev`.";
    }
    return String(err);
}
