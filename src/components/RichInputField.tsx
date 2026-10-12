import { useEffect, useRef, useState } from "react";
import { ChevronDown, CodeXml, Paperclip } from "lucide-react";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
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
    const [showHtml, setShowHtml] = useState(false);
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
            <div className="mb-1.5 flex items-center justify-between">
                <label className="text-xs font-medium uppercase tracking-wider text-muted">
                    {label}
                    {required && <span className="ml-1 text-subtle">*</span>}
                </label>
                <div className="flex items-center gap-0.5">
                    <IconButton label="Toggle HTML" icon={CodeXml} iconSize={14} onClick={() => setShowHtml((v) => !v)} />
                    {allowMedia && (
                        <IconButton label="Attach media" icon={Paperclip} iconSize={14} onClick={onAttachMedia} />
                    )}
                </div>
            </div>

            <div
                ref={richTextRef}
                contentEditable
                onInput={handleRichInput}
                suppressContentEditableWarning
                aria-invalid={error ? true : undefined}
                className={`min-h-20 w-full rounded-xl border bg-white/5 p-3 text-[15px] leading-6 text-white outline-none transition-colors focus:bg-white/[0.07] ${
                    error ? "border-stack/70" : "border-white/10 focus:border-white/30"
                }`}
            />

            {showHtml && (
                <div className="mt-2 overflow-hidden rounded-xl bg-black/30">
                    <div className="flex items-center gap-1 px-3 pt-2 text-muted">
                        <ChevronDown size={12} />
                    </div>
                    <div className="relative h-24">
                        <pre
                            ref={htmlHighlightRef}
                            aria-hidden
                            className="pointer-events-none absolute inset-0 overflow-auto whitespace-pre-wrap break-all p-3 pt-1 font-mono text-xs leading-relaxed text-white/90"
                            dangerouslySetInnerHTML={{ __html: highlightHtml(content) }}
                        />
                        <textarea
                            value={content}
                            onChange={handleHtmlChange}
                            onScroll={handleHtmlScroll}
                            spellCheck={false}
                            className="absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre-wrap break-all bg-transparent p-3 pt-1 font-mono text-xs leading-relaxed text-transparent caret-white outline-none"
                        />
                    </div>
                </div>
            )}

            {error && <p className="mt-1 text-xs text-stack">{error}</p>}

            {allowMedia && (
                <div className="mt-2">
                    <MediaPreview media={media} onRemove={onRemoveMedia} />
                </div>
            )}
        </div>
    );
}
