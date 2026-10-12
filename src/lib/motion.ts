// Motion in this app conveys state: something appeared, moved, was removed or was pressed.
// There is one easing and two durations, so every animation feels like the same hand.

/** Exponential ease-out: quick to react, soft to settle. Starts from an already visible state. */
export const EASE_OUT = [0.16, 1, 0.3, 1] as const;

/** Seconds. `fast` is for feedback (press, swap), `base` for things that move or appear. */
export const DURATION = { fast: 0.15, base: 0.22 } as const;

export const TRANSITION_FAST = { duration: DURATION.fast, ease: EASE_OUT } as const;
export const TRANSITION_BASE = { duration: DURATION.base, ease: EASE_OUT } as const;
