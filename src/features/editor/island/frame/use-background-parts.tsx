// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, useState } from "react";

import {
  type Background,
  sameBackground,
} from "../../tools/frame/background/background";
import { BackgroundTiles } from "../../tools/frame/background/background-tiles";
import { useBackgroundChoice } from "../../tools/frame/background/use-background-choice";
import { IslandPickerColumn } from "../island-picker-column";

import { FrameBackgroundEditor } from "./frame-background-editor";
import { FrameLibraryPictures } from "./frame-library-pictures";

import type { FrameIslandProps } from "./frame-island";

export type BackgroundPartsProps = Pick<
  FrameIslandProps,
  | "background"
  | "onBackgroundChange"
  | "onPickImage"
  | "onRemovePreset"
  | "onSavePreset"
  | "presets"
  | "savedPresets"
> & {
  /** Width over height of the picture the background sits under, which the
   * library's pictures are shown at. */
  aspect: number;
};

/**
 * The parts an island chooses a background with, for the Frame island and a
 * gap's: the tiles, the colours' line under them for a mesh or a colour of
 * your own, and the library's pictures in a column beside the section
 * (`IslandPickerColumn`), which the Library tile opens. One chosen there is
 * worn, and kept as a tile as a picture of your own is, once. The tiles take
 * the section's width and add none of their own, so the lines above set the
 * island's width and the tiles wrap to it.
 */
export function useBackgroundParts(props: BackgroundPartsProps) {
  const columnId = useId();
  const [isLibraryOpen, setIsLibraryOpen] = useState(false);
  const choice = useBackgroundChoice({
    onChange: props.onBackgroundChange,
    presets: props.presets,
    savedPresets: props.savedPresets,
    value: props.background,
  });
  const wearPicture = (path: string, name: string | null) => {
    const picked: Background = { kind: "image", path };
    const kept = props.savedPresets.find((preset) =>
      sameBackground(preset.background, picked),
    );
    if (kept) {
      choice.choosePreset(kept, true);
      return;
    }
    props.onSavePreset(picked, name ?? undefined);
    choice.choose(picked);
  };
  // A tile chosen outside the library puts the library's pictures away; one
  // chosen from them leaves them open, so another can be tried.
  const tileChoice = {
    ...choice,
    choose: (background: Background) => {
      setIsLibraryOpen(false);
      choice.choose(background);
    },
    choosePreset: (...chosen: Parameters<typeof choice.choosePreset>) => {
      setIsLibraryOpen(false);
      choice.choosePreset(...chosen);
    },
    openCustom: () => {
      setIsLibraryOpen(false);
      choice.openCustom();
    },
  };
  return {
    colors: choice.showEditor ? (
      <div key="colors">
        <FrameBackgroundEditor
          onChange={choice.changeFromEditor}
          value={props.background}
        />
      </div>
    ) : null,
    column: (
      <IslandPickerColumn
        content={
          isLibraryOpen ? (
            <FrameLibraryPictures
              aspect={props.aspect}
              onChoose={(picture) => {
                wearPicture(picture.file, picture.title);
              }}
              selectedPath={
                props.background.kind === "image" ? props.background.path : null
              }
            />
          ) : null
        }
        contentKey="library"
        id={columnId}
      />
    ),
    // Their box reaches the small inset past them on every side, as the
    // colour grid's does, so a chosen tile's outline at the edge stays inside
    // it and is not cut off while the part moves (`IslandParts`).
    tiles: (
      <div
        className="-m-inset-sm w-0 min-w-[calc(100%+var(--spacing-inset-sm)*2)] p-inset-sm"
        key="tiles"
      >
        <BackgroundTiles
          choice={tileChoice}
          library={{
            controls: columnId,
            isOpen: isLibraryOpen,
            onToggle: () => {
              setIsLibraryOpen((open) => !open);
            },
          }}
          onPickImage={props.onPickImage}
          onRemovePreset={props.onRemovePreset}
          onSavePreset={props.onSavePreset}
          presets={props.presets}
          savedPresets={props.savedPresets}
        />
      </div>
    ),
  };
}
