// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

// Labels in the caption role, written out rather than through `Text`, so the
// grid reads the same whatever state the primitives are in.
const labelClass = "text-caption text-fg-secondary";

/**
 * Shows a prop's values side by side: a `StoryGridRow` per value, and, when a
 * second prop varies too, a labelled column per value of that one. Pass the
 * column labels, or how many cells each row holds when the columns need none.
 */
export function StoryGrid({
  children,
  columns = 1,
}: {
  children: ReactNode;
  columns?: number | readonly string[];
}) {
  const count = typeof columns === "number" ? columns : columns.length;
  return (
    <div
      className="grid items-center justify-items-start gap-3"
      style={{ gridTemplateColumns: `auto repeat(${String(count)}, auto)` }}
    >
      {typeof columns === "number" ? null : (
        <>
          <span />
          {columns.map((column) => (
            <span className={labelClass} key={column}>
              {column}
            </span>
          ))}
        </>
      )}
      {children}
    </div>
  );
}

/** One value's row: its label, then a cell per column. */
export function StoryGridRow({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <>
      <span className={labelClass}>{label}</span>
      {children}
    </>
  );
}
