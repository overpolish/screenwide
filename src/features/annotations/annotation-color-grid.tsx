// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { Plus, Trash2 } from "lucide-react";
import { ToggleButtonGroup } from "react-aria-components";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import {
  useColorPanel,
  usesSystemColorPanel,
} from "../../primitives/input-fields/use-color-panel";
import { Swatch } from "../../primitives/swatch/swatch";

import {
  ANNOTATION_SWATCHES,
  isAnnotationSwatch,
  isLight,
  sameAnnotationColor,
} from "./palette";
import { useArmedRemoval } from "./use-armed-removal";

/** A glyph over a swatch, in the appearance that reads on `color`; with no
 * colour under it, the window's own. */
function TileGlyph({
  children,
  color,
}: {
  children: ReactNode;
  color: string | null;
}) {
  return (
    <span
      className={cn(
        "flex text-fg",
        color && (isLight(color) ? "appearance-light" : "appearance-dark"),
      )}
    >
      {children}
    </span>
  );
}

/**
 * The colours an annotation is offered: the palette, then the colours of your
 * own, then the Custom tile that opens the system Colours panel.
 *
 * The grid is the background picker's one row down, so the two rows of swatches
 * behave alike wherever they are met. A colour of your own is kept only when
 * asked for, by a right click on the Custom tile showing it, the way the
 * Colours panel's own swatches are filled by hand rather than by whatever was
 * last tried. Two right clicks on a kept colour forget it.
 */
export function AnnotationColorGrid({
  isDisabled,
  offersCustom,
  onChange,
  onRemoveColor,
  onSaveColor,
  savedColors = [],
  value,
}: {
  onChange: (color: string) => void;
  /** `#rrggbb` or `#rrggbbaa`, as the document stores it. */
  value: string;
  isDisabled?: boolean;
  /** Offers the Custom tile. The live overlay's toolbar leaves it out: it
   * closes on every pick, so it could not stay open through a drag. */
  offersCustom?: boolean;
  /** Forget a colour of your own. Omitted, the saved tiles cannot be armed. */
  onRemoveColor?: (color: string) => void;
  /** Keep the colour the Custom tile is showing. */
  onSaveColor?: (color: string) => void;
  /** Colours of your own, drawn after the palette. */
  savedColors?: string[];
}) {
  const systemPanel = usesSystemColorPanel();
  const colorPanel = useColorPanel({ onChange });
  const removal = useArmedRemoval((color) => onRemoveColor?.(color));
  const armed = isDisabled ? null : removal.armed;
  const saved = savedColors.filter((kept) => !isAnnotationSwatch(kept));
  const chosen =
    [...ANNOTATION_SWATCHES.map((swatch) => swatch.color), ...saved].find(
      (offered) => sameAnnotationColor(offered, value),
    ) ?? "custom";
  const isCustom = chosen === "custom";

  return (
    <ToggleButtonGroup
      aria-label={t("annotation-color")}
      className="flex flex-wrap justify-center gap-inline"
      disallowEmptySelection
      isDisabled={isDisabled}
      selectedKeys={new Set([chosen])}
      selectionMode="single"
    >
      {ANNOTATION_SWATCHES.map((swatch) => (
        <Swatch
          aria-label={swatch.name()}
          color={swatch.color}
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
        <Swatch
          aria-label={
            armed === kept
              ? t("annotation-saved-color-armed", { color: kept })
              : kept
          }
          color={kept}
          id={kept}
          isDisabled={isDisabled}
          isSelected={chosen === kept}
          key={kept}
          onContextMenu={(event) => {
            event.preventDefault();
            if (!isDisabled && onRemoveColor) removal.secondary(kept);
          }}
          onHoverEnd={removal.disarm}
          onPress={() => {
            removal.disarm();
            onChange(kept);
          }}
        >
          {armed === kept ? (
            <TileGlyph color={kept}>
              <Trash2 aria-hidden="true" />
            </TileGlyph>
          ) : null}
        </Swatch>
      ))}
      {/* The Custom tile is a button that opens the system Colours panel.
          While the colour matches no tile it shows that colour behind its
          glyph, and a right click keeps it as a tile of its own. Off the Mac
          there is no panel to open, so the platform's own colour input sits
          over the tile at zero opacity, exactly as the colour well does; the
          right click is heard on the wrapper so it reaches past that input. */}
      {offersCustom ? (
        <span
          className="relative inline-flex"
          onContextMenu={(event) => {
            event.preventDefault();
            if (!isDisabled && isCustom) onSaveColor?.(value);
          }}
        >
          <Swatch
            aria-label={t("annotation-custom-color")}
            color={isCustom ? value : undefined}
            id="custom"
            isDisabled={isDisabled}
            isSelected={isCustom}
            onPress={() => {
              if (systemPanel) colorPanel.open(value);
            }}
          >
            <TileGlyph color={isCustom ? value : null}>
              <Plus aria-hidden="true" />
            </TileGlyph>
          </Swatch>
          {systemPanel ? null : (
            <input
              aria-label={t("annotation-custom-color")}
              className="absolute inset-0 size-full cursor-default opacity-0 outline-none"
              disabled={isDisabled}
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
