import { AnimatePresence, motion, MotionConfig } from "motion/react";
import { useLocation, useOutlet } from "react-router";
import { AppNavigationBar } from "../components/AppNavigationBar";

export default function Root() {
  const location = useLocation();
  // The outlet element is kept by AnimatePresence while the old screen fades out, so the old
  // screen does not turn into the new one halfway through.
  const outlet = useOutlet();

  return (
    // Animations are turned off for people who ask the system for reduced motion.
    <MotionConfig reducedMotion="user">
      <div className="h-full bg-panel text-white">
        <AppNavigationBar />
        <main className="h-full overflow-y-auto pb-16 md:pb-0 md:pl-[72px]">
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={location.pathname}
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
            >
              {outlet}
            </motion.div>
          </AnimatePresence>
        </main>
      </div>
    </MotionConfig>
  );
}
