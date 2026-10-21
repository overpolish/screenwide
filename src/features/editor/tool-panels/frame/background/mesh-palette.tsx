// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, LockOpen } from "lucide-react";
import { ToggleButton } from "react-aria-components";

import { ColorButton } from "../../../../../composites/color-button/color-button";
import { ControlRow } from "../../../../../composites/control-row/control-row";
import { t } from "../../../../../i18n/i18n";
import {
  generatePaletteFromLocked,
  PaletteMode,
} from "../../../../../lib/palette-generator";
import { cn, focusStyles } from "../../../../../lib/styling";
import { Button } from "../../../../../primitives/button/button";
import { useColorPicker } from "../../../../../primitives/input-fields/use-color-picker";

// Named for how they look rather than the generator's terms.
const moods = (): { label: string; mode: PaletteMode }[] => [
  { label: t("background-palette-bright"), mode: "bright" },
  { label: t("background-palette-muted"), mode: "dull" },
  { label: t("background-palette-shades"), mode: "shades" },
  { label: t("background-palette-mixed"), mode: "chaotic" },
];

// Stable keys: a mesh has three to five colours, never reordered.
const SLOTS = ["one", "two", "three", "four", "five"];

// Painted as any on/off button: raised neutral while off, primary while on,
// so the two differ in both appearances. The raised material is a tint meant
// for the window's surface, so the open lock lays it over that surface
// rather than over the swatch's colour. Disabled, a lock that is on stays
// visible, so the state is not lost.
const lockStyles = ({ isSelected }: { isSelected: boolean }) =>
  cn(
    focusStyles,
    "group/lock absolute -end-1 -bottom-1 flex h-part-height w-part-height items-center justify-center rounded-full transition-opacity",
    isSelected
      ? "bg-primary text-on-primary primary-raised data-[disabled]:opacity-40 data-[hovered]:bg-primary-hover data-[pressed]:bg-primary-pressed data-[pressed]:primary-raised-pressed"
      : "bg-surface text-fg opacity-0 group-hover/swatch:opacity-100 data-[disabled]:opacity-0 data-[focus-visible]:opacity-100",
  );

const raisedLayer =
  "absolute inset-0 rounded-full control-raised group-data-[hovered]/lock:control-raised-hover group-data-[pressed]/lock:control-raised-pressed";

// A secondary click on the swatch toggles its lock too.
function PaletteColor({
  index,
  isDisabled,
  isLocked,
  onChange,
  onLockedChange,
  value,
}: {
  index: number;
  isLocked: boolean;
  onChange: (value: string) => void;
  onLockedChange: (isLocked: boolean) => void;
  value: string;
  isDisabled?: boolean;
}) {
  const picker = useColorPicker(onChange);
  return (
    <span
      className="group/swatch relative inline-flex"
      onContextMenu={(event) => {
        event.preventDefault();
        if (!isDisabled) onLockedChange(!isLocked);
      }}
    >
      <ColorButton
        aria-label={t("background-palette-color", { index: index + 1 })}
        isDisabled={isDisabled}
        onPress={(event) => {
          picker.open(value, event.target);
        }}
        value={value}
      />
      <ToggleButton
        aria-label={t("background-palette-lock", { index: index + 1 })}
        className={lockStyles}
        isDisabled={isDisabled}
        isSelected={isLocked}
        onChange={onLockedChange}
      >
        {isLocked ? (
          <Lock aria-hidden="true" className="size-icon-xs" />
        ) : (
          <>
            <span aria-hidden="true" className={raisedLayer} />
            <LockOpen aria-hidden="true" className="relative size-icon-xs" />
          </>
        )}
      </ToggleButton>
    </span>
  );
}

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
              <PaletteColor
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
