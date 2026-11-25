// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { isMainInHand } from "../../recording/formats/format-name";
import { DimensionsField } from "../../tools/dimensions/dimensions-field";
import {
  AnnotationChoiceRow,
  AnnotationLine,
} from "../annotation/annotation-island-parts";
import { IslandPickerRow } from "../island-picker-column";
import { PICTURE_CORNERS } from "../picture-corners";
import { SceneOnlyNote } from "../scene-only-note";

import { FrameFormatChoices, FrameFormatLine } from "./frame-format-picker";
import { FRAME_ISLAND_WIDTH } from "./frame-width";
import { useBackgroundParts } from "./use-background-parts";

import type { FrameIslandProps } from "./frame-island";

/** Under the Frame island's row: the finished picture's size, set on the
 * main canvas alone, as each platform keeps its template's size; then for a
 * screenshot its corners on a row named as Radius, what it sits on, and its
 * colours while there are any to set; for a recording with scenes, a note
 * that what it sits on is the scene's while the playhead is in one; then
 * the foot: for a recording's main canvas, the template it is made for at
 * the start (`FrameFormatLine`), and at the end the actions, clearing that
 * scene's own background, putting the main canvas back to the source's
 * size, and on a platform putting its background back to follow the main
 * canvas's. The island is as wide as its lines need, and never narrower than a
 * mesh's colours (`FRAME_ISLAND_WIDTH`). The colours' line comes into place
 * under the tiles for a mesh or a colour of your own, and goes for a solid
 * tile or a picture, the lines after it gliding to make room
 * (`IslandParts`). The Library tile opens the library's pictures in a
 * column beside the section (`useBackgroundParts`), as the text island's
 * templates open; the template line opens the templates in the same
 * column, one set at a time. */
export function FrameSection(props: FrameIslandProps) {
  const isSized = isMainInHand(props.formats);
  const onMadeForChange = isSized ? props.onMadeForChange : undefined;
  // A platform's background follows the main canvas's until it is changed
  // there; then it can be put back to follow it again.
  const onMainBackground = isSized ? undefined : props.onMainBackground;
  const inHand = props.formats?.list.find(
    (shape) => shape.id === props.formats?.activeId,
  );
  const madeFor = props.formats?.preset ?? null;
  const [isFormatsOpen, setIsFormatsOpen] = useState(false);
  const background = useBackgroundParts({
    ...props,
    aside: {
      content:
        isFormatsOpen && onMadeForChange ? (
          <FrameFormatChoices
            madeFor={madeFor}
            onChoose={(preset) => {
              setIsFormatsOpen(false);
              onMadeForChange(preset);
            }}
          />
        ) : null,
      contentKey: "formats",
      onLibraryOpen: () => {
        setIsFormatsOpen(false);
      },
    },
    aspect: props.width / Math.max(props.height, 1),
  });
  return (
    // The controls keep to the end, and the column opens at the start, where
    // the black reveals it as it grows (`IslandPickerRow`).
    <IslandPickerRow>
      <IslandParts
        className={cn(
          "flex shrink-0 flex-col gap-separate self-start",
          FRAME_ISLAND_WIDTH.section,
        )}
      >
        {isSized ? (
          <AnnotationLine key="size">
            <div aria-label={t("editor-island-size")} role="group">
              <DimensionsField
                height={props.height}
                onChange={props.onSizeChange}
                width={props.width}
              />
            </div>
          </AnnotationLine>
        ) : null}
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
        {background.tiles}
        {background.colors}
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
        {props.scene || isSized || onMainBackground ? (
          <div
            className={cn(
              "flex flex-wrap items-center gap-separate",
              onMadeForChange ? "justify-between" : "justify-end",
            )}
            key="actions"
          >
            {onMadeForChange ? (
              <FrameFormatLine
                controls={background.columnId}
                isOpen={isFormatsOpen}
                madeFor={madeFor}
                onPress={() => {
                  if (!isFormatsOpen) background.closeLibrary();
                  setIsFormatsOpen(!isFormatsOpen);
                }}
                size={props}
              />
            ) : null}
            <div className="flex gap-separate">
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
              {isSized ? (
                <Button onPress={props.onOriginalSize}>
                  {t("editor-island-frame-original-size")}
                </Button>
              ) : null}
              {onMainBackground ? (
                <Button
                  isDisabled={inHand?.ownBackground !== true}
                  onPress={onMainBackground}
                >
                  {t("editor-island-reset-to-main")}
                </Button>
              ) : null}
            </div>
          </div>
        ) : null}
      </IslandParts>
      {background.column}
    </IslandPickerRow>
  );
}
