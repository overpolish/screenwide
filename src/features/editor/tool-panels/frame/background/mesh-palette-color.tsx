// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, LockOpen } from "lucide-react";
import { ToggleButton } from "react-aria-components";

import { t } from "../../../../../i18n/i18n";
import { cn, focusStyles } from "../../../../../lib/styling";

import { ColorPickerSwatch } from "./color-picker-swatch";

import type { SwatchSize } from "../../../../../primitives/swatch/swatch-styles";

// Painted as any on/off button: raised neutral while off, primary while on,
// so the two differ in both appearances. The raised material is a tint meant
// for the window's surface, so the open lock lays it over that surface
// rather than over the swatch's colour. Disabled, a lock that is on stays
// visible, so the state is not lost. On a small swatch the badge is an
// icon's size rather than a part's, so it covers a corner and not the
// colour.
const lockStyles =
  (size: SwatchSize) =>
  ({ isSelected }: { isSelected: boolean }) =>
    cn(
      focusStyles,
      "group/lock absolute -end-1 -bottom-1 flex items-center justify-center rounded-full transition-opacity",
      size === "md" ? "size-part-height" : "size-icon-sm",
      isSelected
        ? "bg-primary text-on-primary primary-raised data-[disabled]:opacity-40 data-[hovered]:bg-primary-hover data-[pressed]:bg-primary-pressed data-[pressed]:primary-raised-pressed"
        : "bg-surface text-fg opacity-0 group-hover/swatch:opacity-100 data-[disabled]:opacity-0 data-[focus-visible]:opacity-100",
    );

const raisedLayer =
  "absolute inset-0 rounded-full control-raised group-data-[hovered]/lock:control-raised-hover group-data-[pressed]/lock:control-raised-pressed";

/** One of a mesh's colours: pressed, it opens a colour picker; its corner
 * lock, or a secondary click, keeps it through a new palette. */
export function MeshPaletteColor({
  index,
  isDisabled,
  isLocked,
  onChange,
  onLockedChange,
  size = "md",
  value,
}: {
  /** Place in the palette, from 0. */
  index: number;
  isLocked: boolean;
  onChange: (value: string) => void;
  onLockedChange: (isLocked: boolean) => void;
  value: string;
  isDisabled?: boolean;
  size?: SwatchSize;
}) {
  return (
    <ColorPickerSwatch
      aria-label={t("background-palette-color", { index: index + 1 })}
      className="group/swatch"
      isDisabled={isDisabled}
      onChange={onChange}
      onContextMenu={(event) => {
        event.preventDefault();
        if (!isDisabled) onLockedChange(!isLocked);
      }}
      size={size}
      value={value}
    >
      <ToggleButton
        aria-label={t("background-palette-lock", { index: index + 1 })}
        className={lockStyles(size)}
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
    </ColorPickerSwatch>
  );
}
