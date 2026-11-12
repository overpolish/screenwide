// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { DrawToolIcon } from "../../../../annotations/annotation-tool-icons";
import { PenWeights } from "../../../../annotations/options/pen-weights";
import {
  AnnotationAnimateToggle,
  AnnotationChoiceRow,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import type { EditorIslandContent } from "../../editor-island";

// A freehand drawing on the picture, in hand. Its row ends with whether it
// draws itself in along its line, which only a recording has a clip for,
// and deleting it. Under the row, the live overlay's own pen: its weight
// alone, a row named as the rows round it are, and its colour, a colour of
// the user's own among them. It has no clean or hand-drawn choice, being a
// stroke by hand already; clearing every drawing at once belongs to the pen
// rather than to the one in hand, so it is not offered here.

type DrawingStyle = {
  color: string;
  isAnimated: boolean;
  /** Its line's weight in points. */
  width: number;
};

export type DrawingIslandProps = AnnotationColorProps &
  AnnotationPinProps & {
    /** Whether it can draw itself in: a recording's annotations can, a
     * screenshot's, being one instant, cannot. */
    canAnimate: boolean;
    onStyleChange: (style: Partial<DrawingStyle>) => void;
    style: DrawingStyle;
    /** Left out for the pen in hand with nothing chosen. */
    onDelete?: () => void;
  };

export function drawingIsland(props: DrawingIslandProps): EditorIslandContent {
  const { canAnimate, onDelete, onStyleChange, style } = props;
  return {
    actions: (
      <>
        <AnnotationPinToggle onPinAction={props.onPinAction} pin={props.pin} />
        {canAnimate ? (
          <AnnotationAnimateToggle
            isAnimated={style.isAnimated}
            onChange={(isAnimated) => {
              onStyleChange({ isAnimated });
            }}
          />
        ) : null}
        <AnnotationDelete onDelete={onDelete} />
      </>
    ),
    anchor: "end",
    key: "drawing",
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationPinNote
          onPinAction={props.onPinAction}
          onSceneReset={props.onSceneReset}
          pin={props.pin}
          placedInScene={props.placedInScene}
        />
        <AnnotationChoiceRow title={annotationSizeLabel("width")}>
          <PenWeights
            onChange={(width) => {
              onStyleChange({ width });
            }}
            value={style.width}
          />
        </AnnotationChoiceRow>
        <AnnotationColorChoice
          color={style.color}
          onChange={(color) => {
            onStyleChange({ color });
          }}
          onRemoveColor={props.onRemoveColor}
          onSaveColor={props.onSaveColor}
          savedColors={props.savedColors}
        />
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<DrawToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-drawing")}
      />
    ),
  };
}
