// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, use } from "react";

import { cn } from "../../lib/styling";

import { IslandAsideContext } from "./island-context";

// A part of the section that reaches past the row rather than widening it,
// such as a column of choices: the black steps out past the row to hold it.
// It opens at the section's start for a section hung from the row's end, so
// an island standing at the end of what it sits on grows away from that
// edge, and at its end otherwise; it puts itself first in its row where it
// opens at the start, wherever it is written. The island follows its width,
// so it opens and closes by growing; empty, it takes no room.
export function IslandAside({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  const { isAtStart, ref } = use(IslandAsideContext);
  return (
    <div className={cn(className, isAtStart && "order-first")} ref={ref}>
      {children}
    </div>
  );
}
