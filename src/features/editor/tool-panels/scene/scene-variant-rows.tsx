// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowDownLeft,
  ArrowDownRight,
  ArrowUpLeft,
  ArrowUpRight,
} from "lucide-react";
import { ReactNode } from "react";

import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import {
  DEFAULT_BUBBLE_SIZE,
  DEFAULT_CAMERA_SIZE,
  DEFAULT_CORNER,
  SceneBubbleSize,
  SceneCameraSize,
  SceneCorner,
  SceneVariant,
} from "../../recording/scenes/recording-scene-variant";
import { RecordingScenePreset } from "../../recording/scenes/recording-scenes";

const cameraSizes = (): {
  ariaLabel: string;
  id: SceneCameraSize;
  label: string;
}[] => [
  { ariaLabel: t("editor-panels-one-third"), id: "third", label: "⅓" },
  { ariaLabel: t("editor-panels-two-thirds"), id: "two-thirds", label: "⅔" },
];

const corners = (): { icon: ReactNode; id: SceneCorner; label: string }[] => [
  {
    icon: <ArrowUpLeft />,
    id: "top-left",
    label: t("editor-panels-corner-top-left"),
  },
  {
    icon: <ArrowUpRight />,
    id: "top-right",
    label: t("editor-panels-corner-top-right"),
  },
  {
    icon: <ArrowDownLeft />,
    id: "bottom-left",
    label: t("editor-panels-corner-bottom-left"),
  },
  {
    icon: <ArrowDownRight />,
    id: "bottom-right",
    label: t("editor-panels-corner-bottom-right"),
  },
];

const bubbleSizes = (): { id: SceneBubbleSize; label: string }[] => [
  { id: "small", label: t("editor-panels-size-small") },
  { id: "large", label: t("editor-panels-size-large") },
];

/**
 * The options of the preset under the playhead, under the tiles: which side
 * of a pair the camera takes and how much of it, or which corner a picture
 * in picture's camera sits over and how big it is. Presets without options
 * show none.
 */
export function SceneVariantRows({
  isDisabled,
  onChange,
  preset,
  variant,
}: {
  isDisabled: boolean;
  onChange: (variant: SceneVariant) => void;
  preset: RecordingScenePreset;
  variant: SceneVariant;
}) {
  if (preset === "split-two-thirds" || preset === "stacked")
    return (
      <>
        <ControlRow title={t("editor-panels-swap")}>
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isDisabled}
              isSelected={variant.swap ?? false}
              onChange={(swap) => {
                onChange({ swap });
              }}
            />
          )}
        </ControlRow>
        <ControlRow title={t("editor-panels-camera-size")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-camera-size")}
                display="label"
                isDisabled={isDisabled}
                items={cameraSizes()}
                onSelectionChange={(size) => {
                  onChange({ cameraSize: size as SceneCameraSize });
                }}
                selected={variant.cameraSize ?? DEFAULT_CAMERA_SIZE}
              />
            </div>
          )}
        </ControlRow>
      </>
    );
  if (preset === "picture-in-picture")
    return (
      <>
        <ControlRow title={t("editor-panels-corner")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-corner")}
                isDisabled={isDisabled}
                items={corners()}
                onSelectionChange={(corner) => {
                  onChange({ corner: corner as SceneCorner });
                }}
                selected={variant.corner ?? DEFAULT_CORNER}
              />
            </div>
          )}
        </ControlRow>
        <ControlRow title={t("editor-panels-size")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("editor-panels-camera-size")}
                display="label"
                isDisabled={isDisabled}
                items={bubbleSizes()}
                onSelectionChange={(size) => {
                  onChange({ size: size as SceneBubbleSize });
                }}
                selected={variant.size ?? DEFAULT_BUBBLE_SIZE}
              />
            </div>
          )}
        </ControlRow>
      </>
    );
  return null;
}
