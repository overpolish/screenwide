// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { Text } from "../components/base/text/text";

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
      className="grid items-center justify-items-start gap-section"
      style={{ gridTemplateColumns: `auto repeat(${String(count)}, auto)` }}
    >
      {typeof columns === "number" ? null : (
        <>
          <span />
          {columns.map((column) => (
            <Text key={column} variant="footnote">
              {column}
            </Text>
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
      <Text variant="footnote">{label}</Text>
      {children}
    </>
  );
}
