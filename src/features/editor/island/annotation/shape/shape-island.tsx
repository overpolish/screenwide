// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { ShapeToolIcon } from "../../../../annotations/annotation-tool-icons";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import { HandDrawnToggle } from "../../../../annotations/options/hand-drawn-toggle";
import { PenWeights } from "../../../../annotations/options/pen-weights";
import {
  AnnotationAnimateToggle,
  AnnotationChoiceRow,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationRows,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import type { EditorIslandContent } from "../../editor-island";

// A shape drawn on the picture, in hand. Its row ends with whether it draws
// itself in, which only a recording has a clip for, and deleting it. Under
// the row, the live overlay's own pen, each a row named as the rows round
// it are: its weight, then its corners; then whether it is drawn by hand, a
// fresh wobble each time that is turned on, a toggle on its own line; then
// its colour, a colour of the user's own among them.

type ShapeStyle = {
  color: string;
  isAnimated: boolean;
  isHandDrawn: boolean;
  /** How round its corners are, as a percentage of its shorter side. */
  radius: number;
  /** Its outline's weight in points. */
  width: number;
};

export type ShapeIslandProps = AnnotationColorProps &
  AnnotationPinProps & {
    /** Whether it can draw itself in: a recording's annotations can, a
     * screenshot's, being one instant, cannot. */
    canAnimate: boolean;
    onStyleChange: (style: Partial<ShapeStyle>) => void;
    style: ShapeStyle;
    /** Left out for the pen in hand with nothing chosen. */
    onDelete?: () => void;
  };

export function shapeIsland(props: ShapeIslandProps): EditorIslandContent {
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
    key: "shape",
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationPinNote
          onPinAction={props.onPinAction}
          onSceneReset={props.onSceneReset}
          pin={props.pin}
          placedInScene={props.placedInScene}
        />
        <AnnotationRows>
          <AnnotationChoiceRow key="width" title={annotationSizeLabel("width")}>
            <PenWeights
              onChange={(width) => {
                onStyleChange({ width });
              }}
              value={style.width}
            />
          </AnnotationChoiceRow>
          <AnnotationChoiceRow key="radius" title={t("annotation-radius")}>
            <CornerChoice
              onChange={(radius) => {
                onStyleChange({ radius });
              }}
              value={style.radius}
            />
          </AnnotationChoiceRow>
        </AnnotationRows>
        <div className="flex">
          <HandDrawnToggle
            isHandDrawn={style.isHandDrawn}
            kind="shape"
            onChange={(isHandDrawn) => {
              onStyleChange({ isHandDrawn });
            }}
          />
        </div>
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
        glyph={<ShapeToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-shape")}
      />
    ),
  };
}
