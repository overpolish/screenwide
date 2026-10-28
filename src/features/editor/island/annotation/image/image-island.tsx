// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { Button } from "../../../../../primitives/button/button";
import { IslandRowTitle } from "../../../../../primitives/island/row-title/island-row-title";
import { ImageToolIcon } from "../../../../annotations/annotation-tool-icons";
import { CornerChoice } from "../../../../annotations/options/corner-choice";
import { LoopToggle } from "../../../../annotations/options/loop-toggle";
import { ShadowToggle } from "../../../../annotations/options/shadow-toggle";
import { SwayToggle } from "../../../../annotations/options/sway-toggle";
import { IslandRotation } from "../../island-rotation";
import {
  AnnotationAnimateToggle,
  AnnotationDelete,
  AnnotationLine,
} from "../annotation-island-parts";

import type { SceneRotation } from "../../../../../bindings/SceneRotation";
import type { EditorIslandContent } from "../../editor-island";

// A picture placed on the picture, in hand. Its row ends with whether it
// pops in, which only a recording has a clip for, and deleting it. Under
// the row, how it is turned, in depth and flat, as the screen in a scene
// is; then one line with its corners at the start and its toggles at the
// end: whether it casts a shadow, and in a recording only whether it sways,
// a sway of its own each time that is turned on, and, for a moving
// picture, whether it loops, after the shadow so their coming and going
// moves nothing else. Giving it another picture and mirroring it are
// one-shot actions, at the end in their own row. The frame a moving
// picture starts on is not offered: it starts on its first.

type ImageStyle = {
  hasShadow: boolean;
  isAnimated: boolean;
  /** A moving picture loops for its whole clip rather than playing once. */
  loops: boolean;
  /** How round its corners are, as a percentage of its shorter side. */
  radius: number;
  rotation: SceneRotation;
  sways: boolean;
};

export type ImageIslandProps = {
  /** Whether it can pop in, sway and play: a recording's annotations can,
   * a screenshot's, being one instant, cannot. */
  canAnimate: boolean;
  id: string;
  /** Whether the picture moves, as an animated GIF does. */
  isMoving: boolean;
  onDelete: () => void;
  /** Mirrors the picture across its upright axis. */
  onFlip: () => void;
  /** Gives it another picture, keeping its place, size and turn. */
  onReplace: () => void;
  onStyleChange: (style: Partial<ImageStyle>) => void;
  style: ImageStyle;
};

export function imageIsland(props: ImageIslandProps): EditorIslandContent {
  const { canAnimate, id, isMoving, onDelete, onStyleChange, style } = props;
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
    key: `image-${id}`,
    section: (
      <div className="flex flex-col gap-separate">
        <IslandRotation
          onChange={(rotation) => {
            onStyleChange({ rotation });
          }}
          rotation={style.rotation}
        />
        <AnnotationLine>
          <CornerChoice
            onChange={(radius) => {
              onStyleChange({ radius });
            }}
            value={style.radius}
          />
          <div className="flex items-center gap-separate">
            <ShadowToggle
              hasShadow={style.hasShadow}
              onChange={(hasShadow) => {
                onStyleChange({ hasShadow });
              }}
            />
            {canAnimate ? (
              <SwayToggle
                onChange={(sways) => {
                  onStyleChange({ sways });
                }}
                sways={style.sways}
              />
            ) : null}
            {canAnimate && isMoving ? (
              <LoopToggle
                loops={style.loops}
                onChange={(loops) => {
                  onStyleChange({ loops });
                }}
              />
            ) : null}
          </div>
        </AnnotationLine>
        <div className="flex justify-end gap-separate">
          <Button onPress={props.onReplace}>
            {t("editor-island-image-replace")}
          </Button>
          <Button onPress={props.onFlip}>
            {t("editor-island-image-flip")}
          </Button>
        </div>
      </div>
    ),
    title: (
      <IslandRowTitle
        glyph={<ImageToolIcon aria-hidden className="size-icon" />}
        title={t("editor-island-image")}
      />
    ),
  };
}
