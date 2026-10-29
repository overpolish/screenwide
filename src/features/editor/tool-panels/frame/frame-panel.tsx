// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { SliderNumberField } from "../../../../composites/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { browseBackgroundImage } from "../../api";
import { EditorKind } from "../../types";
import { DimensionsField } from "../dimensions/dimensions-field";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { presetName } from "./background/background";
import { BackgroundPicker } from "./background/background-picker";
import { useBuiltInBackgroundPresets } from "./use-background-presets";

/**
 * The Frame tool's own controls: how big the finished picture is, and what it
 * sits on.
 *
 * The numbers are the output canvas, the same ones the size readout under the
 * preview shows and the same ones a frame drag moves, so a value typed here
 * and a corner dragged out there mean the same thing. Reset puts the canvas
 * back to the size the capture arrived at. The background below them
 * fills that canvas behind every layer in it. A layer's own shadow belongs to
 * the layer, and is set where the layer is: in the Select tool's panel.
 */
export function FramePanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { background, backgroundPresets, frame, isLocked } = snapshot;
  const presets = useBuiltInBackgroundPresets();

  if (!frame) return null;

  return (
    <div className="flex flex-col gap-separate">
      <ControlRow title={t("editor-panels-size")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <DimensionsField
              height={frame.height}
              isDisabled={isLocked}
              onChange={(frameSize) => {
                change({ frameSize });
              }}
              width={frame.width}
            />
          </div>
        )}
      </ControlRow>

      <ControlRow title={t("annotation-radius")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("annotation-radius")}
              className="w-48"
              isDisabled={isLocked}
              maxValue={50}
              minValue={0}
              onChange={(radius) => {
                change({ frameRadius: radius });
              }}
              step={0.1}
              unit="%"
              value={frame.radius ?? 0}
            />
          </div>
        )}
      </ControlRow>

      <div className="flex flex-col gap-inline">
        <h2 className="text-heading text-fg-secondary">
          {t("background-label")}
        </h2>
        <BackgroundPicker
          isDisabled={isLocked}
          onChange={(next) => {
            change({ background: next });
          }}
          onPickImage={browseBackgroundImage}
          onRemovePreset={(id) => {
            change({ removePreset: id });
          }}
          onSavePreset={(saved) => {
            change({
              savePreset: {
                background: saved,
                id: `saved-${Date.now().toString(36)}`,
                name: presetName(saved),
              },
            });
          }}
          presets={presets}
          savedPresets={backgroundPresets}
          value={background}
        />
      </div>
      <div className="flex justify-end">
        <Button
          isDisabled={isLocked}
          onPress={() => {
            change({ resetFrame: true });
          }}
        >
          {t("editor-panels-reset")}
        </Button>
      </div>
    </div>
  );
}
