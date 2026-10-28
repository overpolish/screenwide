// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { FocusToolIcon } from "../../../../annotations/annotation-tool-icons";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import { DimToggle } from "../../../../annotations/options/dim-toggle";
import {
  AnnotationAnimateToggle,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { EditorIslandContent } from "../../editor-island";

// A focus on the picture, in hand: a box kept clear while what lies around
// it is blurred or dimmed. One island for both looks, as the live overlay
// has one tool for them: blurred it is a focus, dimmed a spotlight, and
// changing the look turns one into the other. Its row ends with whether the
// look deepens in round it, which only a recording has a clip for, and
// deleting it. Under the row, the live overlay's own pen and nothing more:
// its corners and its look. Nothing is drawn for it, so it has no colour. A
// focus's blur keeps its strength, the middle step for a new one, and a
// spotlight its soft edge and outside blur, as the live pen leaves them: a
// good standard draws the eye as well as a scale would, without asking.

type FocusStyle = {
  /** Dimmed, a spotlight, rather than blurred, a focus. */
  dims: boolean;
  isAnimated: boolean;
  /** How round its corners are, as a percentage of its shorter side. */
  radius: number;
};

export type FocusIslandProps = {
  /** Whether it can deepen in: a recording's annotations can, a
   * screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  onDelete: () => void;
  onStyleChange: (style: Partial<FocusStyle>) => void;
  style: FocusStyle;
};

export function focusIsland(props: FocusIslandProps): EditorIslandContent {
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
    key: `focus-${id}`,
    section: (
      <AnnotationLine>
        <CornerChoice
          onChange={(radius) => {
            onStyleChange({ radius });
          }}
          value={style.radius}
        />
        <DimToggle
          dims={style.dims}
          onChange={(dims) => {
            onStyleChange({ dims });
          }}
        />
      </AnnotationLine>
    ),
    title: (
      <IslandRowTitle
        glyph={<FocusToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-focus")}
      />
    ),
  };
}
