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
      // Independent tools sit at the standard sibling spacing, each with its
      // own hover and pressed fill.
      className={cn(
        "flex gap-control",
        orientation === "horizontal" ? "flex-row" : "flex-col",
        className,
      )}
      orientation={orientation}
    />
  );
}
