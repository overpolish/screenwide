// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";
import { ToggleButtonGroup } from "react-aria-components";

import {
  useColorPanel,
  usesSystemColorPanel,
} from "../../base/input-fields/use-color-panel";
import { BackgroundTile } from "../background-picker/background-tile";

import {
  ANNOTATION_SWATCHES,
  isAnnotationSwatch,
  sameAnnotationColor,
} from "./palette";

/**
 * The colours an annotation is offered: the palette, then the colours of your
 * own, then the Custom tile that opens the system Colours panel.
 *
 * The grid is the background picker's one row down, so the two rows of swatches
 * behave alike wherever they are met. `savedColors` and the Custom tile are for
 * a caller that has somewhere to keep a colour the palette does not hold; the
 * live overlay's toolbar offers the palette alone, because a setting has no
 * colour list behind it.
 */
export function AnnotationColorGrid({
  isDisabled,
  onChange,
  onContextMenuColor,
  onSettled,
  savedColors = [],
  value,
}: {
  onChange: (color: string) => void;
  /** `#rrggbb` or `#rrggbbaa`, as the document stores it. */
  value: string;
  isDisabled?: boolean;
  /** A press of the secondary button on a colour of your own, which is how
   * one is forgotten. Omitted leaves the saved swatches without a menu. */
  onContextMenuColor?: (color: string, anchor: DOMRect) => void;
  /** The colour the system Colours panel closed on, which is the one moment a
   * caller keeping colours should keep one. Given, the Custom tile is
   * offered; omitted, the grid is the palette and whatever is saved. */
  onSettled?: (color: string) => void;
  /** Colours of your own, drawn after the palette. */
  savedColors?: string[];
}) {
  const systemPanel = usesSystemColorPanel();
  // The panel reports every step of a drag; only what it closed on is
  // settled, so a sweep through the spectrum does not leave a dozen swatches
  // behind.
  const colorPanel = useColorPanel({
    onChange,
    onClose: (settled) => {
      if (settled) onSettled?.(settled);
    },
  });
  const saved = savedColors.filter((kept) => !isAnnotationSwatch(kept));
  const chosen =
    [...ANNOTATION_SWATCHES.map((swatch) => swatch.color), ...saved].find(
      (offered) => sameAnnotationColor(offered, value),
    ) ?? "custom";

  return (
    <ToggleButtonGroup
      aria-label="Colour"
      className="flex flex-wrap justify-center gap-control"
      disallowEmptySelection
      isDisabled={isDisabled}
      selectedKeys={new Set([chosen])}
      selectionMode="single"
    >
      {ANNOTATION_SWATCHES.map((swatch) => (
        <BackgroundTile
          ariaLabel={swatch.name}
          background={{ color: swatch.color, kind: "solid" }}
          id={swatch.color}
          isDisabled={isDisabled}
          isSelected={chosen === swatch.color}
          key={swatch.color}
          onPress={() => {
            onChange(swatch.color);
          }}
        />
      ))}
      {saved.map((kept) => (
        <BackgroundTile
          ariaLabel={kept}
          background={{ color: kept, kind: "solid" }}
          id={kept}
          isDisabled={isDisabled}
          isSelected={chosen === kept}
          key={kept}
          onContextMenu={(event) => {
            event.preventDefault();
            if (isDisabled || !onContextMenuColor) return;
            onContextMenuColor(
              kept,
              event.currentTarget.getBoundingClientRect(),
            );
          }}
          onPress={() => {
            onChange(kept);
          }}
        />
      ))}
      {/* The Custom tile is a button that opens the system Colours panel; a
          colour chosen there becomes a tile of its own, so this one is the
          glyph and is chosen only while none of the tiles matches. Off the Mac
          there is no panel to open, so the platform's own colour input sits
          over the tile at zero opacity, exactly as the colour well does, and
          settles on blur rather than on every step of a drag. */}
      {onSettled ? (
        <span className="relative inline-flex">
          <BackgroundTile
            ariaLabel="Custom colour"
            id="custom"
            isDisabled={isDisabled}
            isSelected={chosen === "custom"}
            onPress={() => {
              if (systemPanel) colorPanel.open(value);
            }}
          >
            <Plus aria-hidden="true" />
          </BackgroundTile>
          {systemPanel ? null : (
            <input
              aria-label="Custom colour"
              className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
              disabled={isDisabled}
              onBlur={() => {
                onSettled(value);
              }}
              onChange={(event) => {
                onChange(event.currentTarget.value.toLowerCase());
              }}
              type="color"
              value={value}
            />
          )}
        </span>
      ) : null}
    </ToggleButtonGroup>
  );
}
