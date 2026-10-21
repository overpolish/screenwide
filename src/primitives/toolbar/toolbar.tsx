// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Toolbar as AriaToolbar,
  ToolbarProps as AriaToolbarProps,
} from "react-aria-components";

import { cn } from "../../lib/styling";

import { ToolbarDisabledContext } from "./toolbar-disabled";

type ToolbarProps = Omit<AriaToolbarProps, "aria-label" | "className"> & {
  "aria-label": string;
  className?: string;
  /** Disables every `ToolbarTool` in it, leaving the chosen one shown. */
  isDisabled?: boolean;
};

export function Toolbar({
  className,
  isDisabled = false,
  orientation = "horizontal",
  ...props
}: ToolbarProps) {
  return (
    <ToolbarDisabledContext value={isDisabled}>
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
    </ToolbarDisabledContext>
  );
}
