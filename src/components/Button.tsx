import { HTMLMotionProps, motion } from "motion/react";
import { Loader2 } from "lucide-react";
import { TRANSITION_FAST } from "../lib/motion";

type Variant = "primary" | "outline" | "ghost" | "danger" | "plain";

const VARIANTS: Record<Variant, string> = {
    primary: "bg-accent text-white hover:brightness-110",
    outline: "border border-line text-muted hover:bg-white/10 hover:text-white",
    ghost: "text-muted hover:bg-white/10 hover:text-white",
    danger: "bg-danger text-white hover:brightness-110",
    /** No colors of its own, for a button that brings its own fill. */
    plain: "",
};

const BASE =
    "inline-flex items-center justify-center gap-1.5 rounded-lg px-3 py-2 text-ui font-medium transition-colors disabled:pointer-events-none disabled:opacity-50";

/** The classes of a button, for a link that should look like one. */
export function buttonClass(variant: Variant = "primary", extra = ""): string {
    return `${BASE} ${VARIANTS[variant]} ${extra}`;
}

export interface ButtonProps extends Omit<HTMLMotionProps<"button">, "children"> {
    variant?: Variant;
    /** Shows a spinner and blocks presses while an action is running. */
    loading?: boolean;
    children: React.ReactNode;
}

/** The app's button, with a small press animation. */
export function Button({ variant = "primary", loading, disabled, className, children, type = "button", ...rest }: ButtonProps) {
    return (
        <motion.button
            type={type}
            disabled={disabled || loading}
            aria-busy={loading || undefined}
            whileTap={{ scale: 0.97 }}
            transition={TRANSITION_FAST}
            className={buttonClass(variant, className)}
            {...rest}
        >
            {loading && <Loader2 size={16} className="animate-spin" aria-hidden />}
            {children}
        </motion.button>
    );
}
