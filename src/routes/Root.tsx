import { Outlet } from "react-router";
import { AppNavigationBar } from "../components/AppNavigationBar";

export default function Root() {
  return (
    <div className="h-full bg-panel text-white">
      <AppNavigationBar />
      <main className="h-full overflow-y-auto pb-16 md:pb-0 md:pl-[72px]">
        <Outlet />
      </main>
    </div>
  );
}
