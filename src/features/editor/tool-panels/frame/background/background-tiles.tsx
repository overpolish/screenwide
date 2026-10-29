// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Image as ImageGlyph, Plus, Trash2 } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { ToggleButtonGroup } from "react-aria-components";

import { TileGlyph } from "../../../../../composites/color-grid/color-grid";
import { useTileMotion } from "../../../../../composites/color-grid/tile-motion";
import { useArmedRemoval } from "../../../../../composites/color-grid/use-armed-removal";
import { t } from "../../../../../i18n/i18n";
import { IconSwap } from "../../../../../primitives/icon-swap/icon-swap";
import { Swatch } from "../../../../../primitives/swatch/swatch";

import { Background, BackgroundPreset } from "./background";
import { BackgroundSwatch } from "./background-swatch";

import type { BackgroundChoice } from "./use-background-choice";

/** The colour a glyph over a background's tile is read against: a solid's
 * own, a mesh's first, or none for a picture, which takes the window's. */
const tileInk = (background: Background) => {
  if (background.kind === "solid") return background.color;
  if (background.kind === "mesh") return background.colors[0] ?? null;
  return null;
};

/**
 * The backgrounds on offer as a grid of swatches, because a background is
 * chosen by the look of it: the ones that ship with the app, then the ones
 * that were kept, then one built by hand (`choice`), and last a picture of
 * your own, which stands for an action rather than a colour.
 *
 * Kept tiles are handled as the colour grid's kept colours are. Custom
 * shows the background of your own (`choice.custom`), and a right click
 * there keeps it. Forgetting a kept tile takes two right clicks, because it
 * cannot be undone. Tiles come and go as `useTileMotion` moves them.
 */
export function BackgroundTiles({
  choice,
  isDisabled,
  onPickImage,
  onRemovePreset,
  onSavePreset,
  presets,
  savedPresets,
}: {
  choice: BackgroundChoice;
  onPickImage: () => Promise<string | null>;
  onRemovePreset: (presetId: string) => void;
  onSavePreset: (background: Background) => void;
  presets: BackgroundPreset[];
  savedPresets: BackgroundPreset[];
  isDisabled?: boolean;
}) {
  const removal = useArmedRemoval(onRemovePreset);
  const armed = isDisabled ? null : removal.armed;
  const { away, shown, slide } = useTileMotion();
  const { custom } = choice;

  return (
    <ToggleButtonGroup
      aria-label={t("background-label")}
      // Centred rows: the tiles keep their size and gap, and whatever the
      // row cannot fill is split evenly between the two edges.
      className="flex flex-wrap justify-center gap-inline"
      disallowEmptySelection
      isDisabled={isDisabled}
      selectedKeys={new Set([choice.selectedId])}
      selectionMode="single"
    >
      {presets.map((preset) => (
        <motion.span className="flex" key={preset.id} {...slide}>
          <BackgroundSwatch
            aria-label={preset.name}
            background={preset.background}
            id={preset.id}
            isDisabled={isDisabled}
            isSelected={choice.selectedId === preset.id}
            onPress={() => {
              choice.choosePreset(preset, false);
            }}
          />
        </motion.span>
      ))}
      <AnimatePresence initial={false} mode="popLayout">
        {savedPresets.map((preset) => (
          <motion.span
            animate={shown}
            className="flex"
            exit={away}
            initial={away}
            key={preset.id}
            {...slide}
          >
            <BackgroundSwatch
              aria-label={
                armed === preset.id
                  ? t("background-saved-armed", { name: preset.name })
                  : preset.name
              }
              background={preset.background}
              className="relative"
              id={preset.id}
              isDisabled={isDisabled}
              isSelected={choice.selectedId === preset.id}
              onContextMenu={(event) => {
                event.preventDefault();
                if (!isDisabled) removal.secondary(preset.id);
              }}
              onHoverEnd={removal.disarm}
              {...(isDisabled ? {} : removal.keyboard(preset.id))}
              onPress={() => {
                removal.disarm();
                choice.choosePreset(preset, true);
              }}
            >
              <IconSwap swapKey={armed === preset.id ? "armed" : "rest"}>
                {armed === preset.id ? (
                  <TileGlyph color={tileInk(preset.background)}>
                    <Trash2 aria-hidden="true" />
                  </TileGlyph>
                ) : null}
              </IconSwap>
            </BackgroundSwatch>
          </motion.span>
        ))}
      </AnimatePresence>
      <motion.span className="flex" {...slide}>
        <BackgroundSwatch
          aria-label={t("background-custom")}
          background={custom}
          id="custom"
          isDisabled={isDisabled}
          isSelected={choice.selectedId === "custom"}
          onContextMenu={(event) => {
            event.preventDefault();
            if (!isDisabled && choice.canKeep) choice.keep(onSavePreset);
          }}
          onPress={choice.openCustom}
        >
          <TileGlyph color={custom ? tileInk(custom) : null}>
            <Plus aria-hidden="true" />
          </TileGlyph>
        </BackgroundSwatch>
      </motion.span>
      {/* The Image tile is a button that picks a new picture; a picked one
          is kept at once with a tile of its own, so this one is always the
          glyph and never the chosen background. */}
      <motion.span className="flex" {...slide}>
        <Swatch
          aria-label={t("background-image")}
          id="image"
          isDisabled={isDisabled}
          isSelected={false}
          onPress={() => {
            void onPickImage().then((path) => {
              if (!path) return;
              const picked: Background = { kind: "image", path };
              onSavePreset(picked);
              choice.choose(picked);
            });
          }}
          size="sm"
        >
          <ImageGlyph aria-hidden="true" />
        </Swatch>
      </motion.span>
    </ToggleButtonGroup>
  );
}
