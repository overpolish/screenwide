// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dices } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { type ReactNode } from "react";

import { useTileMotion } from "../../../../composites/color-grid/tile-motion";
import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { TooltipTrigger } from "../../../../primitives/tooltip/tooltip-trigger";
import { ColorPickerSwatch } from "../../tool-panels/frame/background/color-picker-swatch";
import { MeshPaletteColor } from "../../tool-panels/frame/background/mesh-palette-color";
import { useBackgroundEdit } from "../../tool-panels/frame/background/use-background-edit";

import { MeshToggle } from "./mesh-toggle";

import type { Background } from "../../tool-panels/frame/background/background";

/**
 * The background in hand, on the island, as one line: whether it is one
 * colour or a mesh, its colours, and a dice that makes them again. The
 * colours are each picked by pressing them; a mesh's can be locked. The
 * toggle starts the line and the dice ends it whatever is chosen, so the
 * background can turn into the other kind at any time and neither moves
 * as the colours between them change. The colours are keyed by their place,
 * so turning a mesh into one colour keeps the first where it is while the
 * rest go, and turning it back brings them in beside it, as kept tiles come
 * and go in the tiles above (`useTileMotion`). The colours are the tiles'
 * size, so they read as the tile they came from. Keeping what was built is
 * the Custom tile's, by a right click, as in the colour grid.
 */
export function FrameBackgroundEditor({
  onChange,
  value,
}: {
  onChange: (background: Background) => void;
  value: Background;
}) {
  const edit = useBackgroundEdit({ onChange, value });
  const { away, shown, slide } = useTileMotion();
  // A colour is named by its place, shared by a mesh's first and the one
  // colour, so that one stays where it is as the rest come or go.
  const placed = (place: number, swatch: ReactNode) => (
    <motion.span
      animate={shown}
      className="flex"
      exit={away}
      initial={away}
      key={place}
      {...slide}
    >
      {swatch}
    </motion.span>
  );
  const swatches =
    edit.kind === "mesh"
      ? edit.palette.map((color, index) =>
          placed(
            index,
            <MeshPaletteColor
              index={index}
              isLocked={edit.locked[index] ?? false}
              onChange={(next) => {
                edit.setColors(
                  edit.palette.map((each, at) => (at === index ? next : each)),
                );
              }}
              onLockedChange={(isLocked) => {
                edit.setLocked(
                  edit.locked.map((each, at) =>
                    at === index ? isLocked : each,
                  ),
                );
              }}
              size="sm"
              value={color}
            />,
          ),
        )
      : placed(
          0,
          <ColorPickerSwatch
            aria-label={t("background-color")}
            onChange={edit.setSolid}
            size="sm"
            value={edit.solidColor}
          />,
        );
  return (
    <div className="flex h-control-height items-center justify-between gap-separate">
      <div className="flex items-center gap-separate">
        <MeshToggle
          isMesh={edit.kind === "mesh"}
          onChange={(isMesh) => {
            edit.setKind(isMesh ? "mesh" : "solid");
          }}
        />
        <div
          aria-label={t("background-colors")}
          // Positioned, so a colour on its way out is lifted out of the row
          // here while the others slide to their places.
          className="relative flex items-center gap-inline"
          role="group"
        >
          <AnimatePresence initial={false} mode="popLayout">
            {swatches}
          </AnimatePresence>
        </div>
      </div>
      <TooltipTrigger tooltip={{ label: t("editor-island-frame-randomise") }}>
        <IconButton
          aria-label={t("editor-island-frame-randomise")}
          onPress={edit.randomise}
        >
          <Dices />
        </IconButton>
      </TooltipTrigger>
    </div>
  );
}
