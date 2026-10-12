import { KeyboardEvent, useEffect, useId, useRef, useState } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Check, ChevronDown } from "lucide-react";
import { controlClass } from "./formStyles";
import { TRANSITION_FAST } from "../lib/motion";

export interface SelectOption {
    value: string;
    label: string;
}

export interface SelectProps {
    /** Lets an external `<label htmlFor>` name the control. */
    id?: string;
    options: SelectOption[];
    value: string;
    onChange: (value: string) => void;
    disabled?: boolean;
}

/**
 * A dropdown that matches the app's dark theme, in place of the native `<select>`, whose list
 * is drawn by the system and cannot be styled. It follows the ARIA select-only combobox pattern:
 * arrow keys, Home and End move, Enter and Space choose, Escape and Tab close.
 */
export function Select({ id, options, value, onChange, disabled }: SelectProps) {
    const listId = useId();
    const [open, setOpen] = useState(false);
    const [active, setActive] = useState(0);
    const container = useRef<HTMLDivElement>(null);
    const list = useRef<HTMLUListElement>(null);

    const selectedIndex = Math.max(
        0,
        options.findIndex((o) => o.value === value),
    );
    const selected = options.find((o) => o.value === value);

    const openList = () => {
        setActive(selectedIndex);
        setOpen(true);
    };

    const choose = (index: number) => {
        const option = options[index];
        if (option) onChange(option.value);
        setOpen(false);
    };

    useEffect(() => {
        if (!open) return;
        const onPointerDown = (e: PointerEvent) => {
            if (!container.current?.contains(e.target as Node)) setOpen(false);
        };
        document.addEventListener("pointerdown", onPointerDown);
        return () => document.removeEventListener("pointerdown", onPointerDown);
    }, [open]);

    // Keep the highlighted option visible when the list is longer than its box.
    useEffect(() => {
        if (!open) return;
        list.current?.children[active]?.scrollIntoView({ block: "nearest" });
    }, [open, active]);

    const onKeyDown = (e: KeyboardEvent<HTMLButtonElement>) => {
        if (disabled || options.length === 0) return;
        const last = options.length - 1;

        if (!open) {
            if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
                e.preventDefault();
                openList();
            }
            return;
        }

        switch (e.key) {
            case "ArrowDown":
                e.preventDefault();
                setActive((i) => Math.min(i + 1, last));
                break;
            case "ArrowUp":
                e.preventDefault();
                setActive((i) => Math.max(i - 1, 0));
                break;
            case "Home":
                e.preventDefault();
                setActive(0);
                break;
            case "End":
                e.preventDefault();
                setActive(last);
                break;
            case "Enter":
            case " ":
                e.preventDefault();
                choose(active);
                break;
            case "Escape":
                // Closes the list only, not a dialog around it.
                e.preventDefault();
                e.stopPropagation();
                setOpen(false);
                break;
            case "Tab":
                setOpen(false);
                break;
        }
    };

    return (
        <div ref={container} className="relative">
            <button
                id={id}
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded={open}
                aria-controls={listId}
                aria-activedescendant={open ? `${listId}-${active}` : undefined}
                disabled={disabled || options.length === 0}
                onClick={() => (open ? setOpen(false) : openList())}
                onKeyDown={onKeyDown}
                className={`${controlClass} flex items-center justify-between gap-2 text-left`}
            >
                <span className="truncate">{selected?.label ?? ""}</span>
                <ChevronDown
                    size={16}
                    aria-hidden
                    className={`shrink-0 text-muted transition-transform ${open ? "rotate-180" : ""}`}
                />
            </button>

            <AnimatePresence>
                {open && (
                    <motion.ul
                        ref={list}
                        id={listId}
                        role="listbox"
                        initial={{ opacity: 0, y: -4, scale: 0.98 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: -4, scale: 0.98 }}
                        transition={TRANSITION_FAST}
                        className="absolute left-0 right-0 top-full z-30 mt-1 max-h-56 origin-top overflow-auto rounded-xl border border-line bg-[#2c2d32] py-1 shadow-xl shadow-black/40"
                    >
                        {options.map((option, index) => {
                            const isSelected = option.value === value;
                            return (
                                <li
                                    key={option.value}
                                    id={`${listId}-${index}`}
                                    role="option"
                                    aria-selected={isSelected}
                                    onPointerEnter={() => setActive(index)}
                                    onClick={() => choose(index)}
                                    className={`flex cursor-pointer items-center justify-between gap-2 px-3 py-2 text-sm ${
                                        index === active ? "bg-white/10" : ""
                                    } ${isSelected ? "text-white" : "text-muted"}`}
                                >
                                    <span className="truncate">{option.label}</span>
                                    {isSelected && <Check size={14} aria-hidden className="shrink-0 text-accent-ink" />}
                                </li>
                            );
                        })}
                    </motion.ul>
                )}
            </AnimatePresence>
        </div>
    );
}
