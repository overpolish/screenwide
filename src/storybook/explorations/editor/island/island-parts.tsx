// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { type ReactNode } from "react";

import { IslandListRow } from "../../../../primitives/island/island-controls";
import { Text } from "../../../../primitives/text/text";

// What the editor's island is made of besides the island's own controls:
// the row's title, and the labelled lines a section is built from.

export type IslandContent = {
  /** What it came from, which says where the island opens: beside the
   * toolbar button for a pen or a mode, and over the strip for anything
   * chosen, an annotation over its bar wherever it was chosen. */
  from: { kind: "strip" } | { kind: "tool"; tool: string };
  key: string;
  /** The row's start, usually a `RowTitle`, kept to the island's start. */
  title: ReactNode;
  /** The row's end: what can be done to what is in hand. */
  actions?: ReactNode;
  /** What hangs under the row, and under which end: the colours hang
   * under the colour in hand at the row's end. */
  section?: { key: string; node: ReactNode; anchor?: "end" | "start" };
};

/** What is in hand, said in the row: a glyph where it has one, its name,
 * and a line of when or how long under it. */
export function RowTitle({
  glyph,
  note,
  title,
}: {
  title: string;
  glyph?: ReactNode;
  note?: string;
}) {
  return (
    <span className="flex min-w-0 items-center gap-1 pe-2">
      {glyph ? (
        <span className="flex size-island-control shrink-0 items-center justify-center text-fg-secondary">
          {glyph}
        </span>
      ) : (
        <span aria-hidden className="w-2" />
      )}
      <span className="flex min-w-0 flex-col">
        <Text as="span" className="max-w-72 truncate" variant="label">
          {title}
        </Text>
        {note ? (
          <Text
            as="span"
            className="max-w-72 truncate text-fg-secondary"
            variant="caption"
          >
            {note}
          </Text>
        ) : null}
      </span>
    </span>
  );
}

/** A labelled line of a section: its name, then its circles. */
export function SectionLine({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <div className="flex items-center justify-between gap-separate">
      <Text as="span" className="ps-1 text-fg-secondary" variant="caption">
        {label}
      </Text>
      <div className="flex items-center gap-1">{children}</div>
    </div>
  );
}

/** A row of a list in a section, ticked when it is the one chosen or on. */
export function CheckRow({
  children,
  isChosen,
  onPress,
}: {
  children: ReactNode;
  isChosen: boolean;
  onPress: () => void;
}) {
  return (
    <IslandListRow isChosen={isChosen} onPress={onPress}>
      <span className="flex grow items-center gap-inline">{children}</span>
      <Check
        aria-hidden
        className={isChosen ? "size-icon" : "size-icon opacity-0"}
      />
    </IslandListRow>
  );
}
