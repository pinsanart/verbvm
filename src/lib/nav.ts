import { House, Plus, ClipboardCheck } from "lucide-react";
import type { LucideIcon } from "lucide-react";

export type NavItem = { to: string; label: string; icon: LucideIcon };

export const NAV_ITEMS: NavItem[] = [
  { to: "/", label: "Home", icon: House },
  { to: "/create", label: "Create", icon: Plus },
  { to: "/review", label: "Review", icon: ClipboardCheck },
];
