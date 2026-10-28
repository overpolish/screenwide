// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { useSkin } from "../skin/skin";

// One of several looks, each a tile with a picture of it over its name,
// three to a row: a scene's layout, a caption template. Built on React
// Aria's toggle button group, which names it a radio group and moves between
// tiles with the arrow keys. A tile without a picture keeps its room, filled,
// so the rows stay even while pictures load. Every picture has one shape,
// wide by default; a picture taller than wide is capped in height and
// centred, narrower, so a tall video's tiles stay a row's height rather than
// a column's. A name too long for its tile wraps rather than being cut
// short, since the name is how a tile is told apart. Where the pictures tell
// the tiles apart on their own, the names can be left to assistive
// technology (`showsLabels`), so every tile is just its picture. The chosen
// tile is a quiet choice made in place, so it takes the selected surface
// with its name unchanged; on the island, the island's stronger fill.
// Whatever holds it scrolls it when there are more rows than room.

/** The tallest a picture grows, in pixels: a little over a wide picture's
 * height in a third of an island. */
const PICTURE_MAX_HEIGHT = 64;

export type TileChoiceItem<Id extends string> = {
  id: Id;
  label: string;
  /** What the tile shows above its name, cropped to the pictures' shape. */
  picture?: ReactNode;
};

export function TileChoice<Id extends string>({
  "aria-label": label,
  items,
  onChange,
  pictureAspect = 16 / 9,
  showsLabels = true,
  value,
}: {
  "aria-label": string;
  items: TileChoiceItem<Id>[];
  onChange: (id: Id) => void;
  value: Id;
  /** The pictures' width over their height, the same for every tile. */
  pictureAspect?: number;
  /** Whether each tile shows its name under its picture; it is the tile's
   * accessible name either way. */
  showsLabels?: boolean;
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
            className="mx-auto block max-w-full overflow-hidden rounded-sm bg-fill"
            style={{
              aspectRatio: pictureAspect,
              width: `min(100%, ${String(PICTURE_MAX_HEIGHT * pictureAspect)}px)`,
            }}
          >
            {item.picture}
          </span>
          {showsLabels ? (
            <span className="text-center text-balance break-words">
              {item.label}
            </span>
          ) : null}
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
