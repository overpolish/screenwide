// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToggleButton, ToggleButtonGroup } from "react-aria-components";

import { type Placement } from "../../../../bindings/Placement";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";

// Where an overlay sits on the picture, as a small picture of nine places:
// each a dot, the chosen one filled. The picture's own sides, so it is laid
// out left to right whatever the language.

const PLACES: { id: Placement; label: string }[] = [
  { id: "top-left", label: "Top left" },
  { id: "top", label: "Top" },
  { id: "top-right", label: "Top right" },
  { id: "left", label: "Left" },
  { id: "center", label: "Centre" },
  { id: "right", label: "Right" },
  { id: "bottom-left", label: "Bottom left" },
  { id: "bottom", label: "Bottom" },
  { id: "bottom-right", label: "Bottom right" },
];

export function PlacementChoice({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (place: Placement) => void;
  value: Placement;
}) {
  return (
    <ToggleButtonGroup
      aria-label={label}
      className="grid aspect-[16/10] w-40 grid-cols-3 grid-rows-3 rounded-[10px] bg-fill p-1"
      dir="ltr"
      disallowEmptySelection
      onSelectionChange={(keys) => {
        for (const key of keys) onChange(key as Placement);
      }}
      selectedKeys={[value]}
      selectionMode="single"
    >
      {PLACES.map(({ id, label: name }) => (
        <ToggleButton
          aria-label={name}
          className={cn(
            "group flex cursor-default items-center justify-center rounded-[6px] hover:bg-fill",
            focusStyles,
            elementFocusVisible,
          )}
          id={id}
          key={id}
        >
          <span className="size-2 rounded-full bg-fg-tertiary transition-[background-color,scale] group-data-[selected]:scale-150 group-data-[selected]:bg-primary" />
        </ToggleButton>
      ))}
    </ToggleButtonGroup>
  );
}
