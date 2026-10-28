// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { DrawToolIcon } from "../../../../annotations/annotation-tool-icons";
import { PenWeights } from "../../../../annotations/options/pen-weights";
import {
  AnnotationAnimateToggle,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { EditorIslandContent } from "../../editor-island";

// A freehand drawing on the picture, in hand. Its row ends with whether it
// draws itself in along its line, which only a recording has a clip for,
// and deleting it. Under the row, the live overlay's own pen, its weight
// alone, and its colour, a colour of the user's own among them. It has no
// clean or hand-drawn choice, being a stroke by hand already; clearing every
// drawing at once belongs to the pen rather than to the one in hand, so it
// is not offered here.

type DrawingStyle = {
  color: string;
  isAnimated: boolean;
  /** Its line's weight in points. */
  width: number;
};

export type DrawingIslandProps = AnnotationColorProps & {
  /** Whether it can draw itself in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  onStyleChange: (style: Partial<DrawingStyle>) => void;
  style: DrawingStyle;
};

export function drawingIsland(props: DrawingIslandProps): EditorIslandContent {
  const { canAnimate, id, onDelete, onStyleChange, style } = props;
  return {
    actions: (
      <>
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
    key: `drawing-${id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationLine>
          <PenWeights
            onChange={(width) => {
              onStyleChange({ width });
            }}
            value={style.width}
          />
        </AnnotationLine>
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
