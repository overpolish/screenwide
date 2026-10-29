// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../../composites/control-row/control-row";
import { t } from "../../../../../i18n/i18n";
import {
  generatePaletteFromLocked,
  PaletteMode,
} from "../../../../../lib/palette-generator";
import { Button } from "../../../../../primitives/button/button";

import { MeshPaletteColor } from "./mesh-palette-color";

// Named for how they look rather than the generator's terms.
const moods = (): { label: string; mode: PaletteMode }[] => [
  { label: t("background-palette-bright"), mode: "bright" },
  { label: t("background-palette-muted"), mode: "dull" },
  { label: t("background-palette-shades"), mode: "shades" },
  { label: t("background-palette-mixed"), mode: "chaotic" },
];

// Stable keys: a mesh has three to five colours, never reordered.
const SLOTS = ["one", "two", "three", "four", "five"];

// Each colour is picked by hand or locked; a mood button remakes the rest.
export function MeshPalette({
  colors,
  isDisabled,
  locked,
  onChange,
  onLockedChange,
}: {
  colors: string[];
  locked: boolean[];
  onChange: (colors: string[]) => void;
  onLockedChange: (locked: boolean[]) => void;
  isDisabled?: boolean;
}) {
  return (
    <div className="flex flex-col gap-inline">
      <ControlRow title={t("background-colors")}>
        {(controlProps) => (
          <div {...controlProps} className="flex gap-inline" role="group">
            {colors.slice(0, SLOTS.length).map((color, index) => (
              <MeshPaletteColor
                index={index}
                isDisabled={isDisabled}
                isLocked={locked[index] ?? false}
                key={SLOTS[index]}
                onChange={(value) => {
                  onChange(
                    colors.map((current, at) =>
                      at === index ? value : current,
                    ),
                  );
                }}
                onLockedChange={(isLocked) => {
                  onLockedChange(
                    colors.map((_, at) =>
                      at === index ? isLocked : (locked[at] ?? false),
                    ),
                  );
                }}
                value={color}
              />
            ))}
          </div>
        )}
      </ControlRow>
      <div
        aria-label={t("background-palette-new")}
        className="grid grid-cols-4 gap-inline"
        role="group"
      >
        {moods().map(({ label, mode }) => (
          <Button
            isDisabled={isDisabled}
            key={mode}
            onPress={() => {
              onChange(generatePaletteFromLocked({ colors, locked, mode }));
            }}
          >
            {label}
          </Button>
        ))}
      </div>
    </div>
  );
}
