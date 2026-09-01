// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { type GlideAction, GlideDetector } from "./glide-detection";
import { GlidePreview } from "./glide-preview";
import { describeRegion, type GlideRegion } from "./glide-regions";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** A full-height left cell, refined per story. */
const region = (
  cells: Partial<GlideRegion> & Pick<GlideRegion, "gridCols">,
): GlideRegion => ({
  colSpan: 1,
  colStart: 0,
  rowSpan: 2,
  rowStart: 0,
  ...cells,
});

/** Stands in for an extracted app icon, so Storybook needs no files. */
const sampleIcon =
  "data:image/svg+xml;utf8," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">' +
      '<rect width="16" height="16" rx="4" fill="#4f7cff"/>' +
      '<circle cx="8" cy="8" r="3" fill="#fff"/></svg>',
  );

const meta = {
  args: {
    fit: null,
    iconSrc: null,
    pending: null,
    pulse: 0,
    region: region({ gridCols: 2 }),
  },
  component: GlidePreview,
  decorators: [
    (Story, context) =>
      context.parameters.productionStage === false ? (
        <Story />
      ) : (
        <FeatureStoryStage height={32} viewMode={context.viewMode} width={48}>
          <Story />
        </FeatureStoryStage>
      ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Glide/Preview",
} satisfies Meta<typeof GlidePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const LeftHalf: Story = {};

export const RightTwoThirds: Story = {
  args: { region: region({ colSpan: 2, colStart: 1, gridCols: 3 }) },
};

export const MiddleThird: Story = {
  args: { region: region({ colStart: 1, gridCols: 3 }) },
};

export const TopRightQuarter: Story = {
  args: { region: region({ colStart: 1, gridCols: 2, rowSpan: 1 }) },
};

export const BottomHalf: Story = {
  args: {
    region: region({ colSpan: 2, gridCols: 2, rowSpan: 1, rowStart: 1 }),
  },
};

export const Fill: Story = {
  args: { region: region({ colSpan: 2, gridCols: 2 }) },
};

/**
 * An app that refuses to widen past its own limit: the unmet part of the right
 * half stays neutral, while the extent it reached remains primary.
 */
export const ConstrainedRightHalf: Story = {
  args: {
    fit: {
      actual: { height: 1, width: 0.3, x: 0.7, y: 0 },
      fits: false,
    },
    region: region({ colStart: 1, gridCols: 2 }),
  },
};

/**
 * A window wider than the requested right half. Its overflow stays neutral;
 * only the area that overlaps the requested destination remains primary.
 */
export const ConstrainedRightHalfOverflow: Story = {
  args: {
    fit: {
      actual: { height: 1, width: 0.7, x: 0.3, y: 0 },
      fits: false,
    },
    region: region({ colStart: 1, gridCols: 2 }),
  },
};

export const Minimize: Story = {
  args: { pending: "minimize", region: null },
};

/** Re-armed from the bottom row: the hint wins, the row waits underneath. */
export const MinimizeOverBottomRow: Story = {
  args: {
    pending: "minimize",
    region: region({ colSpan: 2, gridCols: 2, rowSpan: 1, rowStart: 1 }),
  },
};

/** The glided app named in the middle of its own destination. */
export const WithAppIcon: Story = {
  args: { iconSrc: sampleIcon },
};

function GesturePlayground() {
  const activeRef = useRef(false);
  const detectorRef = useRef(new GlideDetector());
  const lastPointRef = useRef({ x: 0, y: 0 });
  const restTimerRef = useRef<ReturnType<typeof setTimeout> | undefined>(
    undefined,
  );
  const [active, setActive] = useState(false);
  const [anchor, setAnchor] = useState({ x: 0, y: 0 });
  const [current, setCurrent] = useState<GlideRegion | null>(null);
  const [pending, setPending] = useState<GlideAction | null>(null);
  const [pulse, setPulse] = useState(0);

  // The playground's tick is the visual one: the same rest timer the real
  // window runs, minus the haptic it has no trackpad to deliver.
  const scheduleRest = () => {
    clearTimeout(restTimerRef.current);
    restTimerRef.current = setTimeout(() => {
      if (detectorRef.current.settle(performance.now()).becameReady) {
        setPulse((count) => count + 1);
      }
    }, detectorRef.current.restRemaining(performance.now()));
  };

  return (
    <div
      className={`relative h-96 w-[640px] overflow-hidden rounded-xl border border-muted/25 bg-neutral-subtle ${active ? "cursor-none" : "cursor-crosshair"}`}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        clearTimeout(restTimerRef.current);
        detectorRef.current.reset();
        setCurrent(null);
        setPending(null);
        setAnchor({
          x: event.nativeEvent.offsetX,
          y: event.nativeEvent.offsetY,
        });
        lastPointRef.current = { x: event.clientX, y: event.clientY };
        activeRef.current = true;
        setActive(true);
      }}
      onPointerMove={(event) => {
        if (!activeRef.current) return;
        const previous = lastPointRef.current;
        lastPointRef.current = { x: event.clientX, y: event.clientY };
        const detection = detectorRef.current.update({
          deltaX: event.clientX - previous.x,
          deltaY: event.clientY - previous.y,
          // Shift is the thirds modifier, as it is in the real gesture.
          thirds: event.shiftKey,
          timestamp: event.timeStamp,
        });
        if (detection.becameReady) setPulse((count) => count + 1);
        if (detection.phase === "settling") scheduleRest();
        if (!detection.changed) return;
        setCurrent(detection.region);
        setPending(detection.pending);
      }}
      onPointerUp={() => {
        clearTimeout(restTimerRef.current);
        activeRef.current = false;
        setActive(false);
      }}
    >
      <div className="pointer-events-none absolute inset-x-0 top-8 text-center text-sm text-muted">
        Drag sideways then fold up or down, swipe up to fill, down to minimize,
        hold Shift for thirds - one move per pause
      </div>
      {active ? (
        <div
          className="pointer-events-none absolute h-8 w-12 -translate-x-1/2 -translate-y-1/2"
          style={{ left: anchor.x, top: anchor.y }}
        >
          <GlidePreview
            fit={null}
            iconSrc={null}
            pending={pending}
            pulse={pulse}
            region={current}
          />
        </div>
      ) : null}
      <div className="pointer-events-none absolute inset-x-0 bottom-8 text-center font-mono text-xs text-muted">
        {pending ??
          (current ? describeRegion(current) : "Press and drag anywhere")}
      </div>
    </div>
  );
}

export const FeelTest: Story = {
  parameters: { layout: "centered", productionStage: false },
  render: () => <GesturePlayground />,
};
