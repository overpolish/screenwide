// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { TIMELINE_MAX_ZOOM, TimelineViewportState } from "./timeline-viewport";
import { TimelineZoomToolbar } from "./timeline-zoom-toolbar";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The toolbar in the gutter it sits in, driving a viewport of its own. */
function ZoomToolbar({ viewport }: { viewport?: TimelineViewportState }) {
  const [isBladeActive, setIsBladeActive] = useState(false);
  const [isRangeActive, setIsRangeActive] = useState(false);
  const [isSnapActive, setIsSnapActive] = useState(true);
  const [state, setState] = useState<TimelineViewportState>(
    viewport ?? { panOffset: 0, zoom: 1 },
  );

  return (
    <div>
      <TimelineZoomToolbar
        isBladeActive={isBladeActive}
        isRangeActive={isRangeActive}
        isSnapActive={isSnapActive}
        onBladeActiveChange={(active) => {
          setIsBladeActive(active);
          if (active) setIsRangeActive(false);
        }}
        onFit={() => {
          setState({ panOffset: 0, zoom: 1 });
        }}
        onRangeActiveChange={(active) => {
          setIsRangeActive(active);
          if (active) setIsBladeActive(false);
        }}
        onSnapActiveChange={setIsSnapActive}
        onZoom={(factor) => {
          setState((current) => ({
            panOffset: current.panOffset,
            zoom: Math.min(
              TIMELINE_MAX_ZOOM,
              Math.max(1, current.zoom * factor),
            ),
          }));
        }}
        viewport={state}
      />
    </div>
  );
}

const meta = {
  component: ZoomToolbar,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline Toolbar",
} satisfies Meta<typeof ZoomToolbar>;

export default meta;
type Story = StoryObj<typeof meta>;

/** At rest: zoomed out, so zoom out and fit have nothing to do. */
export const Default: Story = {};

/** Zoomed in, so every control is live. */
export const Zoomed: Story = {
  args: { viewport: { panOffset: 0.2, zoom: 4 } },
};
