import { useEffect, useId, useRef, useState } from "react";
import { CodeXml, Paperclip } from "lucide-react";
import { useTranslation } from "react-i18next";
import { errorClass } from "./formStyles";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import { RichToolbar } from "./RichToolbar";
import { Media } from "../types/Media";

export interface RichInputFieldProps {
    label: string;
    /** The HTML content. Only read when the field is mounted: the editor keeps its own DOM. */
    value: string;
    onChange: (value: string) => void;
    allowMedia?: boolean;
    media?: Media[];
    /** Opens the file picker for this field. */
    onAttachMedia?: () => void;
    onRemoveMedia?: (sha256: string) => void;
    required?: boolean;
    error?: string;
}

function escapeHtml(value: string) {
    return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// Light syntax highlighting: it only colors tag tokens (<b>, </b>, etc.),
// the rest of the text keeps the default color.
function highlightHtml(html: string) {
    return html
        .split(/(<\/?[a-zA-Z][^>]*>)/g)
        .map((part) => (/^<\/?[a-zA-Z]/.test(part) ? `<span class="text-rose-400">${escapeHtml(part)}</span>` : escapeHtml(part)))
        .join("");
}

export function RichInputField({
    label,
    value,
    onChange,
    allowMedia,
    media,
    onAttachMedia,
    onRemoveMedia,
    required,
    error,
}: RichInputFieldProps) {
    const { t } = useTranslation();
    const [showHtml, setShowHtml] = useState(false);
    const id = useId();
    const labelId = `${id}-label`;
    const errorId = `${id}-error`;
    const [content, setContent] = useState(value);
    const richTextRef = useRef<HTMLDivElement>(null);
    const htmlHighlightRef = useRef<HTMLPreElement>(null);
    const editedFromRichText = useRef(false);

    // The contentEditable is never "controlled" through JSX: reapplying innerHTML on
    // every render would reset the cursor to the end of the text on every keystroke.
    // Its DOM is only touched here, and only when the change to `content` did not
    // come from the div itself.
    useEffect(() => {
        if (editedFromRichText.current) {
            editedFromRichText.current = false;
            return;
        }
        const el = richTextRef.current;
        if (el && el.innerHTML !== content) {
            el.innerHTML = content;
        }
    }, [content]);

    function updateContent(newContent: string) {
        setContent(newContent);
        onChange(newContent);
    }

    function handleRichInput(e: React.FormEvent<HTMLDivElement>) {
        editedFromRichText.current = true;
        updateContent(e.currentTarget.innerHTML);
    }

    function handleHtmlChange(e: React.ChangeEvent<HTMLTextAreaElement>) {
        updateContent(e.target.value);
    }

    function handleHtmlScroll(e: React.UIEvent<HTMLTextAreaElement>) {
        if (htmlHighlightRef.current) {
            htmlHighlightRef.current.scrollTop = e.currentTarget.scrollTop;
            htmlHighlightRef.current.scrollLeft = e.currentTarget.scrollLeft;
        }
    }

    return (
        <div>
            <div className="mb-2 flex items-center justify-between">
                <span id={labelId} className="text-ui font-medium text-muted">
                    {label}
                    {required && (
                        <span className="ml-1 text-subtle" aria-hidden>
                            *
                        </span>
                    )}
                </span>
                <div className="flex items-center gap-0.5">
                    <IconButton label={t("rich.editHtml")} icon={CodeXml} iconSize={16} pressed={showHtml} onClick={() => setShowHtml((v) => !v)} />
                    {allowMedia && (
                        <IconButton label={t("media.attach")} icon={Paperclip} iconSize={16} onClick={onAttachMedia} />
                    )}
                </div>
            </div>

            {/* One box: the formatting bar, the text and, when open, the HTML source it produces. */}
            <div
                className={`overflow-hidden rounded-xl border bg-white/5 transition-colors focus-within:border-accent focus-within:ring-2 focus-within:ring-accent/40 ${
                    error ? "border-stack/70 focus-within:ring-stack/30" : "border-line hover:border-line-strong"
                }`}
            >
                <div className="border-b border-line px-1.5 py-0.5">
                    <RichToolbar editor={richTextRef} />
                </div>

                <div
                    ref={richTextRef}
                    id={id}
                    contentEditable
                    role="textbox"
                    aria-multiline="true"
                    aria-labelledby={labelId}
                    aria-required={required || undefined}
                    aria-invalid={error ? true : undefined}
                    aria-describedby={error ? errorId : undefined}
                    onInput={handleRichInput}
                    suppressContentEditableWarning
                    className="min-h-24 w-full bg-transparent px-4 py-3 text-lead text-ink outline-none"
                />

                {showHtml && (
                    <div className="border-t border-dashed border-line-strong bg-black/25">
                        <p className="px-4 pt-2 font-mono text-label text-subtle">HTML</p>
                        <div className="relative h-24">
                            <pre
                                ref={htmlHighlightRef}
                                aria-hidden
                                className="pointer-events-none absolute inset-0 overflow-auto whitespace-pre-wrap break-all px-4 pb-3 pt-1 font-mono text-meta leading-relaxed text-ink"
                                dangerouslySetInnerHTML={{ __html: highlightHtml(content) }}
                            />
                            <textarea
                                value={content}
                                onChange={handleHtmlChange}
                                onScroll={handleHtmlScroll}
                                spellCheck={false}
                                aria-label={t("rich.htmlSource")}
                                className="absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre-wrap break-all bg-transparent px-4 pb-3 pt-1 font-mono text-meta leading-relaxed text-transparent caret-white outline-none"
                            />
                        </div>
                    </div>
                )}
            </div>

            {error && (
                <p id={errorId} className={errorClass}>
                    {error}
                </p>
            )}

            {allowMedia && (
                <div className="mt-2">
                    <MediaPreview media={media} onRemove={onRemoveMedia} />
                </div>
            )}
        </div>
    );
}
