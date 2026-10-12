import { HTMLMotionProps, motion } from "motion/react";
import { TRANSITION_FAST } from "../lib/motion";

type Variant = "primary" | "outline" | "ghost" | "danger";

const VARIANTS: Record<Variant, string> = {
    primary: "bg-accent text-white hover:brightness-110",
    outline: "border border-line text-muted hover:bg-white/10 hover:text-white",
    ghost: "text-muted hover:bg-white/10 hover:text-white",
    danger: "bg-danger text-white hover:brightness-110",
};

export interface ButtonProps extends Omit<HTMLMotionProps<"button">, "children"> {
    variant?: Variant;
    children: React.ReactNode;
}

/** The app's button, with a small press animation. */
export function Button({ variant = "primary", className, children, type = "button", ...rest }: ButtonProps) {
    return (
        <motion.button
            type={type}
            whileTap={{ scale: 0.97 }}
            transition={TRANSITION_FAST}
            className={`inline-flex items-center justify-center gap-1.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50 ${VARIANTS[variant]} ${className ?? ""}`}
            {...rest}
        >
            {children}
        </motion.button>
    );
}
