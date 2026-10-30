// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { recordedTime, timelineScale } from "../timeline-scale";

import { ShortcutLane, type ShortcutLaneProps } from "./shortcut-lane";

import type { Meta, StoryObj } from "@storybook/react-vite";

// A strip 64 seconds long, 640 px wide, so a second is 10 px.
const WIDTH = 640;

/** The lane, keeping what is chosen and deleted, as the editor would;
 * Controls start it afresh with what they set. */
function LiveLane(args: ShortcutLaneProps) {
  const [shortcuts, setShortcuts] = useState(args.shortcuts);
  const [selected, setSelected] = useState(args.selected);
  return (
    <div className="py-2" style={{ width: WIDTH }}>
      <ShortcutLane
        {...args}
        onDelete={(ids) => {
          args.onDelete(ids);
          setShortcuts((current) =>
            current.filter(({ id }) => !ids.includes(id)),
          );
          setSelected([]);
        }}
        onSelect={(ids) => {
          args.onSelect(ids);
          setSelected(ids);
        }}
        selected={selected}
        shortcuts={shortcuts}
      />
    </div>
  );
}

const meta = {
  args: {
    onDelete: fn(),
    onSelect: fn(),
    scale: timelineScale(recordedTime(64)),
    selected: [],
    shortcuts: [
      { end: 6, id: "copy", keys: ["Command", "C"], start: 3 },
      { end: 12, id: "paste", keys: ["Command", "V"], start: 9 },
      { end: 22, id: "shot", keys: ["Command", "Shift", "4"], start: 16 },
      { end: 31, id: "save", keys: ["Command", "S"], start: 29 },
      { end: 46, id: "tab", keys: ["Command", "Tab"], start: 40 },
      { end: 58, id: "undo", keys: ["Command", "Z"], start: 55 },
    ],
    width: WIDTH,
  },
  component: ShortcutLane,
  decorators: [featureStage({ width: WIDTH })],
  parameters: { controls: { include: ["selected", "shortcuts"] } },
  render: (args) => (
    <LiveLane key={JSON.stringify([args.shortcuts, args.selected])} {...args} />
  ),
  title: "Features/Editor/Timeline/Shortcut Lane",
} satisfies Meta<typeof ShortcutLane>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Two chosen together, as Shift-pressing gathers them; Delete removes both.
export const SeveralChosen: Story = { args: { selected: ["copy", "shot"] } };

// Pressed in quick succession: each tick where it was pressed, the keys set
// side by side rather than over each other.
export const Crowded: Story = {
  args: {
    shortcuts: [
      { end: 11.5, id: "all", keys: ["Command", "A"], start: 10 },
      { end: 12, id: "copy", keys: ["Command", "C"], start: 10.5 },
      { end: 12.5, id: "tab", keys: ["Command", "Tab"], start: 11 },
      { end: 13, id: "paste", keys: ["Command", "V"], start: 11.6 },
      { end: 31, id: "save", keys: ["Command", "S"], start: 29 },
    ],
  },
};
