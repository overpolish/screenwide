// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard, Mic, Monitor } from "lucide-react";
import { useState } from "react";

import { TimelineTrackHeader } from "./timeline-track-header";

import type { Meta, StoryObj } from "@storybook/react-vite";

/**
 * The four row kinds side by side, in the column width they take in the
 * band: a selected video track, an audio track that has been switched out of
 * the export, the last remaining track, whose switch cannot be turned off,
 * and the shortcuts lane, which has nothing to include.
 */
function TrackHeaders({ isSelected = true }: { isSelected?: boolean }) {
  const [isScreenIncluded, setIsScreenIncluded] = useState(true);
  const [isMicrophoneIncluded, setIsMicrophoneIncluded] = useState(false);

  return (
    <div className="flex flex-col gap-control">
      <TimelineTrackHeader
        icon={<Monitor />}
        inclusion={{
          isIncluded: isScreenIncluded,
          isRequired: false,
          onChange: setIsScreenIncluded,
        }}
        isSelected={isSelected}
        label="Screen"
        onSelect={() => undefined}
      />
      <TimelineTrackHeader
        icon={<Mic />}
        inclusion={{
          isIncluded: isMicrophoneIncluded,
          isRequired: false,
          onChange: setIsMicrophoneIncluded,
        }}
        label="Microphone"
        onSelect={() => undefined}
      />
      <TimelineTrackHeader
        icon={<Mic />}
        inclusion={{
          isIncluded: true,
          isRequired: true,
          onChange: () => undefined,
        }}
        label="System audio"
        onSelect={() => undefined}
      />
      <TimelineTrackHeader icon={<Keyboard />} label="Shortcuts" />
    </div>
  );
}

const meta = {
  component: TrackHeaders,
  parameters: { layout: "centered" },
  title: "Features/Editor/Timeline Header",
} satisfies Meta<typeof TrackHeaders>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The selected row takes the accent tint; an excluded one dims. */
export const Default: Story = {};

/** Nothing selected, so every row rests on the band. */
export const Unselected: Story = {
  args: { isSelected: false },
};
