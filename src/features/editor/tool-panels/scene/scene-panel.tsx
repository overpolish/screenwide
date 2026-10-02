// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToggleButtonGroup } from "react-aria-components";

import { Button } from "../../../../components/base/button/button";
import { NumberField } from "../../../../components/base/input-fields/number-field";
import { Text } from "../../../../components/base/text/text";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import {
  RECORDING_SCENE_CUSTOM_LABEL,
  RECORDING_SCENE_LABELS,
  RECORDING_SCENE_PRESETS,
} from "../../recording/scenes/recording-scene-labels";
import { chosenSceneTemplate } from "../../recording/scenes/recording-scene-template";
import {
  MAX_SCENE_ZOOM,
  RecordingScenePreset,
  sceneNeedsCamera,
} from "../../recording/scenes/recording-scenes";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { ScenePresetTile } from "./scene-preset-tile";
import { sceneSchematic } from "./scene-schematic";
import { SceneTemplateTiles } from "./scene-template-tiles";
import { SceneVariantRows } from "./scene-variant-rows";

/** A share as the percent the panel shows, held to one decimal so a redrawn
 * scene is not published as a new value. */
const percent = (share: number) => Math.round(share * 1000) / 10;

/**
 * The Scene tool's controls: Custom, the presets and your templates, each
 * drawn as it looks, then the options of the preset under the playhead, then
 * for that scene how far it zooms into its selected pane, how round that
 * pane's corners are, and which part of it the zoom shows. The camera is
 * that pane while it is selected and the scene places it; the screen
 * otherwise. Without a camera the presets, and the templates that place one,
 * are not offered.
 *
 * Between scenes Custom is chosen and stands for the recording's own
 * composition, so edits change the whole recording and no scene is made.
 * Pressing Custom there adds a custom scene that starts from that
 * composition; pressing a preset or a template adds one laid out so. Over a
 * scene, a press turns the scene under the playhead into that arrangement,
 * Custom keeping its panes where they are for the select tool to move and
 * resize. An arrangement that places the camera bakes it again where baking
 * was turned off. The actions end the panel: a custom scene kept as a
 * template, the preset's boxes and the whole screen and camera shown again,
 * or the scene taken away.
 */
export function ScenePanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { isLocked, scene, sceneTemplates } = snapshot;

  if (!scene) {
    return (
      <Text variant="footnote">Scenes need a recording with a screen.</Text>
    );
  }
  const framing = scene.framing;
  const radius = scene.radius;
  const paneName = scene.framingPane === "camera" ? "Camera" : "Screen";
  const isIdle =
    (scene.boxes
      ? scene.boxes.camera !== null
      : scene.preset !== null && sceneNeedsCamera({ preset: scene.preset })) &&
    !scene.isBaked;
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
  // A template that needs a camera is hidden without one, like the presets.
  const templates = sceneTemplates.filter(
    (template) => scene.hasCamera || !template.boxes.camera,
  );
  // A custom scene laid out as a template chooses that template's tile.
  const templateId = scene.hasSceneAtPlayhead
    ? chosenSceneTemplate(templates, scene)
    : null;
  const preset = isCustom ? null : scene.preset;
  const selected = templateId ?? preset ?? "custom";
  const presets = scene.hasCamera ? RECORDING_SCENE_PRESETS : [];
  return (
    <div className="flex flex-col gap-section">
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

      {framing ? (
        <>
          <ControlRow title="Zoom">
            {(controlProps) => (
              <div {...controlProps} role="group">
                <SliderNumberField
                  aria-label={`${paneName} zoom`}
                  className="w-48"
                  isDisabled={isLocked}
                  maxValue={MAX_SCENE_ZOOM * 100}
                  minValue={100}
                  onChange={(value) => {
                    change({ sceneFraming: { zoom: value / 100 } });
                  }}
                  rightSection="%"
                  step={10}
                  value={Math.round(framing.zoom * 100)}
                />
              </div>
            )}
          </ControlRow>
          {radius === null ? null : (
            <ControlRow title="Radius">
              {(controlProps) => (
                <div {...controlProps} role="group">
                  <SliderNumberField
                    aria-label={`${paneName} radius`}
                    className="w-48"
                    formatOptions={{
                      maximumFractionDigits: 1,
                      minimumFractionDigits: 1,
                    }}
                    isDisabled={isLocked}
                    maxValue={50}
                    minValue={0}
                    onChange={(radius) => {
                      change({ sceneRadius: radius });
                    }}
                    rightSection="%"
                    step={0.1}
                    value={radius}
                  />
                </div>
              )}
            </ControlRow>
          )}

          <ControlRow title="Position">
            {(controlProps) => (
              <div {...controlProps} className="flex gap-control" role="group">
                <NumberField
                  aria-label={`${paneName} X position`}
                  className="w-20"
                  isDisabled={isLocked}
                  leftSection="X"
                  maxValue={100}
                  minValue={0}
                  onChange={(value) => {
                    change({ sceneFraming: { focusX: value / 100 } });
                  }}
                  rightSection="%"
                  showSteppers={false}
                  step={1}
                  value={percent(framing.focusX)}
                />
                <NumberField
                  aria-label={`${paneName} Y position`}
                  className="w-20"
                  isDisabled={isLocked}
                  leftSection="Y"
                  maxValue={100}
                  minValue={0}
                  onChange={(value) => {
                    change({ sceneFraming: { focusY: value / 100 } });
                  }}
                  rightSection="%"
                  showSteppers={false}
                  step={1}
                  value={percent(framing.focusY)}
                />
              </div>
            )}
          </ControlRow>
        </>
      ) : null}

      {isIdle ? (
        <Text variant="footnote">
          This scene is paused while the camera is not baked in. Choosing a
          scene bakes it again.
        </Text>
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
