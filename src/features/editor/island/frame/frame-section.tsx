// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { DimensionsField } from "../../tools/dimensions/dimensions-field";
import { BackgroundTiles } from "../../tools/frame/background/background-tiles";
import { useBackgroundChoice } from "../../tools/frame/background/use-background-choice";
import {
  AnnotationChoiceRow,
  AnnotationLine,
} from "../annotation/annotation-island-parts";
import { PICTURE_CORNERS } from "../picture-corners";

import { FrameBackgroundEditor } from "./frame-background-editor";
import { FRAME_ISLAND_WIDTH } from "./frame-width";

import type { FrameIslandProps } from "./frame-island";

/** Under the Frame island's row: the finished picture's size, then for a
 * screenshot its corners on a row named as Radius, what it sits on, its
 * colours while there are any to set, and putting its size back. The
 * island is as wide as its lines need, and never narrower than a mesh's
 * colours (`FRAME_ISLAND_WIDTH`). The colours' line comes into place under
 * the tiles for a mesh or a colour of your own, and goes for a solid tile
 * or a picture, the lines after it gliding to make room (`IslandParts`). */
export function FrameSection(props: FrameIslandProps) {
  const choice = useBackgroundChoice({
    onChange: props.onBackgroundChange,
    presets: props.presets,
    savedPresets: props.savedPresets,
    value: props.background,
  });
  return (
    <IslandParts
      className={cn("flex flex-col gap-separate", FRAME_ISLAND_WIDTH.section)}
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
       * the lines above set the island's width and the tiles wrap to it. Their
       * box reaches the small inset past them on every side, as the colour
       * grid's does, so a chosen tile's outline at the edge stays inside it
       * and is not cut off while the part moves (`IslandParts`). */}
      <div
        className="-m-inset-sm w-0 min-w-[calc(100%+var(--spacing-inset-sm)*2)] p-inset-sm"
        key="tiles"
      >
        <BackgroundTiles
          choice={choice}
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
      <div className="flex justify-end" key="original-size">
        <Button onPress={props.onOriginalSize}>
          {t("editor-island-frame-original-size")}
        </Button>
      </div>
    </IslandParts>
  );
}
