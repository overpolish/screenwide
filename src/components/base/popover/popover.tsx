// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import {
  Dialog,
  Popover as AriaPopover,
  type PopoverProps as AriaPopoverProps,
} from "react-aria-components";

import { cn } from "../../../lib/styling";

/** The gap between a trigger and what it opens, in logical px. */
const TRIGGER_GAP = 4;

type PopoverProps = Omit<AriaPopoverProps, "children" | "className"> & {
  /** Names the dialog for assistive technology. */
  "aria-label": string;
  children: ReactNode;
  className?: string;
};

/**
 * A small dialog hung off the control that opens it, on the surface the
 * Select's list uses: for a few choices that would crowd the row they belong
 * to. It goes inside a React Aria `DialogTrigger` beside its trigger, and an
 * outside press or Escape closes it.
 *
 * In-page, so it is for windows with nothing native drawn over the page; the
 * Editor's pop-ups go through the popup panel window instead.
 */
export function Popover({
  "aria-label": ariaLabel,
  children,
  className,
  placement = "bottom",
  ...props
}: PopoverProps) {
  return (
    <AriaPopover
      {...props}
      className="data-[entering]:animate-in data-[entering]:fade-in data-[exiting]:animate-out data-[exiting]:fade-out"
      offset={TRIGGER_GAP}
      placement={placement}
    >
      <Dialog
        aria-label={ariaLabel}
        className={cn(
          "rounded-panel bg-popover p-control-inset shadow-md inset-ring inset-ring-popover-stroke outline-none",
          className,
        )}
      >
        {children}
      </Dialog>
    </AriaPopover>
  );
}
