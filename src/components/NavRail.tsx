import { NavLink } from "react-router";

import { NAV_ITEMS } from "../lib/nav";

export default function NavRail() {
  return (
    <nav
      className="
        fixed inset-x-0 bottom-0 z-50 flex h-16 flex-row items-center justify-around
        border-t border-neutral-200 bg-white pb-[env(safe-area-inset-bottom)]
        md:static md:h-full md:w-16 md:flex-col md:justify-start md:gap-1
        md:border-t-0 md:border-r md:py-4
        dark:border-neutral-800 dark:bg-neutral-900
      "
    >
      {NAV_ITEMS.map(({ to, label, icon: Icon }) => (
        <NavLink
          key={to}
          to={to}
          end={to === "/"}
          className={({ isActive }) =>
            [
              "group relative flex flex-1 flex-col items-center justify-center gap-1 rounded-lg transition-colors",
              "md:h-12 md:w-12 md:flex-none",
              isActive
                ? "text-neutral-900 md:bg-neutral-100 dark:text-white dark:md:bg-neutral-800"
                : "text-neutral-500 hover:text-neutral-900 dark:hover:text-white",
            ].join(" ")
          }
        >
          <Icon size={20} aria-hidden />

          {/* label: só no mobile */}
          <span className="text-[10px] leading-none md:hidden">{label}</span>

          {/* tooltip: só no rail do desktop */}
          <span
            role="tooltip"
            className="
              pointer-events-none absolute left-full z-50 ml-2 hidden whitespace-nowrap
              rounded-md bg-neutral-900 px-2 py-1 text-xs text-white opacity-0
              transition-opacity md:block md:group-hover:opacity-100
              dark:bg-neutral-700
            "
          >
            {label}
          </span>
        </NavLink>
      ))}
    </nav>
  );
}