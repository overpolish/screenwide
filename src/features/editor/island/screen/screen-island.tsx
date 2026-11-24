// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Monitor } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { IslandParts } from "../../../../primitives/island/island-parts";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { ShadowToggle } from "../../../annotations/options/shadow-toggle";
import {
  AnnotationChoiceRow,
  AnnotationRows,
} from "../annotation/annotation-island-parts";
import { CropLayerButton } from "../crop-layer-button";
import { FramingMenu } from "../framing-menu";
import { IslandRotation } from "../island-rotation";
import { LayerCornerChoice } from "../layer-corner-choice";
import { SceneOnlyNote } from "../scene-only-note";

import { ScreenInsetChoice } from "./screen-inset-choice";

import type { Measure } from "../../../../bindings/Measure";
import type { SceneRotation } from "../../../../bindings/SceneRotation";
import type { ToolPanelFormats } from "../../tools/tool-panel-frame";
import type { FramingPatch } from "../../tools/tool-panel-patch";
import type { EditorIslandContent } from "../editor-island";

// What was captured, the screen of a recording or a screenshot, as it sits
// on the canvas, as the Camera island is the camera's. Its crop is its own,
// taken up at the row's end. Here, its named rows first: the opposite of a
// crop, the edges extended and filled with the colour found behind what was
// captured; the screen's corners; how it is turned. Then its shadow, a
// toggle on a line of its own at the start. All of these hold for the whole
// recording but the inset, the corners and the turn, which over a scene are
// that scene's, as a note under the lines says: a scene may pad, round and
// turn the screen its own way over its stretch. Then putting what was
// captured back in the middle of its box, the scene's box over a scene, and
// the screen back where a fresh project places it, the scene's pane back in
// its layout's box over a scene. With a platform in hand the row starts with
// which platforms its framing is for (`FramingMenu`), and outside a scene
// the screen is put back where the main canvas lays it out instead.

export type ScreenIslandProps = {
  /** What was captured, which names the island. */
  capture: "recording" | "screenshot";
  /** Whether the screen casts a shadow onto the canvas behind it. */
  hasShadow: boolean;
  /** How far the screen's edges are extended on each side, in percent of
   * the canvas's shorter side, the scene's own while `isInScene`. */
  inset: number;
  /** Whether the screen sits where a fresh project places it, the scene's
   * own pane where its layout puts it while `isInScene`. */
  isAtDefaultPlacement: boolean;
  /** Whether the playhead is in a recording's scene, whose own inset,
   * corners, turn and Recenter the island then sets. A screenshot has no
   * scenes. */
  isInScene: boolean;
  /** Takes up the screen's crop. */
  onCrop: () => void;
  onInsetChange: (inset: number) => void;
  onRadiusChange: (radius: Measure) => void;
  /** Puts what was captured back in the middle of the screen's box, the
   * scene's while `isInScene`. */
  onRecenter: () => void;
  /** Puts the screen back where a fresh project places it, its crop,
   * padding, corners, shadow and turn kept; the scene's pane back in the
   * box its layout gives it while `isInScene`. */
  onResetPlacement: () => void;
  onShadowChange: (hasShadow: boolean) => void;
  /** The scene's own while `isInScene`. These are the screen's own
   * corners, inside the canvas the Frame island rounds. */
  radius: Measure;
  /** A recording's formats, and their asks of the platform in hand's
   * framing; absent for a screenshot. */
  framing?: {
    formats: ToolPanelFormats;
    onPatch: (patch: FramingPatch) => void;
  };
  /** Turns the screen; left out where it cannot be turned. */
  onRotationChange?: (rotation: SceneRotation) => void;
  /** How the screen is turned, in degrees about each axis, the scene's own
   * while `isInScene`. */
  rotation?: SceneRotation;
};

export function screenIsland(props: ScreenIslandProps): EditorIslandContent {
  const framing = props.framing;
  // Outside a scene, with a platform in hand, the screen goes back to where
  // the main canvas lays it out.
  const inHand = framing?.formats.list.find(
    (shape) => shape.id === framing.formats.activeId,
  );
  const isOnPlatform =
    framing !== undefined &&
    !props.isInScene &&
    framing.formats.list[0]?.id !== framing.formats.activeId;
  return {
    actions: (
      <>
        {framing ? (
          <FramingMenu formats={framing.formats} onPatch={framing.onPatch} />
        ) : null}
        <CropLayerButton onPress={props.onCrop} />
      </>
    ),
    // Hung from the row's end, as the other editor islands in that corner
    // are, so its rows span the island when the row is the wider part.
    anchor: "end",
    key: "screen",
    section: (
      <IslandParts className="flex flex-col gap-separate">
        <AnnotationRows key="rows">
          <AnnotationChoiceRow
            key="inset"
            title={t("editor-island-screen-inset")}
          >
            <ScreenInsetChoice
              inset={props.inset}
              onChange={props.onInsetChange}
            />
          </AnnotationChoiceRow>
          <AnnotationChoiceRow key="corners" title={t("annotation-radius")}>
            <LayerCornerChoice
              onChange={props.onRadiusChange}
              value={props.radius}
            />
          </AnnotationChoiceRow>
          {props.rotation && props.onRotationChange ? (
            <IslandRotation
              key="rotation"
              onChange={props.onRotationChange}
              rotation={props.rotation}
            />
          ) : null}
        </AnnotationRows>
        <div className="flex" key="shadow">
          <ShadowToggle
            hasShadow={props.hasShadow}
            onChange={props.onShadowChange}
          />
        </div>
        {props.isInScene ? (
          <SceneOnlyNote key="scene-note" layer="screen" />
        ) : null}
        <div className="flex justify-end gap-separate" key="actions">
          <Button onPress={props.onRecenter}>
            {t("editor-island-screen-recenter")}
          </Button>
          <Button
            isDisabled={
              isOnPlatform
                ? inHand?.placed !== true
                : props.isAtDefaultPlacement
            }
            onPress={
              isOnPlatform
                ? () => {
                    framing.onPatch({ kind: "main-placement" });
                  }
                : props.onResetPlacement
            }
          >
            {isOnPlatform
              ? t("editor-island-reset-to-main")
              : t("editor-island-reset-placement")}
          </Button>
        </div>
      </IslandParts>
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
