// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../../../components/base/text/text";
import { presetName } from "../../../components/shared/background-picker/background";
import { BackgroundPicker } from "../../../components/shared/background-picker/background-picker";
import { Dimensions } from "../../../components/shared/dimensions/dimensions";
import { SliderNumberField } from "../../../components/shared/slider-number-field/slider-number-field";
import { browseBackgroundImage } from "../api";
import { EditorKind } from "../types";

import { useBackgroundPresetMenu } from "./use-background-preset-menu";
import { useBuiltInBackgroundPresets } from "./use-background-presets";
import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/**
 * The Frame tool's own controls: how big the finished picture is, and what it
 * sits on.
 *
 * The numbers are the output canvas, the same ones the size readout under the
 * preview shows and the same ones a frame drag moves, so a value typed here
 * and a corner dragged out there mean the same thing. The reset puts the
 * canvas back to the size the capture arrived at. The background below them
 * fills that canvas behind every layer in it. A layer's own shadow belongs to
 * the layer, and is set where the layer is: in the Select tool's panel.
 */
export function FramePanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { background, backgroundPresets, frame, isSaving } = snapshot;
  const presets = useBuiltInBackgroundPresets();
  const showPresetMenu = useBackgroundPresetMenu((id) => {
    change({ removePreset: id });
  });

  if (!frame) return null;

  const size = (next: { height?: number; width?: number }) => {
    change({ frameSize: next });
  };

  return (
    // `Dimensions` has no disabled state of its own: a save takes the whole
    // group out of reach the way the selection panel's does.
    <div
      className={`flex flex-col gap-section ${isSaving ? "pointer-events-none opacity-50" : ""}`}
    >
      <Dimensions
        height={frame.height}
        initialLinked
        label="Size"
        layout="stacked"
        onReset={() => {
          change({ resetFrame: true });
        }}
        setDimensions={(width, height) => {
          size({ height, width });
        }}
        setHeight={(height) => {
          size({ height });
        }}
        setWidth={(width) => {
          size({ width });
        }}
        width={frame.width}
      />

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Radius</span>
        <SliderNumberField
          aria-label="Radius"
          className="w-48"
          formatOptions={{ maximumFractionDigits: 1, minimumFractionDigits: 1 }}
          isDisabled={isSaving}
          maxValue={50}
          minValue={0}
          onChange={(radius) => {
            change({ frameRadius: radius });
          }}
          rightSection="%"
          step={0.1}
          value={frame.radius ?? 0}
        />
      </div>

      <div className="flex flex-col gap-control">
        <Text as="h2" variant="section">
          Background
        </Text>
        <BackgroundPicker
          isDisabled={isSaving}
          onChange={(next) => {
            change({ background: next });
          }}
          onPickImage={browseBackgroundImage}
          onPresetMenu={(id, anchor) => {
            void showPresetMenu(id, anchor);
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
    </div>
  );
}
