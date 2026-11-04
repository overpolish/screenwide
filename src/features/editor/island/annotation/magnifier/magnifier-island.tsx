// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { MagnifyToolIcon } from "../../../../annotations/annotation-tool-icons";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import {
  AnnotationAnimateToggle,
  AnnotationChoiceRow,
  AnnotationDelete,
  AnnotationRows,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import { LensZoomChoice } from "./lens-zoom-choice";

import type { EditorIslandContent } from "../../editor-island";

// A lens on the picture, in hand: glass that shows what lies under its
// middle larger. Its row ends with whether it grows in as it arrives, which
// only a recording has a clip for, and deleting it. Under the row, each a
// row named as the rows round it are, how many times it enlarges and then
// how round its corners are. The lens is glass: its depth and its edge come
// from the material, refraction, highlight and its own soft shadow, so
// neither a shadow nor a rim colour is offered; how much it shows is how
// big the lens is dragged.

type MagnifierStyle = {
  isAnimated: boolean;
  /** How round the lens is, as a percentage of its shorter side. */
  radius: number;
  /** How many times it enlarges. */
  zoom: number;
};

export type MagnifierIslandProps = AnnotationPinProps & {
  /** Whether it can grow in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onStyleChange: (style: Partial<MagnifierStyle>) => void;
  style: MagnifierStyle;
  /** Left out for the pen in hand with nothing chosen. */
  onDelete?: () => void;
};

export function magnifierIsland(
  props: MagnifierIslandProps,
): EditorIslandContent {
  const { canAnimate, id, onDelete, onStyleChange, style } = props;
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
    key: `magnifier-${id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationPinNote onPinAction={props.onPinAction} pin={props.pin} />
        <AnnotationRows>
          <AnnotationChoiceRow key="zoom" title={annotationSizeLabel("zoom")}>
            <LensZoomChoice
              onChange={(zoom) => {
                onStyleChange({ zoom });
              }}
              value={style.zoom}
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
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<MagnifyToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-magnifier")}
      />
    ),
  };
}
