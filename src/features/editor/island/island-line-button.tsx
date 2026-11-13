// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { Button, type ButtonProps } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { IslandFill } from "../../../primitives/island/island-fill";

/** A line across a section that is pressed whole, such as a folded picker
 * or a sound in the library: quiet at rest, as the island's controls are,
 * filled under the pointer, and deeper while what it opens is open. Padded
 * the small inset, so what it shows starts on the island's text column
 * while its fill keeps to the section's edge. */
export function IslandLineButton({
  children,
  className,
  isChosen = false,
  ...props
}: Omit<ButtonProps, "children" | "className"> & {
  children: ReactNode;
  className?: string;
  /** Whether what it opens is open. */
  isChosen?: boolean;
}) {
  return (
    <Button
      {...props}
      className={cn(
        "group relative isolate flex h-island-control w-full cursor-default items-center gap-inline rounded-full px-inset-sm text-label text-fg outline-none",
        focusStyles,
        elementFocusVisible,
        className,
      )}
    >
      <IslandFill isChosen={isChosen} />
      {children}
    </Button>
  );
}
