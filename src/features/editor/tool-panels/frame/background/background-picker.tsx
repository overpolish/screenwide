// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Background, BackgroundPreset } from "./background";
import { BackgroundEditor } from "./background-editor";
import { BackgroundTiles } from "./background-tiles";
import { useBackgroundChoice } from "./use-background-choice";

/**
 * The backgrounds on offer, and the one the canvas is wearing: the grid of
 * tiles, and the hand-built editor under it only while it is wanted, so the
 * common choice stays a single press.
 */
export function BackgroundPicker({
  isDisabled,
  onChange,
  onPickImage,
  onRemovePreset,
  onSavePreset,
  presets,
  savedPresets,
  value,
}: {
  onChange: (background: Background) => void;
  onPickImage: () => Promise<string | null>;
  onRemovePreset: (presetId: string) => void;
  onSavePreset: (background: Background) => void;
  presets: BackgroundPreset[];
  savedPresets: BackgroundPreset[];
  value: Background;
  isDisabled?: boolean;
}) {
  const choice = useBackgroundChoice({
    onChange,
    presets,
    savedPresets,
    value,
  });
  return (
    <div className="flex flex-col gap-separate">
      <BackgroundTiles
        choice={choice}
        isDisabled={isDisabled}
        onPickImage={onPickImage}
        onRemovePreset={onRemovePreset}
        onSavePreset={onSavePreset}
        presets={presets}
        savedPresets={savedPresets}
      />
      {choice.showEditor ? (
        <BackgroundEditor
          isDisabled={isDisabled}
          onChange={choice.changeFromEditor}
          onSavePreset={onSavePreset}
          value={value}
        />
      ) : null}
    </div>
  );
}
