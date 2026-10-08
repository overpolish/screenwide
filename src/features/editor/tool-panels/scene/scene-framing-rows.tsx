// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NumberField } from "../../../../components/base/input-fields/number-field";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import {
  MAX_SCENE_ZOOM,
  SceneFraming,
} from "../../recording/scenes/recording-scenes";

/** A share as the percent the panel shows, held to one decimal so a redrawn
 * scene is not published as a new value. */
const percent = (share: number) => Math.round(share * 1000) / 10;

/**
 * For the scene under the playhead: how far it zooms into its selected pane,
 * how round that pane's corners are, and which part of it the zoom shows. A
 * pane the scene does not round has no Radius row.
 */
export function SceneFramingRows({
  framing,
  isDisabled,
  onFramingChange,
  onRadiusChange,
  paneName,
  radius,
}: {
  framing: SceneFraming;
  isDisabled: boolean;
  onFramingChange: (framing: Partial<SceneFraming>) => void;
  onRadiusChange: (radius: number) => void;
  /** The selected pane, as the fields' accessible names call it. */
  paneName: string;
  radius: number | null;
}) {
  return (
    <>
      <ControlRow title={t("editor-panels-zoom")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("editor-panels-pane-zoom", { pane: paneName })}
              className="w-48"
              isDisabled={isDisabled}
              maxValue={MAX_SCENE_ZOOM * 100}
              minValue={100}
              onChange={(value) => {
                onFramingChange({ zoom: value / 100 });
              }}
              rightSection="%"
              step={10}
              value={Math.round(framing.zoom * 100)}
            />
          </div>
        )}
      </ControlRow>
      {radius === null ? null : (
        <ControlRow title={t("annotation-radius")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <SliderNumberField
                aria-label={t("editor-panels-pane-radius", { pane: paneName })}
                className="w-48"
                formatOptions={{
                  maximumFractionDigits: 1,
                  minimumFractionDigits: 1,
                }}
                isDisabled={isDisabled}
                maxValue={50}
                minValue={0}
                onChange={onRadiusChange}
                rightSection="%"
                step={0.1}
                value={radius}
              />
            </div>
          )}
        </ControlRow>
      )}

      <ControlRow title={t("editor-panels-position")}>
        {(controlProps) => (
          <div {...controlProps} className="flex gap-control" role="group">
            <NumberField
              aria-label={t("editor-panels-pane-x", { pane: paneName })}
              className="w-20"
              isDisabled={isDisabled}
              leftSection="X"
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                onFramingChange({ focusX: value / 100 });
              }}
              rightSection="%"
              showSteppers={false}
              step={1}
              value={percent(framing.focusX)}
            />
            <NumberField
              aria-label={t("editor-panels-pane-y", { pane: paneName })}
              className="w-20"
              isDisabled={isDisabled}
              leftSection="Y"
              maxValue={100}
              minValue={0}
              onChange={(value) => {
                onFramingChange({ focusY: value / 100 });
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
  );
}
