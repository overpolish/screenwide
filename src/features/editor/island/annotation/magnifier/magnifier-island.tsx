// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { MagnifyToolIcon } from "../../../../annotations/annotation-tool-icons";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import {
  AnnotationAnimateToggle,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { EditorIslandContent } from "../../editor-island";

// A magnifier on the picture, in hand: a zoom area and the glass lens that
// shows it larger. Its row ends with whether the lens grows out of its zoom
// area as it arrives, which only a recording has a clip for, and deleting
// it. Under the row, only the corners both round. The lens is glass: its
// depth and its edge come from the material, refraction, highlight and its
// own soft shadow, so neither a shadow nor a rim colour is offered; how big
// it shows is how big the lens is dragged.

type MagnifierStyle = {
  isAnimated: boolean;
  /** How round the zoom area and the lens are, as a percentage of their
   * shorter sides. */
  radius: number;
};

export type MagnifierIslandProps = {
  /** Whether it can grow in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  onStyleChange: (style: Partial<MagnifierStyle>) => void;
  style: MagnifierStyle;
};

export function magnifierIsland(
  props: MagnifierIslandProps,
): EditorIslandContent {
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
    key: `magnifier-${id}`,
    section: (
      <AnnotationLine>
        <CornerChoice
          onChange={(radius) => {
            onStyleChange({ radius });
          }}
          value={style.radius}
        />
      </AnnotationLine>
    ),
    title: (
      <IslandRowTitle
        glyph={<MagnifyToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-magnifier")}
      />
    ),
  };
}
