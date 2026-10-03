// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToggleButtonGroup } from "react-aria-components";

import { Button } from "../../../../components/base/button/button";
import { Switch } from "../../../../components/base/switch/switch";
import { Text } from "../../../../components/base/text/text";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import {
  RECORDING_SCENE_CUSTOM_LABEL,
  RECORDING_SCENE_LABELS,
  RECORDING_SCENE_PRESETS,
} from "../../recording/scenes/recording-scene-labels";
import { chosenSceneTemplate } from "../../recording/scenes/recording-scene-template";
import {
  RecordingScenePreset,
  sceneNeedsCamera,
} from "../../recording/scenes/recording-scenes";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { SceneFramingRows } from "./scene-framing-rows";
import { SceneOutputRow } from "./scene-output-row";
import { ScenePresetTile } from "./scene-preset-tile";
import { sceneSchematic } from "./scene-schematic";
import { SceneTemplateTiles } from "./scene-template-tiles";
import { SceneVariantRows } from "./scene-variant-rows";

/**
 * The Scene tool's controls: the output first, since it decides what a scene
 * can place, then Custom, the presets and your templates, each drawn as it
 * looks, then the options of the preset under the playhead, or a custom
 * scene's Swap, then for that scene how far it zooms into its selected pane,
 * how round that pane's corners are, and which part of it the zoom shows.
 * The camera is that pane while it is selected and the scene places it; the
 * screen otherwise. The presets, and the templates that place a camera, are
 * only offered while there is a camera drawn into the screen's video; over a
 * scene that is paused for want of one, only the output is.
 *
 * Between scenes Custom is chosen and stands for the recording's own
 * composition, so edits change the whole recording and no scene is made.
 * Pressing Custom there adds a custom scene that starts from that
 * composition; pressing a preset or a template adds one laid out so. Over a
 * scene, a press turns the scene under the playhead into that arrangement,
 * Custom keeping its panes where they are for the select tool to move and
 * resize. Auto zoom makes the zooms that follow the cursor again, leaving
 * the scenes you made or edited alone. The actions end the panel: a custom
 * scene kept as a template, the preset's boxes and the whole screen and
 * camera shown again, or the scene taken away.
 */
