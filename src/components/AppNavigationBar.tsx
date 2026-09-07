import { motion } from "motion/react";
import { CircleUserRound } from "lucide-react";
import { buttons } from "../data/NavigationButton";
import { NavigationButton } from "./NavigationButton";
import verbvmLogo from "../assets/verbvm-white-logo.png";

const railVariants = { collapsed: { width: 72 }, expanded: { width: 240 } };
const fadeVariants = { collapsed: { opacity: 0 }, expanded: { opacity: 1 } };

export function AppNavigationBar() {
  return (
    <>
      <nav
        className="
          fixed inset-x-0 bottom-0 z-50 flex h-16 flex-row items-center justify-around gap-1
          border-t border-panel-border bg-panel px-2 pb-[env(safe-area-inset-bottom)]
          md:hidden
        "
      >
        {buttons.map((button) => (
          <NavigationButton key={button.to} {...button} variant="mobile" />
        ))}
      </nav>

      <motion.nav
        initial="collapsed"
        whileHover="expanded"
        variants={railVariants}
        transition={{ duration: 0.2, ease: "easeInOut" }}
        className="
          fixed inset-y-0 left-0 z-50 hidden flex-col overflow-hidden
          border-r border-panel-border bg-panel p-3
          md:flex
        "
      >
        <div className="flex items-center gap-3 overflow-hidden px-1 pb-4">
          <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-vocabulary p-1.5">
            <img src={verbvmLogo} alt="Verbvm" className="h-full w-full object-contain" />
          </div>
          <motion.span
            variants={fadeVariants}
            className="overflow-hidden whitespace-nowrap text-lg font-semibold"
          >
            Verbvm
          </motion.span>
        </div>

        <div className="flex flex-1 flex-col gap-1">
          {buttons.map((button) => (
            <NavigationButton key={button.to} {...button} variant="desktop" />
          ))}
        </div>

        <div className="flex items-center gap-3 overflow-hidden border-t border-panel-border px-1 pt-3">
          <CircleUserRound size={36} className="shrink-0 text-white/70" aria-hidden />
          <motion.span
            variants={fadeVariants}
            className="overflow-hidden whitespace-nowrap text-sm text-white/70"
          >
            Login
          </motion.span>
        </div>
      </motion.nav>
    </>
  );
}
