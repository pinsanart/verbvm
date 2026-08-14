import { Outlet } from "react-router";

import NavRail from "../components/NavRail";

export default function Root() {
  return (
    <div className="flex h-full bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <NavRail />
      <main className="flex-1 overflow-y-auto pb-16 md:pb-0">
        <Outlet />
      </main>
    </div>
  );
}
