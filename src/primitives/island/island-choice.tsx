// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";

import { IslandFill } from "./island-fill";

// A set of the island's circles of which one is chosen, for a section under
// the row. Built on React Aria's toggle button group, which names it a radio
// group and moves between its circles with the arrow keys.
//
// Each circle has the island's fill (`IslandFill`), which is its hover and
// its choice, so a circle chosen under the pointer keeps the fill it already
// showed, one chosen from the keyboard grows its own, and the one left behind
// shrinks away; nothing travels across or vanishes at once.

export type IslandChoiceItem<Id extends string> = {
  /** What the circle shows: a glyph, a dot, a picture. */
  content: ReactNode;
  id: Id;
  /** Its accessible name; the island shows no tooltips. */
  label: string;
};

export function IslandChoice<Id extends string>({
  items,
  label,
  onChange,
  value,
}: {
  items: IslandChoiceItem<Id>[];
  label: string;
  onChange: (id: Id) => void;
  value: Id;
}) {
  return (
    <ToggleButtonGroup
      aria-label={label}
      className="flex gap-1"
      disallowEmptySelection
      // Single selection that cannot be emptied: the set holds the one chosen.
      onSelectionChange={(keys) => {
        for (const key of keys) onChange(key as Id);
      }}
      selectedKeys={[value]}
      selectionMode="single"
    >
      {items.map((item) => (
        <ToggleButton
          aria-label={item.label}
          className={cn(
            "relative isolate flex size-island-control shrink-0 items-center justify-center rounded-full",
            focusStyles,
            elementFocusVisible,
          )}
          id={item.id}
          key={item.id}
        >
          {({ isHovered, isSelected }) => (
            <>
              <IslandFill isChosen={isSelected} isHovered={isHovered} />
              {item.content}
            </>
          )}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
