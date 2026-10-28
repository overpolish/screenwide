// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { Button } from "../../../../../primitives/button/button";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { ArrowToolIcon } from "../../../../annotations/annotation-tool-icons";
import { ArrowHeadChoice } from "../../../../annotations/options/arrow-head-choice";
import { PenWeights } from "../../../../annotations/options/pen-weights";
import {
  AnnotationAnimateToggle,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { AnnotationHead } from "../../../../../bindings/AnnotationHead";
import type { EditorIslandContent } from "../../editor-island";

// An arrow drawn on the picture, in hand. Its row ends with whether it draws
// itself in, which only a recording has a clip for, and deleting it. Under
// the row, the live overlay's own pen as it draws one, its weight and its
// heads; its colour, a colour of the user's own among them; and turning it
// round, a one-shot action at the end of its own row.

type ArrowStyle = {
  color: string;
  head: AnnotationHead;
  isAnimated: boolean;
  /** The stroke's width in points. */
  width: number;
};

export type ArrowIslandProps = AnnotationColorProps & {
  /** Whether it can draw itself in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  /** Turns it round, so its head points the other way. */
  onReverse: () => void;
  onStyleChange: (style: Partial<ArrowStyle>) => void;
  style: ArrowStyle;
};

export function arrowIsland(props: ArrowIslandProps): EditorIslandContent {
  const { canAnimate, id, onDelete, onReverse, onStyleChange, style } = props;
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
    key: `arrow-${id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationLine>
          <PenWeights
            onChange={(width) => {
              onStyleChange({ width });
            }}
            value={style.width}
          />
          <ArrowHeadChoice
            onChange={(head) => {
              onStyleChange({ head });
            }}
            value={style.head}
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
        <div className="flex justify-end">
          <Button onPress={onReverse}>
            {t("editor-island-arrow-reverse")}
          </Button>
        </div>
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<ArrowToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-arrow")}
      />
    ),
  };
}
