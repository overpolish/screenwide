// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, LockOpen } from "lucide-react";
import { ToggleButton } from "react-aria-components";

import { t } from "../../../../../i18n/i18n";
import { cn } from "../../../../../lib/styling";
import { buttonVariants } from "../../../../../primitives/button/button-variants";

import { ColorPickerSwatch } from "./color-picker-swatch";

import type { SwatchSize } from "../../../../../primitives/swatch/swatch-styles";

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
      {/* The lock is a button as any on/off one is, raised neutral while off
       * and primary while on, so the two differ in both appearances; its
       * glyph says which too. The raised material is a tint meant for the
       * window's surface, so the open lock stands on that surface rather
       * than on the swatch's colour. It shows under the pointer or keyboard
       * focus. Disabled, a lock that is on stays visible, faded as a
       * disabled button is, so the state is not lost; one that is off hides.
       * On a small swatch the badge is an icon's size rather than a part's,
       * so it covers a corner and not the colour. */}
      <span
        className={cn(
          "absolute -end-1 -bottom-1 flex rounded-full transition-opacity",
          size === "md" ? "size-part-height" : "size-icon-sm",
          !isLocked && "bg-surface opacity-0",
          !isLocked &&
            !isDisabled &&
            "group-hover/swatch:opacity-100 has-data-[focus-visible]:opacity-100",
        )}
      >
        <ToggleButton
          aria-label={t("background-palette-lock", { index: index + 1 })}
          className={({ isSelected }) =>
            cn(
              buttonVariants(
                isSelected ? { color: "primary", isDisabled } : { isDisabled },
              ),
              "size-full rounded-full px-0 [&_svg.lucide]:size-icon-xs",
            )
          }
          isDisabled={isDisabled}
          isSelected={isLocked}
          onChange={onLockedChange}
        >
          {isLocked ? (
            <Lock aria-hidden="true" />
          ) : (
            <LockOpen aria-hidden="true" />
          )}
        </ToggleButton>
      </span>
    </ColorPickerSwatch>
  );
}
