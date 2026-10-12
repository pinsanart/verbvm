import { HTMLMotionProps, motion } from "motion/react";

type Variant = "primary" | "outline" | "ghost" | "danger";

const VARIANTS: Record<Variant, string> = {
    primary: "bg-vocabulary text-white hover:brightness-110",
    outline: "border border-white/10 text-white/70 hover:bg-white/10 hover:text-white",
    ghost: "text-muted hover:bg-white/10 hover:text-white",
    danger: "bg-stack text-white hover:brightness-110",
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
            transition={{ duration: 0.1 }}
            className={`inline-flex items-center justify-center gap-1.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50 ${VARIANTS[variant]} ${className ?? ""}`}
            {...rest}
        >
            {children}
        </motion.button>
    );
}
