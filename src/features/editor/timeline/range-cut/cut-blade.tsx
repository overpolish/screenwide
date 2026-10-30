// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors } from "lucide-react";

// Where a press will cut while the scissors are in hand, with the Cut tool
// on or ⌥ held: it follows the pointer, so what a press does shows before
// it is made.

export function CutBlade({
  x,
}: {
  /** Where the pointer is, as a share of the strip's width. */
  x: number;
}) {
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute top-1.5 bottom-0 z-30 w-0"
      dir="ltr"
      style={{ insetInlineStart: `${String(x * 100)}%` }}
    >
      <span className="absolute inset-y-0 -start-px w-0.5 rounded-full bg-primary" />
      <span className="absolute -top-1.5 -start-2.5 flex size-5 items-center justify-center rounded-full bg-primary text-on-primary">
        <Scissors className="size-icon-xs" />
      </span>
    </span>
  );
}
