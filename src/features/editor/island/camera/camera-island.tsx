// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Video } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IslandFlow } from "../../../../primitives/island/island-flow";
import { IslandKeyHint } from "../../../../primitives/island/island-key-hint";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";
import { CornerChoice } from "../../../annotations/options/corner-choice";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { ShadowToggle } from "../../../annotations/options/shadow-toggle";
import { AnnotationLine } from "../annotation/annotation-island-parts";
import { FollowFaceToggle } from "../follow-face-toggle";
import { IslandModelLine } from "../island-model-line";
import { IslandWorkLine } from "../island-work-line";
import { SceneOnlyNote } from "../scene-only-note";

import { CameraBackgroundLine } from "./camera-background-line";
import { EnhanceGlyph, RemoveBackgroundGlyph } from "./camera-glyphs";
import { CameraObjectsLine } from "./camera-objects-line";

import type { ToolPanelCameraEffects } from "../../tool-panels/tool-panel-selection";
import type { EditorIslandContent } from "../editor-island";

// The camera, for the whole recording, as the Screen island is the
// screen's; its crop is the Crop tool's. One line of animated toggles and
// presets, in the order the camera's settings are read: what changes the
// camera's picture (Enhance, then taking the room away), its corners, its
// shadow, then whether it follows the face, which holds the face in the
// middle of the crop. Then what taking the room away brings: what goes
// behind the person, drawn as each looks, with choosing another picture at
// that line's end, and objects put right where the cutout has them wrong.
// A tool whose model is not on this computer yet offers the download in
// its place, and a tool at work shows how far it has got, each on a line
// that comes and goes as the others glide (`IslandFlow`). Over a scene,
// which may round the camera and have it follow the face its own way, the
// corners and the following are that scene's, and a note at the island's
// foot says so.

export type CameraIslandProps = {
  effects: ToolPanelCameraEffects;
  /** Whether the camera follows the face, the scene's own while
   * `isInScene`; outside every scene, and in a scene that does not say
   * otherwise, the recording's. */
  followFace: boolean;
  /** Whether the camera casts a shadow onto what is behind it. */
  hasShadow: boolean;
  /** While the face is first looked for, once `followFace` is turned on. */
  isFindingFace: boolean;
  /** Whether the playhead is in a scene, whose own corners and following
   * of the face `radius`, `followFace` and their handlers then are. */
  isInScene: boolean;
  onBackgroundChange: (
    background: ToolPanelCameraEffects["background"],
  ) => void;
  /** Asks for a picture to show behind the person. */
  onChoosePicture: () => void;
  onDownloadBackgroundModel: () => void;
  onDownloadObjectsModel: () => void;
  onEnhanceChange: (enhance: boolean) => void;
  onFollowFaceChange: (followFace: boolean) => void;
  /** Turns picking objects on the preview on or off. */
  onPickObjects: (isPicking: boolean) => void;
  onRadiusChange: (radius: number) => void;
  onRemoveObject: (id: number) => void;
  onShadowChange: (hasShadow: boolean) => void;
  /** Goes to where an object was picked, choosing it. */
  onShowObject: (id: number) => void;
  /** Percent of the camera's shorter side, the scene's own while
   * `isInScene`; half is a circle. */
  radius: number;
};

export function cameraIsland(props: CameraIslandProps): EditorIslandContent {
  const { effects } = props;
  const isRemoved = effects.background !== "kept";
  const hasBackgroundModel = effects.model?.status === "downloaded";
  const { objects } = effects;
  return {
    key: "camera",
    section: (
      <IslandFlow className="flex flex-col gap-separate">
        <AnnotationLine key="looks">
          <OptionToggle
            glyph={(isOn) => <EnhanceGlyph isEnhanced={isOn} />}
            isOn={effects.enhance}
            label={t("editor-island-enhance")}
            onChange={props.onEnhanceChange}
          />
          {hasBackgroundModel ? (
            <OptionToggle
              glyph={(isOn) => <RemoveBackgroundGlyph isRemoved={isOn} />}
              isOn={isRemoved}
              label={t("editor-island-remove-background")}
              onChange={(remove) => {
                props.onBackgroundChange(remove ? "transparent" : "kept");
              }}
            />
          ) : null}
          {/* Square, rounded and round: a camera is often drawn as a
              circle, so its corners run all the way to one. */}
          <CornerChoice onChange={props.onRadiusChange} value={props.radius} />
          <ShadowToggle
            hasShadow={props.hasShadow}
            onChange={props.onShadowChange}
          />
          <FollowFaceToggle
            isFindingFace={props.isFindingFace}
            isFollowing={props.followFace}
            onChange={props.onFollowFaceChange}
          />
        </AnnotationLine>
        {effects.working.colour ? (
          <IslandWorkLine
            key="enhance-work"
            label={t("editor-island-enhance-working")}
            value={effects.progress.colour}
          />
        ) : null}
        {effects.model && !hasBackgroundModel ? (
          <IslandModelLine
            key="background-model"
            model={effects.model}
            onDownload={props.onDownloadBackgroundModel}
            title={t("editor-island-remove-background")}
          />
        ) : null}
        {effects.working.background ? (
          <IslandWorkLine
            key="background-work"
            label={t("editor-island-remove-background-working")}
            value={effects.progress.background}
          />
        ) : null}
        {isRemoved && hasBackgroundModel ? (
          <CameraBackgroundLine
            background={effects.background}
            hasImage={effects.hasImage}
            key="background"
            onChange={props.onBackgroundChange}
            onChoosePicture={props.onChoosePicture}
          />
        ) : null}
        {isRemoved && objects?.model ? (
          objects.model.status === "downloaded" ? (
            <CameraObjectsLine
              key="objects"
              objects={objects}
              onPick={props.onPickObjects}
              onRemove={props.onRemoveObject}
              onShow={props.onShowObject}
            />
          ) : (
            <IslandModelLine
              key="objects-model"
              model={objects.model}
              onDownload={props.onDownloadObjectsModel}
              title={t("editor-island-camera-objects")}
            />
          )
        ) : null}
        {objects?.isPicking ? (
          <IslandKeyHint
            binding="Alt"
            key="objects-hint"
            message={(key) => t("editor-island-camera-objects-hint", { key })}
          />
        ) : null}
        {objects?.working ? (
          <IslandWorkLine
            key="objects-work"
            label={t("editor-island-camera-objects-working")}
            value={objects.progress}
          />
        ) : null}
        {props.isInScene ? (
          <SceneOnlyNote key="scene-note" layer="camera" />
        ) : null}
      </IslandFlow>
    ),
    title: (
      <IslandRowTitle
        glyph={<Video aria-hidden className="size-icon" />}
        title={t("editor-island-camera")}
      />
    ),
  };
}
