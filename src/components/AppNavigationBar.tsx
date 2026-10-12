import { useState } from "react";
import { motion } from "motion/react";
import { TRANSITION_BASE } from "../lib/motion";
import { CircleUserRound } from "lucide-react";
import { buttons } from "../data/NavigationButton";
import { NavigationButton } from "./NavigationButton";
import verbvmLogo from "../assets/verbvm-white-logo.png";

const railVariants = { collapsed: { width: 72 }, expanded: { width: 240 } };
const fadeVariants = { collapsed: { opacity: 0 }, expanded: { opacity: 1 } };

export function AppNavigationBar() {
  // The rail opens on hover and when keyboard focus is inside it, so it works without a mouse.
  const [expanded, setExpanded] = useState(false);

  return (
    <>
      <nav
        aria-label="Main"
        className="
          fixed inset-x-0 bottom-0 z-50 flex h-16 flex-row items-center justify-around gap-1
          border-t border-line bg-panel px-2 pb-[env(safe-area-inset-bottom)]
          md:hidden
        "
      >
        {buttons.map((button) => (
          <NavigationButton key={button.to} {...button} variant="mobile" />
        ))}
      </nav>

      <motion.nav
        aria-label="Main"
        initial={false}
        animate={expanded ? "expanded" : "collapsed"}
        onHoverStart={() => setExpanded(true)}
        onHoverEnd={() => setExpanded(false)}
        onFocusCapture={() => setExpanded(true)}
        onBlurCapture={(e) => {
          if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setExpanded(false);
        }}
        variants={railVariants}
        transition={TRANSITION_BASE}
        className="
          fixed inset-y-0 left-0 z-50 hidden flex-col overflow-hidden
          border-r border-line bg-panel p-3
          md:flex
        "
      >
        <div className="flex items-center gap-3 overflow-hidden px-0.5 pb-5">
          <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-accent p-2">
            <img src={verbvmLogo} alt="Verbvm" className="h-full w-full object-contain" />
          </div>
          <motion.span
            variants={fadeVariants}
            className="overflow-hidden whitespace-nowrap text-heading font-semibold tracking-tight"
          >
            Verbvm
          </motion.span>
        </div>

        <div className="flex flex-1 flex-col gap-1">
          {buttons.map((button) => (
            <NavigationButton key={button.to} {...button} variant="desktop" />
          ))}
        </div>

        <div className="flex items-center gap-3 overflow-hidden border-t border-line px-3 pt-4 pb-1">
          <CircleUserRound size={24} className="shrink-0 text-muted" aria-hidden />
          <motion.span
            variants={fadeVariants}
            className="overflow-hidden whitespace-nowrap text-ui text-muted"
          >
            Login
          </motion.span>
        </div>
      </motion.nav>
    </>
  );
}
