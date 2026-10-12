/**
 * Global keyboard shortcuts, as the observer pattern. One subject listens to the page's keys and
 * tells each observer only about the keys it asked for. A key nobody asked for (the volume keys,
 * for example) is dropped here, in one place, without running any handler or calling
 * `preventDefault`.
 */

export interface KeyObserver {
    /** The `KeyboardEvent.key` values it wants, such as `"Escape"`, `"1"`, `" "` or `"r"`. */
    keys: readonly string[];
    notify(event: KeyboardEvent): void;
}

/** Letters match whatever the case, so Shift or Caps Lock do not hide a shortcut. */
const normalize = (key: string) => (key.length === 1 ? key.toLowerCase() : key);

export class KeyboardSubject {
    private readonly observers = new Map<string, Set<KeyObserver>>();
    private listening = false;

    constructor(private readonly target: EventTarget = document) {}

    /** Starts telling the observer about its keys. Returns the function that stops it. */
    subscribe(observer: KeyObserver): () => void {
        for (const key of observer.keys) {
            const name = normalize(key);
            const set = this.observers.get(name) ?? new Set<KeyObserver>();
            set.add(observer);
            this.observers.set(name, set);
        }
        this.listen();
        return () => this.unsubscribe(observer);
    }

    private unsubscribe(observer: KeyObserver) {
        for (const key of observer.keys) {
            const name = normalize(key);
            const set = this.observers.get(name);
            if (!set) continue;
            set.delete(observer);
            if (set.size === 0) this.observers.delete(name);
        }
        if (this.observers.size === 0) this.stop();
    }

    // One listener for the whole app, present only while somebody is subscribed.
    private readonly onKeyDown = (event: Event) => {
        const keyboardEvent = event as KeyboardEvent;
        const interested = this.observers.get(normalize(keyboardEvent.key));
        if (!interested) return;
        for (const observer of [...interested]) observer.notify(keyboardEvent);
    };

    private listen() {
        if (this.listening) return;
        this.target.addEventListener("keydown", this.onKeyDown);
        this.listening = true;
    }

    private stop() {
        if (!this.listening) return;
        this.target.removeEventListener("keydown", this.onKeyDown);
        this.listening = false;
    }
}

/** Created on first use, so importing this file does not need a `document`. */
let shared: KeyboardSubject | null = null;
export const keyboard = (): KeyboardSubject => (shared ??= new KeyboardSubject());

/** The key is pressed together with Ctrl, Alt or Meta, so it is not a plain shortcut. */
export const hasModifier = (event: KeyboardEvent) => event.metaKey || event.ctrlKey || event.altKey;

/** The focus is in a field where keys write text, so no shortcut may take them. */
export const isTyping = (target: EventTarget | null) =>
    target instanceof HTMLElement &&
    target.closest("input, textarea, select, [contenteditable=''], [contenteditable='true']") !== null;

/** Keys that press the focused control themselves, so a shortcut must leave them alone. */
export const isTypingOrPressing = (target: EventTarget | null) =>
    target instanceof HTMLElement && target.closest("input, textarea, select, button, a") !== null;
