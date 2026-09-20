import { useEffect, useRef, useState } from "react";
import { ChevronDown, CodeXml, Paperclip } from "lucide-react";
import { IconButton } from "./IconButton";
import { MediaPreview } from "./MediaPreview";
import { Field } from "../types/Flashcard";

export interface RichInputFieldProps {
    label: string;
    field: Field;
    onChange: (field: Field) => void;
    allowMedia?: boolean;
}

function escapeHtml(value: string) {
    return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// Realce leve de sintaxe: só colore os tokens de tag (<b>, </b>, etc.), o
// resto do texto continua com a cor padrão.
function highlightHtml(html: string) {
    return html
        .split(/(<\/?[a-zA-Z][^>]*>)/g)
        .map((part) => (/^<\/?[a-zA-Z]/.test(part) ? `<span class="text-rose-400">${escapeHtml(part)}</span>` : escapeHtml(part)))
        .join("");
}

export function RichInputField({ label, field, onChange, allowMedia }: RichInputFieldProps) {
    const [showHtml, setShowHtml] = useState(false);
    const [content, setContent] = useState(field.content);
    const richTextRef = useRef<HTMLDivElement>(null);
    const htmlHighlightRef = useRef<HTMLPreElement>(null);
    const fileInputRef = useRef<HTMLInputElement>(null);
    const editedFromRichText = useRef(false);

    // O contentEditable nunca é "controlado" via JSX — reaplicar innerHTML a cada
    // render resetaria o cursor pro fim do texto a cada tecla. O DOM dele só é
    // tocado aqui, e só quando a mudança de `content` não veio do próprio div.
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
        onChange({ ...field, content: newContent });
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

    function handleFilesSelected(_e: React.ChangeEvent<HTMLInputElement>) {
        // Estrutura visual apenas: anexar mídia de verdade (criar Media, atualizar
        // field.media) fica para uma etapa futura.
    }

    return (
        <div>
            <div className="mb-1.5 flex items-center justify-between">
                <label className="text-xs font-medium uppercase tracking-wide text-white/40">{label}</label>
                <div className="flex items-center gap-0.5">
                    <IconButton label="Toggle HTML" icon={CodeXml} iconSize={14} onClick={() => setShowHtml((v) => !v)} />
                    {allowMedia && (
                        <>
                            <IconButton label="Attach media" icon={Paperclip} iconSize={14} onClick={() => fileInputRef.current?.click()} />
                            <input ref={fileInputRef} type="file" accept="image/*,audio/*" multiple hidden onChange={handleFilesSelected} />
                        </>
                    )}
                </div>
            </div>

            <div
                ref={richTextRef}
                contentEditable
                onInput={handleRichInput}
                suppressContentEditableWarning
                className="min-h-20 w-full rounded-xl border border-white/10 bg-white/5 p-3 text-sm text-white outline-none transition-colors focus:border-white/30 focus:bg-white/[0.07]"
            />

            {showHtml && (
                <div className="mt-2 overflow-hidden rounded-xl bg-black/30">
                    <div className="flex items-center gap-1 px-3 pt-2 text-white/40">
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

            {allowMedia && (
                <div className="mt-2">
                    <MediaPreview media={field.media} />
                </div>
            )}
        </div>
    );
}
