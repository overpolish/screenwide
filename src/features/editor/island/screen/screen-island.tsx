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
import { IslandRotation } from "../island-rotation";
import { PICTURE_CORNERS } from "../picture-corners";
import { SceneOnlyNote } from "../scene-only-note";

import { ScreenInsetChoice } from "./screen-inset-choice";

import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { EditorIslandContent } from "../editor-island";

// What was captured, the screen of a recording or a screenshot, as it sits
// on the canvas, as the Camera island is the camera's. Its crop is the Crop
// tool's. Here, the opposite of a crop, the edges extended and filled with
// the colour found behind what was captured, then the screen's corners and
// its shadow, and how it is turned. All of these hold for the whole
// recording but the inset, the corners and the turn, which over a scene
// are that scene's, as a note under the lines says: a scene may pad, round
// and turn the screen its own way over its stretch. Then putting what was
// captured back in the middle of its box, the scene's box over a scene.

export type ScreenIslandProps = {
  /** What was captured, which names the island. */
  capture: "recording" | "screenshot";
  /** Whether the screen casts a shadow onto the canvas behind it. */
  hasShadow: boolean;
  /** How far the screen's edges are extended on each side, as a share of
   * its picture's shorter side, the scene's own (a share of its box's)
   * while `isInScene`. */
  inset: number;
  /** Whether the playhead is in a recording's scene, whose own inset,
   * corners, turn and Recenter the island then sets. A screenshot has no
   * scenes. */
  isInScene: boolean;
  onInsetChange: (inset: number) => void;
  onRadiusChange: (radius: number) => void;
  /** Puts what was captured back in the middle of the screen's box, the
   * scene's while `isInScene`. */
  onRecenter: () => void;
  onShadowChange: (hasShadow: boolean) => void;
  /** Percent of the screen's shorter side, the scene's own while
   * `isInScene`. These are the screen's own corners, inside the canvas the
   * Frame island rounds. */
  radius: number;
  /** Turns the screen; left out where it cannot be turned. */
  onRotationChange?: (rotation: SceneRotation) => void;
  /** How the screen is turned, in degrees about each axis, the scene's own
   * while `isInScene`. */
  rotation?: SceneRotation;
};

export function screenIsland(props: ScreenIslandProps): EditorIslandContent {
  return {
    key: "screen",
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine key="looks">
          <ScreenInsetChoice
            inset={props.inset}
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
        {props.rotation && props.onRotationChange ? (
          <IslandRotation
            key="rotation"
            onChange={props.onRotationChange}
            rotation={props.rotation}
          />
        ) : null}
        {props.isInScene ? (
          <SceneOnlyNote key="scene-note" layer="screen" />
        ) : null}
        <div className="flex justify-end" key="actions">
          <Button onPress={props.onRecenter}>
            {t("editor-island-screen-recenter")}
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
