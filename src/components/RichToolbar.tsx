import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { ParseKeys } from "i18next";
import { Bold, Eraser, Italic, List, ListOrdered, Strikethrough, Underline } from "lucide-react";
import { IconButton } from "./IconButton";

// Only formatting the backend keeps: bold, italic, underline, strikethrough and lists.
const TOOLS = [
    { command: "bold", labelKey: "rich.bold", shortcut: "Ctrl+B", icon: Bold },
    { command: "italic", labelKey: "rich.italic", shortcut: "Ctrl+I", icon: Italic },
    { command: "underline", labelKey: "rich.underline", shortcut: "Ctrl+U", icon: Underline },
    { command: "strikeThrough", labelKey: "rich.strikethrough", shortcut: "", icon: Strikethrough },
    { command: "insertUnorderedList", labelKey: "rich.bulletedList", shortcut: "", icon: List },
    { command: "insertOrderedList", labelKey: "rich.numberedList", shortcut: "", icon: ListOrdered },
    { command: "removeFormat", labelKey: "rich.clearFormatting", shortcut: "", icon: Eraser },
] as const satisfies readonly { command: string; labelKey: ParseKeys; shortcut: string; icon: unknown }[];

export interface RichToolbarProps {
    /** The editable element the commands apply to. */
    editor: React.RefObject<HTMLElement | null>;
    disabled?: boolean;
}

/**
 * A small formatting bar for a rich field. Buttons keep the selection in the text by not taking
 * focus on press, and show which formats apply at the caret. Arrow keys move between the buttons.
 */
export function RichToolbar({ editor, disabled }: RichToolbarProps) {
    const { t } = useTranslation();
    const [active, setActive] = useState<Set<string>>(new Set());
    const bar = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const update = () => {
            const el = editor.current;
            if (!el || !el.contains(document.getSelection()?.anchorNode ?? null)) return;
            setActive(new Set(TOOLS.filter((t) => t.command !== "removeFormat" && document.queryCommandState(t.command)).map((t) => t.command)));
        };
        document.addEventListener("selectionchange", update);
        return () => document.removeEventListener("selectionchange", update);
    }, [editor]);

    const apply = (command: string) => {
        editor.current?.focus();
        document.execCommand(command);
    };

    const onKeyDown = (e: React.KeyboardEvent) => {
        if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
        const buttons = [...(bar.current?.querySelectorAll("button") ?? [])];
        const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
        const next = (at + (e.key === "ArrowRight" ? 1 : -1) + buttons.length) % buttons.length;
        buttons[next]?.focus();
        e.preventDefault();
    };

    return (
        <div
            ref={bar}
            role="toolbar"
            aria-label={t("rich.toolbar")}
            aria-orientation="horizontal"
            onKeyDown={onKeyDown}
            // Pressing a button must not move the focus out of the text, or the selection is lost.
            onMouseDown={(e) => e.preventDefault()}
            className="flex flex-wrap items-center gap-0.5"
        >
            {TOOLS.map(({ command, labelKey, shortcut, icon }, index) => (
                <IconButton
                    key={command}
                    label={shortcut ? `${t(labelKey)} (${shortcut})` : t(labelKey)}
                    icon={icon}
                    iconSize={16}
                    disabled={disabled}
                    pressed={active.has(command)}
                    onClick={() => apply(command)}
                    tabIndex={index === 0 ? 0 : -1}
                />
            ))}
        </div>
    );
}
