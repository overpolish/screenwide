// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { edgeSnap, stripSnapTargets } from "../strip/strip-snap";
import { recordedTime, timelineScale } from "../timeline-scale";

import { AnnotationLane, type AnnotationLaneProps } from "./annotation-lane";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 64 seconds long, 640 px wide, so a second is 10 px.
const WIDTH = 640;
const scale = timelineScale(recordedTime(64));

// Keeps choices, retimes and restacks as the editor would, ends snapping with
// ⌘; Controls start it afresh.
function LiveLane(args: AnnotationLaneProps) {
  const [annotations, setAnnotations] = useState(args.annotations);
  const [selected, setSelected] = useState(args.selected);
  const logRetime = useRestingAction(args.onRetime);
  return (
    // Room for the lanes a drag opens, so the stage does not recentre the
    // lane as it grows.
    <div className="py-2" style={{ minHeight: 160, width: WIDTH }}>
      <AnnotationLane
        {...args}
        annotations={annotations}
        onRestack={(lanes) => {
          args.onRestack(lanes);
          setAnnotations((current) =>
            current.map((annotation) => ({
              ...annotation,
              lane: lanes[annotation.id] ?? annotation.lane,
            })),
          );
        }}
        onRetime={(id, start, end) => {
          logRetime(id, start, end);
          setAnnotations((current) =>
            current.map((annotation) =>
              annotation.id === id ? { ...annotation, end, start } : annotation,
            ),
          );
        }}
        onSelect={(id) => {
          args.onSelect(id);
          setSelected(id);
        }}
        selected={selected}
        snap={(except) =>
          edgeSnap({
            except,
            scale,
            targets: stripSnapTargets({ annotations }),
            width: WIDTH,
          })
        }
      />
    </div>
  );
}

const meta = {
  args: {
    // Each lane holds annotations that never show at once; the top lane is
    // drawn in front.
    annotations: [
      { end: 30, id: "arrow", kind: "arrow", lane: 0, start: 14 },
      { end: 50, id: "highlight", kind: "highlight", lane: 0, start: 46 },
      { end: 32, id: "counter", kind: "counter", lane: 1, start: 28 },
      { end: 62, id: "focus", kind: "focus", lane: 1, start: 40 },
      { end: 24, id: "text", kind: "text", lane: 1, start: 2 },
      { end: 52, id: "redact", kind: "redact", lane: 2, start: 36 },
    ],
    onRestack: fn(),
    onRetime: fn(),
    onSelect: fn(),
    scale,
    width: WIDTH,
  },
  component: AnnotationLane,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["annotations", "selected"] } },
  render: (args) => (
    <LiveLane
      key={JSON.stringify([args.annotations, args.selected])}
      {...args}
    />
  ),
  title: "Features/Editor/Timeline/Annotation Lane",
} satisfies Meta<typeof AnnotationLane>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Selected: Story = { args: { selected: "arrow" } };

// Hatched where the content is out of view, ticked where put right by hand,
// in the warning colour where the tracker is unsure.
export const Pinned: Story = {
  args: {
    annotations: [
      {
        end: 34,
        id: "arrow",
        kind: "arrow",
        lane: 0,
        pin: {
          corrections: [12, 31],
          outOfView: [{ end: 26, start: 19 }],
          unsure: [
            { end: 10, start: 7 },
            { end: 29, start: 26 },
          ],
        },
        start: 6,
      },
      { end: 50, id: "highlight", kind: "highlight", lane: 0, start: 44 },
      {
        end: 62,
        id: "focus",
        kind: "focus",
        lane: 1,
        pin: {
          corrections: [],
          outOfView: [{ end: 62, start: 55 }],
          unsure: [{ end: 47, start: 42 }],
        },
        start: 38,
      },
      {
        end: 30,
        id: "counter",
        kind: "counter",
        lane: 1,
        pin: { corrections: [24], outOfView: [], unsure: [] },
        start: 18,
      },
    ],
  },
};
