// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { memo, ReactNode } from "react";

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
 * rate re-renders this line alone and leaves the toolbar's memo intact.
 */
export const RecordingOutputSize = memo(function RecordingOutputSize() {
  const dimensions = useRecordingOutputDimensions();
  if (!dimensions) return null;
  return (
    <PreviewOutputSize height={dimensions.height} width={dimensions.width} />
  );
});

/**
 * The editor's toolbar row: what the pointer does on the left, what the picture
 * measures on the right.
 *
 * Memoized: the tool buttons are react-aria trees that cost more to re-render
 * than the whole native preview pane, and none of their props change while a
 * canvas-resize gesture updates the output draft at pointer rate.
 */
export const PreviewToolbar = memo(function PreviewToolbar({
  outputSize,
  tools,
  zoomPercent,
}: {
  zoomPercent: number;
  outputSize?: ReactNode;
  tools?: ReactNode;
}) {
  return (
    <div className="relative flex shrink-0 items-center justify-between gap-section px-window-inset pb-control-inset">
      {/* Groups of tools sit a section apart, as the recording bar's do:
          effects with their panels first, then the view tools. */}
      <div className="flex h-control-height min-w-0 items-center gap-section">
        {tools}
      </div>
      <div className="flex h-control-height shrink-0 items-center gap-section">
        {outputSize}
        {/* Read-only: the native surface owns the transform, and a pinch on it
            publishes the zoom back here. */}
        <Text
          className="tabular-nums text-content-fg-secondary"
          variant="subheadline"
        >
          {zoomPercent}%
        </Text>
      </div>
    </div>
  );
});
