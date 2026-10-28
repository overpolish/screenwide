// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { CounterToolIcon } from "../../../../annotations/annotation-tool-icons";
import {
  CounterAngleDial,
  CounterSizeChoice,
} from "../../../../annotations/options/counter-choices";
import {
  AnnotationAnimateToggle,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { EditorIslandContent } from "../../editor-island";

// A counter placed on the picture, in hand. Its row ends with whether it
// pops in, which only a recording has a clip for, and deleting it. Under the
// row, the live overlay's own pen as it places one, its size and where its
// tail points; then its colour, a colour of the user's own among them. Its
// number is not offered: counters are numbered in the order they come, and
// close up when one is deleted.

type CounterStyle = {
  /** Where its tail points, in radians clockwise from east. */
  angle: number;
  color: string;
  isAnimated: boolean;
  /** The disc's size in points. */
  size: number;
};

export type CounterIslandProps = AnnotationColorProps & {
  /** Whether it can pop in: a recording's annotations can, a screenshot's,
   * being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  onStyleChange: (style: Partial<CounterStyle>) => void;
  style: CounterStyle;
};

export function counterIsland(props: CounterIslandProps): EditorIslandContent {
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
    key: `counter-${id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationLine>
          <CounterSizeChoice
            onChange={(size) => {
              onStyleChange({ size });
            }}
            value={style.size}
          />
          <CounterAngleDial
            onChange={(angle) => {
              onStyleChange({ angle });
            }}
            value={style.angle}
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
        glyph={<CounterToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-counter")}
      />
    ),
  };
}
