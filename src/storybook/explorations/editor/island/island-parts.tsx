// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { ListBox } from "../../../../primitives/listbox/listbox";
import { ListBoxItem } from "../../../../primitives/listbox-item/listbox-item";

// What the editor's island is made of besides the components it skins: what
// it holds, a labelled line of a section, and a list of ticked rows.

export type IslandContent = {
  /** What it came from, which says where the island opens: beside the
   * toolbar button for a pen or a mode, and over the strip for anything
   * chosen, an annotation over its bar wherever it was chosen. */
  from: { kind: "strip" } | { kind: "tool"; tool: string };
  key: string;
  /** The row's start, usually an `IslandRowTitle`, kept to the island's
   * start. */
  title: ReactNode;
  /** The row's end: what can be done to what is in hand. */
  actions?: ReactNode;
  /** What hangs under the row, and under which end: the colours hang
   * under the colour in hand at the row's end. */
  section?: { key: string; node: ReactNode; anchor?: "end" | "start" };
};

/** A labelled line of a section: its name, then its controls. */
export function SectionLine({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return <ControlRow title={label}>{() => children}</ControlRow>;
}

/** Rows ticked while chosen or on. A press reads which row it was, from
 * what the list would have changed, so one list serves a set of switches
 * and a choice of one alike: the owner says what the press does. */
export function CheckList({
  "aria-label": label,
  items,
  onToggle,
  selected,
}: {
  "aria-label": string;
  items: { id: string; label: string }[];
  onToggle: (id: string) => void;
  selected: string[];
}) {
  return (
    <ListBox
      aria-label={label}
      onSelectionChange={(keys) => {
        if (keys === "all") return;
        const pressed = items.find(
          ({ id }) => keys.has(id) !== selected.includes(id),
        );
        if (pressed) onToggle(pressed.id);
      }}
      selectedKeys={selected}
      selectionBehavior="toggle"
      selectionMode="multiple"
    >
      {items.map((item) => (
        <ListBoxItem id={item.id} key={item.id} textValue={item.label}>
          {item.label}
        </ListBoxItem>
      ))}
    </ListBox>
  );
}
