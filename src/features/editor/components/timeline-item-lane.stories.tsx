// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "lucide-react";
import { useState } from "react";

import {
  createRecordingTimelineEdit,
  cutRecordingTimeline,
} from "../recording-timeline-edit";

import { TimelineItemLane } from "./timeline-item-lane";
import { selectTimelineItem } from "./timeline-item-selection";
import { fitTimelineViewport } from "./timeline-viewport";

import type { Meta, StoryObj } from "@storybook/react-vite";

const STORY_DURATION_MS = 120_000;

/** One cut at the halfway mark, so an item that spans it renders as a seamed
 * run: two fragments, joined rounding, and the label carried once. */
const SPLIT_EDIT = cutRecordingTimeline(createRecordingTimelineEdit(1), 0.5);

type ShortcutItem = {
  endMs: number;
  id: string;
  label: string;
  startMs: number;
};

const shortcuts: ShortcutItem[] = [
  { endMs: 13_000, id: "copy", label: "⌘ C", startMs: 10_000 },
  { endMs: 33_000, id: "paste", label: "⌘ V", startMs: 30_000 },
  // Straddles the cut at 0.5, so it lays out as a seamed run.
  { endMs: 66_000, id: "screenshot", label: "⇧ ⌘ 4", startMs: 56_000 },
  { endMs: 100_000, id: "save", label: "⌘ S", startMs: 96_000 },
];

/** Overlaps `save`, so the lane grows to two sublane rows. */
const overlapping: ShortcutItem[] = [
  ...shortcuts,
  { endMs: 104_000, id: "quit", label: "⌘ Q", startMs: 98_000 },
];

function ItemLanePreview({
  items = shortcuts,
  selectedIds,
  warningIds,
}: {
  items?: ShortcutItem[];
  selectedIds?: string[];
  warningIds?: string[];
}) {
  const [selected, setSelected] = useState(
    () => new Set(selectedIds ?? new Set<string>()),
  );
  return (
    <div className="w-[760px]">
      <TimelineItemLane
        edit={SPLIT_EDIT}
        icon={<Keyboard className="size-icon-small" />}
        items={items}
        label="Shortcuts"
        minimumItemWidthPx={48}
        onClearSelection={() => {
          setSelected(new Set());
        }}
        onSelect={(fragment, _outputPosition, toggle) => {
          setSelected((current) =>
            selectTimelineItem(current, fragment.fragmentId, toggle),
          );
        }}
        selectedFragmentIds={selected}
        sourceDurationMs={STORY_DURATION_MS}
        viewport={fitTimelineViewport()}
        warningFragmentIds={new Set(warningIds ?? [])}
      />
    </div>
  );
}

const meta = {
  component: ItemLanePreview,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline Item Lane",
} satisfies Meta<typeof ItemLanePreview>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The resting lane: badges on a neutral fill, and the run across the cut
 * carrying one label. */
export const Default: Story = {};

/** A picked badge takes the accent the app selects with, under accent-
 * contrast text. */
export const Selected: Story = {
  args: { selectedIds: ["paste:0"] },
};

/** A shortcut whose timing the edit moved keeps the warning colour, selected
 * or not. */
export const Adjusted: Story = {
  args: { selectedIds: ["save:1"], warningIds: ["save:1", "copy:0"] },
};

/** Simultaneous items stack into sublanes and the lane grows to fit. */
export const Stacked: Story = {
  args: { items: overlapping },
};
