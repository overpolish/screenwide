// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ReactNode } from "react";

import { Plus, Trash2 } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { ToggleButtonGroup } from "react-aria-components";

import { t } from "../../i18n/i18n";
import {
  motionDurationCss,
  motionDurations,
  motionEasings,
} from "../../lib/motion";
import { cn } from "../../lib/styling";
import { IconSwap } from "../../primitives/icon-swap/icon-swap";
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

// A glyph over a swatch, in the appearance that reads on `color`; with no
// colour under it, the window's own. Its ink eases with the swatch's paint,
// so the Custom tile's glyph turns as the tile empties when its colour is
// kept.
function TileGlyph({
  children,
  color,
}: {
  children: ReactNode;
  color: string | null;
}) {
  const transitionDuration = useReducedMotion()
    ? "0s"
    : motionDurationCss("travel");
  return (
    <span
      className={cn(
        "flex text-fg transition-colors ease-out",
        color && (isLight(color) ? "appearance-light" : "appearance-dark"),
      )}
      style={{ transitionDuration }}
    >
      {children}
    </span>
  );
}

// The grid is the background picker's one row down, so the two rows of
// swatches behave alike wherever they are met. A colour is kept only when
// asked for, by a right click on the Custom tile showing it, the way the
// Colours panel's own swatches are filled by hand rather than by whatever was
// last tried. Forgetting a kept colour takes two right clicks because it
// cannot be undone.
//
// Tiles kept or forgotten come and go the way the recording island's and
// Glide's glyphs do, out of a blur from a smaller size, while the tiles after
// them slide to their new places; the outgoing tile is lifted from the flow
// so the rest move at once.
//
// Swatches are the small size everywhere: a grid is a row of many, where the
// control's height would crowd.
export function ColorGrid({
  columns,
  isDisabled,
  offersCustom,
  onChange,
  onRemoveColor,
  onSaveColor,
  savedColors = [],
  value,
}: {
  onChange: (color: string) => void;
  /** `#rrggbb` or `#rrggbbaa`. */
  value: string;
  columns?: number;
  isDisabled?: boolean;
  // The live overlay's toolbar leaves Custom out: it closes on every pick,
  // so it could not stay open through a drag in the Colours panel.
  offersCustom?: boolean;
  // Omitted, the kept tiles cannot be armed.
  onRemoveColor?: (color: string) => void;
  onSaveColor?: (color: string) => void;
  savedColors?: string[];
}) {
  const systemPanel = usesSystemColorPanel();
  const colorPanel = useColorPanel({ onChange });
  const removal = useArmedRemoval((color) => onRemoveColor?.(color));
  const reducedMotion = useReducedMotion() ?? false;
  const armed = isDisabled ? null : removal.armed;
  const saved = savedColors.filter((kept) => !isAnnotationSwatch(kept));
  const chosen =
    [...ANNOTATION_SWATCHES.map((swatch) => swatch.color), ...saved].find(
      (offered) => sameAnnotationColor(offered, value),
    ) ?? "custom";
  const isCustom = chosen === "custom";
  const count =
    ANNOTATION_SWATCHES.length + saved.length + (offersCustom ? 1 : 0);

  const ease = { duration: motionDurations.travel, ease: motionEasings.out };
  const slide = {
    layout: "position",
    transition: { ...ease, layout: reducedMotion ? { duration: 0 } : ease },
  } as const;
  const away = reducedMotion
    ? { opacity: 0 }
    : { filter: "blur(2px)", opacity: 0, scale: 0.6 };

  return (
    <ToggleButtonGroup
      aria-label={t("annotation-color")}
      className={cn(
        "relative gap-inline",
        columns === undefined ? "flex flex-wrap justify-center" : "grid",
      )}
      disallowEmptySelection
      isDisabled={isDisabled}
      selectedKeys={new Set([chosen])}
      selectionMode="single"
      // The rows start at the same edge, so the columns line up.
      style={
        columns === undefined
          ? undefined
          : {
              gridTemplateColumns: `repeat(${String(Math.min(count, columns))}, auto)`,
            }
      }
    >
      {ANNOTATION_SWATCHES.map((swatch) => (
        <motion.span className="flex" key={swatch.color} {...slide}>
          <Swatch
            aria-label={swatch.name()}
            color={swatch.color}
            id={swatch.color}
            isDisabled={isDisabled}
            isSelected={chosen === swatch.color}
            onPress={() => {
              onChange(swatch.color);
            }}
            size="sm"
          />
        </motion.span>
      ))}
      <AnimatePresence initial={false} mode="popLayout">
        {saved.map((kept) => (
          <motion.span
            animate={{ filter: "blur(0px)", opacity: 1, scale: 1 }}
            className="flex"
            exit={away}
            initial={away}
            key={kept}
            {...slide}
          >
            <Swatch
              aria-label={
                armed === kept
                  ? t("annotation-saved-color-armed", { color: kept })
                  : kept
              }
              className="relative"
              color={kept}
              id={kept}
              isDisabled={isDisabled}
              isSelected={chosen === kept}
              onContextMenu={(event) => {
                event.preventDefault();
                if (!isDisabled && onRemoveColor) removal.secondary(kept);
              }}
              onHoverEnd={removal.disarm}
              onPress={() => {
                removal.disarm();
                onChange(kept);
              }}
              size="sm"
            >
              <IconSwap swapKey={armed === kept ? "armed" : "rest"}>
                {armed === kept ? (
                  <TileGlyph color={kept}>
                    <Trash2 aria-hidden="true" />
                  </TileGlyph>
                ) : null}
              </IconSwap>
            </Swatch>
          </motion.span>
        ))}
      </AnimatePresence>
      {/* The Custom tile is a button that opens the system Colours panel.
          While the colour matches no tile it shows that colour behind its
          glyph, and a right click keeps it as a tile of its own. Off the Mac
          there is no panel to open, so the platform's own colour input sits
          over the tile at zero opacity, exactly as the colour well does; the
          right click is heard on the wrapper so it reaches past that input. */}
      {offersCustom ? (
        <motion.span
          className="relative inline-flex"
          onContextMenu={(event) => {
            event.preventDefault();
            if (!isDisabled && isCustom) onSaveColor?.(value);
          }}
          {...slide}
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
            size="sm"
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
        </motion.span>
      ) : null}
    </ToggleButtonGroup>
  );
}
