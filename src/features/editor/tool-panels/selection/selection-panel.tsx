// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { Dimensions } from "../../../../composites/dimensions/dimensions";
import { SliderNumberField } from "../../../../composites/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { Switch } from "../../../../primitives/switch/switch";
import { Text } from "../../../../primitives/text/text";
import { EditorKind } from "../../types";
import { RotationRow } from "../rotation-row";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { AudioSelectionRows } from "./audio-selection-rows";
import { CameraBackgroundRows, CameraEnhanceRow } from "./camera-effects-rows";
import { CameraFollowFaceRow } from "./camera-follow-face-row";
import { CameraObjectRows } from "./camera-object-rows";

/**
 * The Select tool's own controls: what is selected, how big it is in the
 * finished picture, where it sits in it, and how the screen is turned in
 * depth.
 *
 * Every number is in output pixels, the same units the size readout under the
 * preview uses, so a value typed here and a value dragged out there mean the
 * same thing. Dragging in the preview republishes the placement, and the panel
 * follows it live. The shadow is the layer's own, cast onto whatever the
 * canvas is wearing, so it is set here rather than with the canvas.
 *
 * Padding is the same idea one step out: the frame the layer is placed by can
 * be grown past the picture in it, filled with the colour behind that picture,
 * and the content inside it put back in the middle.
 *
 * An audio track is heard rather than placed: there is nothing to place, so the
 * panel offers how loud it is played back, and for a microphone the speech
 * tools. A camera can have the room behind the person taken away, blurred
 * or replaced, and its colour corrected.
 */
export function SelectionPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { isLocked, scene, selection } = snapshot;

  if (!selection) {
    return <Text variant="body">{t("editor-panels-nothing-selected")}</Text>;
  }

  if (selection.kind === "audio") {
    return (
      <AudioSelectionRows
        change={change}
        isLocked={isLocked}
        selection={selection}
      />
    );
  }

  const place = (placement: { height?: number; width?: number }) => {
    change({ selectionOutput: placement });
  };

  return (
    <div className="flex flex-col gap-section">
      {selection.cameraEffects ? (
        <CameraEnhanceRow
          change={change}
          effects={selection.cameraEffects}
          isLocked={isLocked}
        />
      ) : null}
      {selection.cameraEffects ? (
        <CameraBackgroundRows
          change={change}
          effects={selection.cameraEffects}
          isLocked={isLocked}
        />
      ) : null}
      {selection.cameraEffects?.objects ? (
        <CameraObjectRows
          change={change}
          isLocked={isLocked}
          objects={selection.cameraEffects.objects}
        />
      ) : null}
      {selection.followsFace === undefined ? null : (
        <CameraFollowFaceRow
          follows={selection.followsFace}
          isLocked={isLocked}
          onChange={(follow) => {
            change({ selectionFollowFace: follow });
          }}
        />
      )}
      {/* `Dimensions` has no disabled state of its own: a save takes the whole
          group out of reach the way the output controls do. */}
      <div className={isLocked ? "pointer-events-none opacity-50" : ""}>
        <Dimensions
          height={selection.height}
          initialLinked
          label={t("editor-panels-size")}
          layout="stacked"
          onReset={() => {
            change({ resetSelection: true });
          }}
          setDimensions={(width, height) => {
            place({ height, width });
          }}
          setHeight={(height) => {
            place({ height });
          }}
          setWidth={(width) => {
            place({ width });
          }}
          width={selection.width}
        />
      </div>

      <ControlRow title={t("annotation-radius")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("annotation-radius")}
              className="w-48"
              formatOptions={{
                maximumFractionDigits: 1,
                minimumFractionDigits: 1,
              }}
              isDisabled={isLocked}
              maxValue={50}
              minValue={0}
              onChange={(radius) => {
                change({ selectionRadius: radius });
              }}
              rightSection="%"
              step={0.1}
              value={selection.radius}
            />
          </div>
        )}
      </ControlRow>

      {/* The screen turns in depth, as the select tool's rotation gizmo turns
          it: by the scene's turn inside a scene, which replaces the layer's
          own, and by its own turn outside every scene. */}
      {scene?.hasSceneAtPlayhead ? (
        scene.framingPane === "screen" && scene.rotation ? (
          <RotationRow
            isDisabled={isLocked}
            onChange={(next) => {
              change({ sceneRotation: next });
            }}
            rotation={scene.rotation}
          />
        ) : null
      ) : selection.rotation ? (
        <RotationRow
          isDisabled={isLocked}
          onChange={(next) => {
            change({ selectionRotation: next });
          }}
          rotation={selection.rotation}
        />
      ) : null}

      {/* A camera track is placed in the screen's picture rather than padded
          against a colour of its own, so it is offered no padding. */}
      {selection.kind === "camera" ? null : (
        <ControlRow title={t("editor-panels-inset")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              {/* The bridge carries one kind of request, so every value the
                  knob passes is committed as it is reached; the draft holds
                  the one just sent until the editor answers with it. */}
              <SliderNumberField
                aria-label={t("editor-panels-inset")}
                className="w-48"
                isDisabled={isLocked}
                maxValue={Number.MAX_SAFE_INTEGER}
                minValue={0}
                onChange={(inset) => {
                  change({ selectionInset: inset });
                }}
                rightSection="px"
                sliderMaxValue={Math.max(1, selection.insetMaximum)}
                value={selection.inset}
              />
            </div>
          )}
        </ControlRow>
      )}

      <ControlRow title={t("editor-panels-drop-shadow")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={selection.dropShadow}
            onChange={(next) => {
              change({ selectionDropShadow: next });
            }}
          />
        )}
      </ControlRow>

      {selection.kind === "camera" ? null : (
        <div className="flex justify-end">
          <Button
            isDisabled={isLocked}
            onPress={() => {
              change({ recenterSelection: true });
            }}
          >
            {t("editor-panels-recenter")}
          </Button>
        </div>
      )}
    </div>
  );
}
