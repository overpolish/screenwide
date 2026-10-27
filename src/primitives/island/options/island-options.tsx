// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { type ReactNode } from "react";

import { cn } from "../../../lib/styling";

// A control with choices behind it: a small chevron just clear of it opens
// them, plain on the black, so it reads as belonging to the control before
// it and not to the one after.
export function IslandOptions({
  children,
  isDisabled = false,
  isOpen,
  label,
  onOpen,
}: {
  children: ReactNode;
  isOpen: boolean;
  label: string;
  onOpen: () => void;
  isDisabled?: boolean;
}) {
  return (
    <span className="flex items-center">
      {children}
      <button
        aria-expanded={isOpen}
        aria-label={label}
        className={cn(
          "ms-0.5 flex h-island-control w-icon-sm items-center justify-center transition-[color,opacity] disabled:opacity-40",
          isOpen ? "text-fg" : "text-fg-tertiary enabled:hover:text-fg",
        )}
        disabled={isDisabled}
        onClick={onOpen}
        type="button"
      >
        <ChevronDown aria-hidden className="size-3" />
      </button>
    </span>
  );
}
