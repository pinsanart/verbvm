// Class names shared by the form controls, so inputs, textareas, selects and the rich editor
// look and behave alike.

const controlBase =
    "w-full rounded-xl border border-line bg-white/5 text-ink outline-none transition-colors hover:border-line-strong focus-visible:border-accent focus-visible:ring-2 focus-visible:ring-accent/40 disabled:cursor-not-allowed disabled:opacity-50 aria-[invalid=true]:border-stack/70 aria-[invalid=true]:focus-visible:ring-stack/30";

/** The box of a text control: default, hover, focus, invalid and disabled states. */
export const controlClass = `${controlBase} px-3 py-2 text-body`;

/** The same box for the text people write in a flashcard: bigger, with more room. */
export const editorControlClass = `${controlBase} px-4 py-3 text-lead`;

/** The label above a control. */
export const labelClass = "mb-1.5 block text-meta font-medium text-muted";

/** The label above a field of a flashcard. */
export const editorLabelClass = "mb-2 block text-ui font-medium text-muted";

/** The message under an invalid control. */
export const errorClass = "mt-1.5 text-meta text-stack-ink";
