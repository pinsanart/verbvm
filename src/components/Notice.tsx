import { motion } from "motion/react";
import { TRANSITION_BASE } from "../lib/motion";

export interface NoticeProps {
    tone: "success" | "error" | "info";
    children: React.ReactNode;
}

const TONES = {
    success: "border-orthographic/50 text-orthographic",
    error: "border-stack/70 text-stack-ink",
    info: "border-line-strong text-muted",
} as const;

/**
 * A message above a screen's content. Render it inside `AnimatePresence`, with a `key`, so it
 * animates in and out.
 */
export function Notice({ tone, children }: NoticeProps) {
    return (
        <motion.div
            role={tone === "error" ? "alert" : "status"}
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={TRANSITION_BASE}
            className={`rounded-lg border px-3 py-2 text-ui ${TONES[tone]}`}
        >
            {children}
        </motion.div>
    );
}
