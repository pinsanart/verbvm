import { NavLink } from "react-router";
import { motion } from "motion/react";
import { NavigationButtonType } from "../types/NavigationButton";

const fadeVariants = { collapsed: { opacity: 0 }, expanded: { opacity: 1 } };

type NavigationButtonProps = NavigationButtonType & {
  variant: "mobile" | "desktop";
};

export function NavigationButton({ to, label, icon: Icon, variant }: NavigationButtonProps) {
  if (variant === "mobile") {
    return (
      <NavLink
        to={to}
        end={to === "/"}
        className={({ isActive }) =>
          [
            "flex flex-1 flex-col items-center justify-center gap-0.5 rounded-lg py-1 text-label transition-colors",
            isActive
              ? "bg-white/10 text-ink"
              : "text-muted hover:bg-white/5 hover:text-ink",
          ].join(" ")
        }
      >
        <Icon size={20} aria-hidden />
        <span className="leading-none">{label}</span>
      </NavLink>
    );
  }

  return (
    <NavLink
      to={to}
      end={to === "/"}
      className={({ isActive }) =>
        [
          "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-ui transition-colors",
          isActive
            ? "bg-white/10 text-ink"
            : "text-muted hover:bg-white/5 hover:text-ink",
        ].join(" ")
      }
    >
      <Icon size={20} className="shrink-0" aria-hidden />
      <motion.span variants={fadeVariants} className="overflow-hidden whitespace-nowrap">
        {label}
      </motion.span>
    </NavLink>
  );
}
