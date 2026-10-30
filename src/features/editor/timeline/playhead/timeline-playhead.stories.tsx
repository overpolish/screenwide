// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { useRestingAction } from "../../../../storybook/use-resting-action";
import { createPlayhead } from "../scrub-playhead";
import { STORY_TRACK_PX } from "../timeline-story-fixtures";

import { TimelinePlayhead } from "./timeline-playhead";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** A strip's height of room under the knob, the knob dragged along it and
 * the drag logged to Actions once it rests. */
function StoryPlayhead({
  at,
  onScrub,
}: {
  /** Where the playhead starts, as a share of the recording. */
  at: number;
  onScrub: (event: React.PointerEvent) => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [playhead] = useState(createPlayhead);
  const logScrub = useRestingAction(onScrub);
  useEffect(() => {
    playhead.publish(at, at);
  }, [at, playhead]);
  return (
    <div className="pt-4" style={{ width: STORY_TRACK_PX }}>
      <div className="relative h-24" ref={ref}>
        <TimelinePlayhead
          onScrub={(event) => {
            logScrub(event);
            const box = ref.current?.getBoundingClientRect();
            if (!box) return;
            const share = Math.min(
              Math.max((event.clientX - box.left) / box.width, 0),
              1,
            );
            playhead.publish(share, share);
          }}
          playhead={playhead}
        />
      </div>
    </div>
  );
}

const meta = {
  argTypes: {
    at: { control: { max: 1, min: 0, step: 0.01, type: "range" } },
  },
  args: { at: 0.3, onScrub: fn() },
  component: StoryPlayhead,
  decorators: [featureStage({ width: STORY_TRACK_PX })],
  parameters: { controls: { include: ["at"] } },
  title: "Features/Editor/Timeline/Playhead",
} satisfies Meta<typeof StoryPlayhead>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
