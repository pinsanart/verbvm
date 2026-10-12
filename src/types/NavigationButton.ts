import { LucideIcon } from "lucide-react"
import type { ParseKeys } from "i18next"

export type NavigationButtonType = {
    icon: LucideIcon,
    labelKey: ParseKeys,
    to: string
}
