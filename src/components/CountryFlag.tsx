import type { ComponentType, SVGProps } from "react";
import { Globe } from "lucide-react";
import * as Flags from "country-flag-icons/react/3x2";

type FlagComponent = ComponentType<SVGProps<SVGSVGElement> & { title?: string }>;

export interface CountryFlagProps {
    /** ISO 3166-1 alpha-2 code, such as `US`. */
    code: string | null;
    className?: string;
}

/** The flag of a country, or a globe when there is no country or no flag for it. */
export function CountryFlag({ code, className }: CountryFlagProps) {
    const Flag = code ? (Flags as unknown as Record<string, FlagComponent>)[code.toUpperCase()] : undefined;
    if (!Flag) {
        return <Globe className={className} aria-hidden />;
    }
    return <Flag className={`rounded-sm ${className ?? ""}`} aria-hidden />;
}
