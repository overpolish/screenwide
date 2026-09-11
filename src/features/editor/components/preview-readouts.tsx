// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { memo } from "react";

import { NumberField } from "../../../components/base/input-fields/number-field";
import { Text } from "../../../components/base/text/text";

import { useRecordingOutputDimensions } from "./recording-output-dimensions-channel";

/** The output size in pixels, as the finished file will measure. */
export function PreviewOutputSize({
  height,
  width,
}: {
  height: number;
  width: number;
}) {
  return (
    <Text
      className="tabular-nums text-content-fg-secondary"
      variant="subheadline"
    >
      {width} × {height}
    </Text>
  );
}

/**
 * The recording workspace's size readout. It subscribes to the output channel
 * rather than taking the size as a prop, so a frame resize running at pointer
 * rate re-renders this line alone and leaves its row's memo intact.
 */
export const RecordingOutputSize = memo(function RecordingOutputSize() {
  const dimensions = useRecordingOutputDimensions();
  if (!dimensions) return null;
  return (
    <PreviewOutputSize height={dimensions.height} width={dimensions.width} />
  );
});

/**
 * The zoom readout, beside the size it applies to. The native surface owns the
 * transform: a pinch on it publishes the zoom back here, and typing or
 * scrubbing a zoom here sets it.
 */
export function PreviewZoomField({
  onChange,
  zoomPercent,
}: {
  onChange: (zoomPercent: number) => void;
  zoomPercent: number;
}) {
  return (
    <NumberField
      aria-label="Zoom"
      className="w-20"
      maxValue={800}
      minValue={10}
      onChange={onChange}
      rightSection="%"
      showSteppers={false}
      step={1}
      value={zoomPercent}
    />
  );
}
