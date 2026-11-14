// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useId, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { DimensionsField } from "../../tools/dimensions/dimensions-field";
import {
  type Background,
  sameBackground,
} from "../../tools/frame/background/background";
import { BackgroundTiles } from "../../tools/frame/background/background-tiles";
import { useBackgroundChoice } from "../../tools/frame/background/use-background-choice";
import {
  AnnotationChoiceRow,
  AnnotationLine,
} from "../annotation/annotation-island-parts";
import { IslandPickerColumn } from "../island-picker-column";
import { PICTURE_CORNERS } from "../picture-corners";
import { SceneOnlyNote } from "../scene-only-note";

import { FrameBackgroundEditor } from "./frame-background-editor";
import { FrameLibraryPictures } from "./frame-library-pictures";
import { FRAME_ISLAND_WIDTH } from "./frame-width";

import type { FrameIslandProps } from "./frame-island";

/** Under the Frame island's row: the finished picture's size, then for a
 * screenshot its corners on a row named as Radius, what it sits on, and its
 * colours while there are any to set; for a recording with scenes, a note
 * that what it sits on is the scene's while the playhead is in one; then
 * the actions: clearing that scene's own background, and putting the size
 * back. The island is as wide as its lines need, and never narrower than a
 * mesh's colours (`FRAME_ISLAND_WIDTH`). The colours' line comes into place
 * under the tiles for a mesh or a colour of your own, and goes for a solid
 * tile or a picture, the lines after it gliding to make room (`IslandParts`).
 * The Library tile opens the library's pictures in a column beside the
 * section (`IslandPickerColumn`), as the text island's templates open; one
 * chosen there is worn, and kept as a tile as a picture of your own is,
 * once. */
export function FrameSection(props: FrameIslandProps) {
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
  return (
    // The controls keep to the start, and the column opens at the end,
    // where the black reveals it as it grows (`TextLookPickers`).
    <div className="flex">
      <IslandParts
        className={cn(
          "flex shrink-0 flex-col gap-separate self-start",
          FRAME_ISLAND_WIDTH.section,
        )}
      >
        <AnnotationLine key="size">
          <div aria-label={t("editor-island-size")} role="group">
            <DimensionsField
              height={props.height}
              onChange={props.onSizeChange}
              width={props.width}
            />
          </div>
        </AnnotationLine>
        {props.radius === undefined ? null : (
          <AnnotationChoiceRow key="radius" title={t("annotation-radius")}>
            <CornerChoice
              corners={PICTURE_CORNERS}
              onChange={(radius) => {
                props.onRadiusChange?.(radius);
              }}
              value={props.radius}
            />
          </AnnotationChoiceRow>
        )}
        {/* The tiles take the section's width and add none of their own, so
         * the lines above set the island's width and the tiles wrap to it.
         * Their box reaches the small inset past them on every side, as the
         * colour grid's does, so a chosen tile's outline at the edge stays
         * inside it and is not cut off while the part moves
         * (`IslandParts`). */}
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
        {choice.showEditor ? (
          <div key="colors">
            <FrameBackgroundEditor
              onChange={choice.changeFromEditor}
              value={props.background}
            />
          </div>
        ) : null}
        {props.scene ? (
          // Outside a scene the note and its action keep their room, unseen,
          // so playback crossing into and out of scenes leaves the island
          // its size.
          <div
            aria-hidden={!props.scene.isInScene}
            className={cn(!props.scene.isInScene && "invisible")}
            key="scene-note"
          >
            <SceneOnlyNote layer="frame" />
          </div>
        ) : null}
        <div className="flex justify-end gap-separate" key="actions">
          {props.scene ? (
            <Button
              className={cn(!props.scene.isInScene && "invisible")}
              isDisabled={
                !props.scene.isInScene || !props.scene.hasOwnBackground
              }
              onPress={props.scene.onClearSceneBackground}
            >
              {t("editor-island-frame-clear-scene-background")}
            </Button>
          ) : null}
          <Button onPress={props.onOriginalSize}>
            {t("editor-island-frame-original-size")}
          </Button>
        </div>
      </IslandParts>
      <IslandPickerColumn
        content={
          isLibraryOpen ? (
            <FrameLibraryPictures
              aspect={props.width / Math.max(props.height, 1)}
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
    </div>
  );
}
