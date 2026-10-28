// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, use } from "react";

import { IslandAsideContext } from "./island-context";

/** A part of the section that reaches past the island's row rather than
 * widening it, such as a column of choices opened beside the section: the
 * row stays as wide as the rest of the section makes it, and the black
 * steps out past its end to hold this. It sits at the section's start for a
 * section hung from the row's end, and at its end otherwise; the island
 * follows its width as it changes, so it can open and close by growing.
 * Empty, it takes no room and the island is as it would be without it. */
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
