// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { ToolPanelPatch } from "../tool-panel-patch";
import { ToolPanelCameraEffects } from "../tool-panel-store";

import { ModelSwitchRow, WorkingSwitchRow } from "./microphone-switch-rows";

type RowsProps = {
  change: (values: ToolPanelPatch) => void;
  effects: ToolPanelCameraEffects;
  isLocked: boolean;
};

type Tool = keyof ToolPanelCameraEffects["working"];

const switchState = (effects: ToolPanelCameraEffects, tool: Tool) => {
  const isOn =
    tool === "background" ? effects.background !== "kept" : effects[tool];
  if (effects.working[tool]) return "cleaning" as const;
  return isOn ? ("on" as const) : ("off" as const);
};

/**
 * What is behind the person on camera: the room, nothing, the room blurred,
 * or a picture. These rows change which rows follow, so they lead the panel.
 * Choosing a picture background before any picture is chosen asks for one.
 */
export function CameraBackgroundRows({ change, effects, isLocked }: RowsProps) {
  const isRemoved = effects.background !== "kept";
  const choosePicture = () => {
    change({ chooseCameraBackground: true });
  };
  return (
    <>
      <ModelSwitchRow
        description={t("editor-panels-remove-background-description")}
        isLocked={isLocked}
        model={effects.model}
        onChange={(enabled) => {
          change({
            cameraEffects: { background: enabled ? "transparent" : "kept" },
          });
        }}
        onDownload={() => {
          change({ downloadCameraBackground: true });
        }}
        progress={effects.progress.background}
        state={switchState(effects, "background")}
        title={t("editor-panels-remove-background")}
        working={t("editor-panels-remove-background-working")}
      />
      {isRemoved && effects.model?.status === "downloaded" ? (
        <ControlRow title={t("editor-panels-camera-background")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-camera-background")}
                display="label"
                isDisabled={isLocked}
                items={[
                  {
                    id: "transparent",
                    label: t("editor-panels-camera-background-none"),
                  },
                  {
                    id: "blurred",
                    label: t("editor-panels-camera-background-blur"),
                  },
                  {
                    id: "image",
                    label: t("editor-panels-camera-background-picture"),
                  },
                ]}
                onSelectionChange={(next) => {
                  if (next === "image" && !effects.hasImage) {
                    choosePicture();
                    return;
                  }
                  change({
                    cameraEffects: {
                      background: next as ToolPanelCameraEffects["background"],
                    },
                  });
                }}
                selected={effects.background}
              />
            </div>
          )}
        </ControlRow>
      ) : null}
      {effects.background === "image" ? (
        <ControlRow title={t("editor-panels-camera-picture")}>
          {(controlProps) => (
            <Button
              {...controlProps}
              aria-label={t("editor-panels-replace-camera-picture")}
              isDisabled={isLocked}
              onPress={choosePicture}
            >
              {t("editor-panels-replace")}
            </Button>
          )}
        </ControlRow>
      ) : null}
    </>
  );
}

/**
 * Enhance and colour correction, which sit with the panel's other switches.
 * Enhance needs nothing made first, so it takes effect at once.
 */
export function CameraOptionRows({ change, effects, isLocked }: RowsProps) {
  return (
    <>
      <ControlRow title={t("editor-panels-enhance")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={effects.enhance}
            onChange={(enabled) => {
              change({ cameraEffects: { enhance: enabled } });
            }}
          />
        )}
      </ControlRow>
      <WorkingSwitchRow
        isLocked={isLocked}
        onChange={(enabled) => {
          change({ cameraEffects: { colour: enabled } });
        }}
        progress={effects.progress.colour}
        state={switchState(effects, "colour")}
        title={t("editor-panels-colour-correction")}
        working={t("editor-panels-colour-correction-working")}
      />
    </>
  );
}
