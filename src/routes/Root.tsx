import { MotionConfig } from "motion/react";
import { Outlet } from "react-router";
import { AppNavigationBar } from "../components/AppNavigationBar";

export default function Root() {
  return (
    // Animations are turned off for people who ask the system for reduced motion.
    <MotionConfig reducedMotion="user">
      <div className="h-full bg-canvas text-ink">
        <AppNavigationBar />
        <main className="h-full overflow-y-auto pb-16 md:pb-0 md:pl-[72px]">
          <Outlet />
        </main>
      </div>
    </MotionConfig>
  );
}
