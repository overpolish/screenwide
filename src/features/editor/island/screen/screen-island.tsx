// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Monitor } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { ShadowToggle } from "../../../annotations/options/shadow-toggle";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { type CropAspect, CropAspectChoice } from "../crop-aspect-choice";
import { PICTURE_CORNERS } from "../picture-corners";
import { SceneOnlyNote } from "../scene-only-note";

import { ScreenInsetChoice } from "./screen-inset-choice";

import type { EditorIslandContent } from "../editor-island";

// What was captured, the screen of a recording or a screenshot, as it sits
// on the canvas, as the Camera island is the camera's. First the shape its
// crop keeps. How much the crop keeps is dragged on the picture with its
// handles, where a size can be seen rather than read as a number; here it
// only takes a shape, kept as the handles are dragged until another is
// chosen or the crop goes free again. Under it, the opposite of a crop, the
// edges extended and filled with the colour found behind what was
// captured, then the screen's corners and its shadow. All of these hold for
// the whole recording but the corners, which over a scene are that scene's,
// as a note under the lines says: a scene may round the screen its own way
// over its stretch. Then putting what was captured back in the middle, and
// giving the whole capture back.

export type ScreenIslandProps = {
  aspect: CropAspect;
  /** What was captured, which names the island. */
  capture: "recording" | "screenshot";
  /** Whether the screen casts a shadow onto the canvas behind it. */
  hasShadow: boolean;
  /** How far the screen's edges are extended on each side, in output
   * pixels. */
  inset: number;
  /** The screen's shorter side, in output pixels. */
  insetMaximum: number;
  /** Whether the playhead is in a recording's scene, whose own corners
   * `radius` and `onRadiusChange` then are. A screenshot has no scenes. */
  isInScene: boolean;
  onAspectChange: (aspect: CropAspect) => void;
  onInsetChange: (inset: number) => void;
  onRadiusChange: (radius: number) => void;
  /** Puts what was captured back in the middle of the screen's box. */
  onRecenter: () => void;
  /** Gives the whole capture back, free of any shape. */
  onReset: () => void;
  onShadowChange: (hasShadow: boolean) => void;
  /** Percent of the screen's shorter side, the scene's own while
   * `isInScene`. These are the screen's own corners, inside the canvas the
   * Frame island rounds. */
  radius: number;
};

export function screenIsland(props: ScreenIslandProps): EditorIslandContent {
  return {
    key: "screen",
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine key="aspect">
          <CropAspectChoice
            onChange={props.onAspectChange}
            value={props.aspect}
          />
        </AnnotationLine>
        <AnnotationLine key="looks">
          <ScreenInsetChoice
            inset={props.inset}
            insetMaximum={props.insetMaximum}
            onChange={props.onInsetChange}
          />
          <CornerChoice
            corners={PICTURE_CORNERS}
            onChange={props.onRadiusChange}
            value={props.radius}
          />
          <ShadowToggle
            hasShadow={props.hasShadow}
            onChange={props.onShadowChange}
          />
        </AnnotationLine>
        {props.isInScene ? (
          <SceneOnlyNote key="scene-note" layer="screen" />
        ) : null}
        <div className="flex justify-end gap-inline" key="actions">
          <Button onPress={props.onRecenter}>
            {t("editor-island-screen-recenter")}
          </Button>
          <Button onPress={props.onReset}>
            {t("editor-island-reset-crop")}
          </Button>
        </div>
      </IslandFlow>
    ),
    title: (
      <IslandRowTitle
        glyph={<Monitor aria-hidden className="size-icon" />}
        title={
          props.capture === "screenshot"
            ? t("editor-island-screenshot")
            : t("editor-island-screen")
        }
      />
    ),
  };
}
