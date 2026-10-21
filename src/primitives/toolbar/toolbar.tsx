// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Toolbar as AriaToolbar,
  ToolbarProps as AriaToolbarProps,
} from "react-aria-components";

import { cn } from "../../lib/styling";

type ToolbarProps = Omit<AriaToolbarProps, "aria-label" | "className"> & {
  "aria-label": string;
  className?: string;
};

export function Toolbar({
  className,
  orientation = "horizontal",
  ...props
}: ToolbarProps) {
  return (
    <AriaToolbar
      {...props}
      // Tools sit 2px apart, so each one's hover fill and the chosen tool's
      // primary fill stay apart while the set still reads as one. Separate
      // groups are separate toolbars, spaced by whoever lays them out.
      className={cn(
        "flex gap-0.5",
        orientation === "horizontal" ? "flex-row" : "flex-col",
        className,
      )}
      orientation={orientation}
    />
  );
}
