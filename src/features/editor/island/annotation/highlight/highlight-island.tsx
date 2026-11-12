// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { HighlightToolIcon } from "../../../../annotations/annotation-tool-icons";
import { FitToggle } from "../../../../annotations/options/fit-toggle";
import { HandDrawnToggle } from "../../../../annotations/options/hand-drawn-toggle";
import { TintToggle } from "../../../../annotations/options/tint-toggle";
import {
  AnnotationAnimateToggle,
  AnnotationColorChoice,
  type AnnotationColorProps,
  AnnotationDelete,
  AnnotationOptions,
} from "../annotation-island-parts";
import {
  AnnotationPinNote,
  type AnnotationPinProps,
  AnnotationPinToggle,
} from "../annotation-pin";

import type { EditorIslandContent } from "../../editor-island";

// A highlight drawn on the picture, in hand. Its row ends with whether it
// draws itself in, which only a recording has a clip for, and deleting it.
// Under the row, the live overlay's own pen, drawn by hand or clean, a fresh
// wobble each time it is turned on; whether it covers the box dragged
// rather than fitting the text, which live the drag decides and here can
// change after; and whether it tints what it covers rather than recolouring
// it, for pages the recolour misreads. Then its colour, a colour of the
// user's own among them.

type HighlightStyle = {
  color: string;
  isAnimated: boolean;
  isHandDrawn: boolean;
  /** Covers the box dragged rather than fitting the text. */
  isManual: boolean;
  /** Tints what it covers rather than recolouring the page under it. */
  isTinted: boolean;
};

export type HighlightIslandProps = AnnotationColorProps &
  AnnotationPinProps & {
    /** Whether it can draw itself in: a recording's annotations can, a
     * screenshot's, being one instant, cannot. */
    canAnimate: boolean;
    onStyleChange: (style: Partial<HighlightStyle>) => void;
    style: HighlightStyle;
    /** Left out for the pen in hand with nothing chosen. */
    onDelete?: () => void;
  };

export function highlightIsland(
  props: HighlightIslandProps,
): EditorIslandContent {
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
    key: "highlight",
    section: (
      <div className="flex flex-col gap-separate">
        <AnnotationPinNote
          onPinAction={props.onPinAction}
          onSceneReset={props.onSceneReset}
          pin={props.pin}
          placedInScene={props.placedInScene}
        />
        <AnnotationOptions>
          <HandDrawnToggle
            isHandDrawn={style.isHandDrawn}
            kind="highlight"
            onChange={(isHandDrawn) => {
              onStyleChange({ isHandDrawn });
            }}
          />
          <FitToggle
            coversBox={style.isManual}
            onChange={(isManual) => {
              onStyleChange({ isManual });
            }}
          />
          <TintToggle
            isTinted={style.isTinted}
            onChange={(isTinted) => {
              onStyleChange({ isTinted });
            }}
          />
        </AnnotationOptions>
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
        glyph={<HighlightToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-highlight")}
      />
    ),
  };
}
