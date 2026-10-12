// Class names shared by the form controls, so inputs, textareas, selects and the rich editor
// look and behave alike.

/** The box of a text control: default, hover, focus and invalid states. */
export const controlClass =
    "w-full rounded-xl border border-line bg-white/5 px-3 py-2 text-body text-ink outline-none transition-colors hover:border-line-strong focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40 disabled:cursor-not-allowed disabled:opacity-50 aria-[invalid=true]:border-stack/70 aria-[invalid=true]:focus-visible:ring-stack/30";

/** The label above a control. */
export const labelClass = "mb-1.5 block text-meta font-medium text-muted";

/** The message under an invalid control. */
export const errorClass = "mt-1.5 text-meta text-stack-ink";
