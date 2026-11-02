// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, use } from "react";

import { IslandAsideContext } from "./island-context";

// A part of the section that reaches past the row rather than widening it,
// such as a column of choices: the black steps out past the row's end to hold
// it. It sits at the section's start for a section hung from the row's end,
// and at its end otherwise. The island follows its width, so it opens and
// closes by growing; empty, it takes no room.
export function IslandAside({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={className} ref={use(IslandAsideContext)}>
      {children}
    </div>
  );
}
