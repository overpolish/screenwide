// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Image as ImageGlyph, Plus } from "lucide-react";
import { useState } from "react";
import { ToggleButtonGroup } from "react-aria-components";

import { Background, BackgroundPreset, sameBackground } from "./background";
import { BackgroundEditor } from "./background-editor";
import { randomMeshForGenerator } from "./background-random";
import { BackgroundTile } from "./background-tile";

/**
 * The backgrounds on offer, and the one the canvas is wearing.
 *
 * A grid of swatches first, because a background is chosen by the look of it:
 * the ones that ship with the app, then the ones that were saved, then the
 * two that stand for an action rather than a colour - a picture of your own,
 * and one built by hand. The hand-built editor appears under the grid only
 * while it is wanted, so the common choice stays a single press.
 */
export function BackgroundPicker({
  isDisabled,
  onChange,
  onPickImage,
  onPresetMenu,
  onSavePreset,
  presets,
  savedPresets,
  value,
}: {
  onChange: (background: Background) => void;
  onPickImage: () => Promise<string | null>;
  onSavePreset: (background: Background) => void;
  presets: BackgroundPreset[];
  savedPresets: BackgroundPreset[];
  value: Background;
  isDisabled?: boolean;
  /** A right click on a saved tile. The feature that owns the window opens
   * the app's own menu against these bounds: this component is shared, and
   * the pop-up panel belongs to a feature. */
  onPresetMenu?: (presetId: string, anchor: DOMRect) => void;
}) {
  const [isCustomOpen, setIsCustomOpen] = useState(false);
  // A saved preset is matched by everything it holds; a built-in mesh tile
  // stands for its generator, so it stays chosen while the colours under it
  // are changed. Only a solid that matches nothing is a custom colour.
  const matched =
    savedPresets.find((preset) => sameBackground(preset.background, value)) ??
    presets.find((preset) =>
      value.kind === "mesh" && preset.background.kind === "mesh"
        ? preset.background.generator === value.generator
        : sameBackground(preset.background, value),
    );
  const isImage = value.kind === "image";
  // The editor's visibility is a choice, not a comparison: it is open because
  // the Custom tile was pressed, and closes only when another tile is pressed.
  // Nothing the editor itself hands back can shut it or select a preset,
  // even when the edited value happens to match one.
  // A background that matches nothing on offer was built by hand,
  // so the editor is already open on it; the Custom tile only opens it early.
  // A mesh always shows its colours: the generator is chosen in the grid,
  // and what it paints with is set here.
  const showEditor =
    isCustomOpen || value.kind === "mesh" || (!matched && !isImage);
  // An image always has a tile of its own once picked, so no chosen value
  // ever lands on the Image button; anything unmatched is a custom edit.
  const selectedId = isCustomOpen ? "custom" : (matched?.id ?? "custom");

  /** A tile's own choice: it closes the editor, which only a tile can do. */
  const choose = (background: Background) => {
    setIsCustomOpen(false);
    onChange(background);
  };

  /** A change the editor made. It holds the editor open rather than leaving
   * it to the comparison: a mesh turned back into a flat colour can land on
   * one of the built-in tones, and the controls that made it must not vanish
   * under the hand that was using them. */
  const changeFromEditor = (background: Background) => {
    setIsCustomOpen(true);
    onChange(background);
  };

  // A built-in mesh is a generator under its colours, not a picture: pressing
  // its tile draws that generator again under a seed made on the spot. A
  // saved mesh is a background somebody kept, so it comes back exactly as it
  // was kept.
  // Pressing the tile of the generator already in use rolls the picture
  // again under the colours that were chosen for it, rather than under the
  // generator's own.
  const chosenFrom = (preset: BackgroundPreset, isSaved: boolean) => {
    if (isSaved || preset.background.kind !== "mesh") return preset.background;
    if (
      value.kind !== "mesh" ||
      value.generator !== preset.background.generator
    )
      return randomMeshForGenerator(preset.background.generator);
    const rolled = randomMeshForGenerator(value.generator, value.colors.length);
    return {
      ...rolled,
      colors: value.colors,
      lockedColors: value.lockedColors,
    };
  };

  const tile = (preset: BackgroundPreset, isSaved: boolean) => (
    <BackgroundTile
      ariaLabel={preset.name}
      background={preset.background}
      id={preset.id}
      isDisabled={isDisabled}
      isSelected={selectedId === preset.id}
      key={preset.id}
      onContextMenu={
        isSaved && onPresetMenu
          ? (event) => {
              event.preventDefault();
              if (isDisabled) return;
              onPresetMenu(
                preset.id,
                event.currentTarget.getBoundingClientRect(),
              );
            }
          : undefined
      }
      onPress={() => {
        choose(chosenFrom(preset, isSaved));
      }}
      thumbnailPath={preset.thumbnailPath}
    />
  );

  return (
    <div className="flex flex-col gap-control-inset">
      <ToggleButtonGroup
        aria-label="Background"
        className="flex flex-wrap gap-control"
        disallowEmptySelection
        isDisabled={isDisabled}
        selectedKeys={new Set([selectedId])}
        selectionMode="single"
      >
        {presets.map((preset) => tile(preset, false))}
        {savedPresets.map((preset) => tile(preset, true))}
        {/* The Image tile is a button that picks a new picture; a picked one
            becomes a preset with a tile of its own, so this one is always the
            glyph and never the chosen background. */}
        <BackgroundTile
          ariaLabel="Image"
          id="image"
          isDisabled={isDisabled}
          isSelected={false}
          onPress={() => {
            // A picked image is kept as a preset at once, so it can be found
            // again next time, and applied.
            void onPickImage().then((path) => {
              if (!path) return;
              const picked: Background = { kind: "image", path };
              onSavePreset(picked);
              choose(picked);
            });
          }}
        >
          <ImageGlyph aria-hidden="true" />
        </BackgroundTile>
        <BackgroundTile
          ariaLabel="Custom"
          id="custom"
          isDisabled={isDisabled}
          isSelected={selectedId === "custom"}
          onPress={() => {
            setIsCustomOpen(true);
          }}
        >
          <Plus aria-hidden="true" />
        </BackgroundTile>
      </ToggleButtonGroup>

      {showEditor ? (
        <BackgroundEditor
          isDisabled={isDisabled}
          onChange={changeFromEditor}
          onSavePreset={onSavePreset}
          value={value}
        />
      ) : null}
    </div>
  );
}
