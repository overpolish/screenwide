// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { DimensionsField } from "../../tool-panels/dimensions/dimensions-field";
import { BackgroundTiles } from "../../tool-panels/frame/background/background-tiles";
import { useBackgroundChoice } from "../../tool-panels/frame/background/use-background-choice";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { PICTURE_CORNERS } from "../picture-corners";

import { FrameBackgroundEditor } from "./frame-background-editor";

import type { FrameIslandProps } from "./frame-island";

/** Under the Frame island's row: the finished picture's size, with its
 * corners beside it for a screenshot, what it sits on, its colours while
 * there are any to set, and putting its size back. The island is as wide
 * as that first line needs. The colours' line comes into place under the
 * tiles for a mesh or a colour of your own, and goes for a solid tile or a
 * picture, the lines after it gliding to make room (`IslandFlow`). */
export function FrameSection(props: FrameIslandProps) {
  const choice = useBackgroundChoice({
    onChange: props.onBackgroundChange,
    presets: props.presets,
    savedPresets: props.savedPresets,
    value: props.background,
  });
  return (
    <IslandFlow className="flex flex-col gap-separate">
      <AnnotationLine key="size">
        <div aria-label={t("editor-panels-size")} role="group">
          <DimensionsField
            height={props.height}
            onChange={props.onSizeChange}
            width={props.width}
          />
        </div>
        {props.radius === undefined ? null : (
          <CornerChoice
            corners={PICTURE_CORNERS}
            onChange={(radius) => {
              props.onRadiusChange?.(radius);
            }}
            value={props.radius}
          />
        )}
      </AnnotationLine>
      {/* The tiles take the line's width and add none of their own, so the
       * line above sets the island's width and the tiles wrap to it. Their
       * box reaches the small inset past them on every side, as the colour
       * grid's does, so a chosen tile's outline at the edge stays inside it
       * and is not cut off while the part moves (`IslandFlow`). */}
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
    </IslandFlow>
  );
}
