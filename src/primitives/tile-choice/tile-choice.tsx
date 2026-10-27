// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { useSkin } from "../skin/skin";

// One of several looks, each a tile with a picture of it over its name,
// three to a row: a scene's arrangement, a caption template. Built on React
// Aria's toggle button group, which names it a radio group and moves between
// tiles with the arrow keys. A tile without a picture keeps its room, filled,
// so the rows stay even while pictures load. The chosen tile is a quiet
// choice made in place, so it takes the selected surface with its name
// unchanged; on the island, the island's stronger fill. Whatever holds it
// scrolls it when there are more rows than room.

export type TileChoiceItem<Id extends string> = {
  id: Id;
  label: string;
  /** What the tile shows above its name, cropped to 16:9. */
  picture?: ReactNode;
};

export function TileChoice<Id extends string>({
  "aria-label": label,
  items,
  onChange,
  value,
}: {
  "aria-label": string;
  items: TileChoiceItem<Id>[];
  onChange: (id: Id) => void;
  value: Id;
}) {
  const isIsland = useSkin() === "island";
  return (
    <ToggleButtonGroup
      aria-label={label}
      className="grid grid-cols-3 gap-1"
      disallowEmptySelection
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
            "flex min-w-0 cursor-default flex-col gap-1 rounded-control p-1 text-caption text-fg-secondary transition-colors data-[hovered]:bg-fill data-[selected]:text-fg",
            isIsland
              ? "data-[selected]:bg-fill-strong"
              : "data-[selected]:bg-selected",
            focusStyles,
            elementFocusVisible,
          )}
          id={item.id}
          key={item.id}
        >
          <span
            aria-hidden
            className="block aspect-video w-full overflow-hidden rounded-sm bg-fill"
          >
            {item.picture}
          </span>
          <span className="truncate text-center">{item.label}</span>
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
