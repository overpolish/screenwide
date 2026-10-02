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

const cameraSizes = [
  { ariaLabel: "One third", id: "third", label: "⅓" },
  { ariaLabel: "Two thirds", id: "two-thirds", label: "⅔" },
] satisfies { ariaLabel: string; id: SceneCameraSize; label: string }[];

const corners = [
  { icon: <ArrowUpLeft />, id: "top-left", label: "Top left" },
  { icon: <ArrowUpRight />, id: "top-right", label: "Top right" },
  { icon: <ArrowDownLeft />, id: "bottom-left", label: "Bottom left" },
  { icon: <ArrowDownRight />, id: "bottom-right", label: "Bottom right" },
] satisfies { icon: ReactNode; id: SceneCorner; label: string }[];

const bubbleSizes = [
  { id: "small", label: "Small" },
  { id: "large", label: "Large" },
] satisfies { id: SceneBubbleSize; label: string }[];

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
        <ControlRow title="Swap">
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
        <ControlRow title="Camera size">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Camera size"
                display="label"
                isDisabled={isDisabled}
                items={cameraSizes}
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
        <ControlRow title="Corner">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Corner"
                isDisabled={isDisabled}
                items={corners}
                onSelectionChange={(corner) => {
                  onChange({ corner: corner as SceneCorner });
                }}
                selected={variant.corner ?? DEFAULT_CORNER}
              />
            </div>
          )}
        </ControlRow>
        <ControlRow title="Size">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Camera size"
                display="label"
                isDisabled={isDisabled}
                items={bubbleSizes}
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