export function ScenePanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { cameraOutput, hasCursorData, isLocked, scene, sceneTemplates } =
    snapshot;

  if (!scene) {
    return (
      <Text variant="footnote">Scenes need a recording with a screen.</Text>
    );
  }
  // The output is read from the panel's own answer, which flips at once; the
  // scene's own flag follows only once the preview has rebuilt.
  const isCombined = cameraOutput ? cameraOutput === "combined" : scene.isBaked;
  const canPlaceCamera = scene.hasCamera && isCombined;
  const framing = scene.framing;
  const radius = scene.radius;
  const paneName = scene.framingPane === "camera" ? "Camera" : "Screen";
  const isIdle =
    (scene.boxes
      ? scene.boxes.camera !== null
      : scene.preset !== null && sceneNeedsCamera({ preset: scene.preset })) &&
    !isCombined;
  const outputRow = cameraOutput ? (
    <SceneOutputRow
      cameraOutput={cameraOutput}
      isDisabled={isLocked}
      onChange={(output) => {
        change({ bakeCamera: output === "combined" });
      }}
    />
  ) : null;
  // A paused scene can only be brought back, so nothing else is offered over
  // one; the lane marks it paused. Without a choice to make, the panel says
  // why instead of standing empty.
  if (isIdle)
    return (
      outputRow ?? (
        <Text variant="footnote">
          This scene is paused while the camera is left out.
        </Text>
      )
    );
  const schematicOf = (preset: RecordingScenePreset) =>
    // Each tile is drawn with the scene's options, which a preset chosen
    // instead keeps.
    sceneSchematic(preset, {
      cameraAspect: scene.cameraAspect,
      canvasAspect: scene.canvasAspect,
      composition: scene.composition,
      screenAspect: scene.screenAspect,
      variant: scene.variant ?? undefined,
    });
  // Between scenes Custom stands for the recording's own composition, which
  // every edit changes; a scene kept in those boxes is custom too.
  const isCustom =
    scene.boxes !== null || scene.preset === null || scene.preset === "full";
  // A template that places a camera is hidden while none is drawn into the
  // video, like the presets.
  const templates = sceneTemplates.filter(
    (template) => canPlaceCamera || !template.boxes.camera,
  );
  // A custom scene laid out as a template chooses that template's tile.
  const templateId = scene.hasSceneAtPlayhead
    ? chosenSceneTemplate(templates, scene)
    : null;
  const preset = isCustom ? null : scene.preset;
  const selected = templateId ?? preset ?? "custom";
  const presets = canPlaceCamera ? RECORDING_SCENE_PRESETS : [];
  return (
    <div className="flex flex-col gap-section">
      {outputRow}

      <ToggleButtonGroup
        aria-label="Scene"
        className="grid justify-between gap-y-control"
        isDisabled={isLocked}
        selectedKeys={new Set([selected])}
        selectionMode="single"
        // Custom, the presets and the templates in rows of three. Each column
        // is a whole number of pixels, so every tile, and every pane inside
        // it, starts on a whole pixel; the columns spread to the edges, and a
        // row left short fills from the start.
        style={{
          gridTemplateColumns:
            "repeat(3, round(down, calc((100% - 2 * var(--spacing-control)) / 3), 1px))",
        }}
      >
        <ScenePresetTile
          canvasAspect={scene.canvasAspect}
          // Over a preset's scene a press keeps the layout and frees its
          // boxes, which the drawing alone cannot say. A zoom is already
          // Custom, so it wears none.
          hasEditBadge={!isCustom}
          id="custom"
          isDisabled={isLocked}
          isSelected={selected === "custom"}
          label={RECORDING_SCENE_CUSTOM_LABEL}
          onPress={() => {
            change({ customizeScene: true });
          }}
          schematic={scene.boxes ?? schematicOf(scene.preset ?? "full")}
        />
        {presets.map((preset) => (
          <ScenePresetTile
            canvasAspect={scene.canvasAspect}
            id={preset}
            isDisabled={isLocked}
            isSelected={selected === preset}
            key={preset}
            label={RECORDING_SCENE_LABELS[preset]}
            onPress={() => {
              change({ scenePreset: preset });
            }}
            schematic={schematicOf(preset)}
          />
        ))}
        <SceneTemplateTiles
          canvasAspect={scene.canvasAspect}
          isDisabled={isLocked}
          onChoose={(template) => {
            change({ chooseSceneTemplate: template });
          }}
          onRemove={(id) => {
            change({ removeSceneTemplate: id });
          }}
          selectedId={templateId}
          templates={templates}
        />
      </ToggleButtonGroup>

      {scene.variant && preset ? (
        <SceneVariantRows
          isDisabled={isLocked}
          onChange={(variant) => {
            change({ sceneVariant: variant });
          }}
          preset={preset}
          variant={scene.variant}
        />
      ) : null}

      {/* A custom scene's Swap trades its panes' boxes and their order, so
          it reads on wherever the camera sits behind the screen. */}
      {scene.hasSceneAtPlayhead && scene.boxes?.camera ? (
        <ControlRow title="Swap">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isLocked}
              isSelected={scene.boxes?.cameraBehind ?? false}
              onChange={() => {
                change({ swapScenePanes: true });
              }}
            />
          )}
        </ControlRow>
      ) : null}

      {framing ? (
        <SceneFramingRows
          framing={framing}
          isDisabled={isLocked}
          onFramingChange={(next) => {
            change({ sceneFraming: next });
          }}
          onRadiusChange={(next) => {
            change({ sceneRadius: next });
          }}
          paneName={paneName}
          radius={radius}
        />
      ) : null}

      {/* The zooms follow the cursor, so a recording without one has none
          to make. */}
      {hasCursorData ? (
        <ControlRow title="Auto zoom">
          {(controlProps) => (
            <Button
              {...controlProps}
              isDisabled={isLocked}
              onPress={() => {
                change({ autoZoomScenes: true });
              }}
            >
              Regenerate
            </Button>
          )}
        </ControlRow>
      ) : null}

      {scene.hasSceneAtPlayhead ? (
        <div className="flex justify-end gap-control">
          {scene.boxes ? (
            <Button
              isDisabled={isLocked}
              onPress={() => {
                change({ saveSceneTemplate: true });
              }}
            >
              Save template
            </Button>
          ) : null}
          <Button
            isDisabled={isLocked}
            onPress={() => {
              change({ resetSceneFraming: true });
            }}
          >
            Reset
          </Button>
          <Button
            isDisabled={isLocked}
            onPress={() => {
              change({ deleteScene: true });
            }}
          >
            Delete
          </Button>
        </div>
      ) : null}
    </div>
  );
}
