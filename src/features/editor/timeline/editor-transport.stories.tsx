// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";

import { EditorTransport, type EditorTransportProps } from "./editor-transport";
import { createPlayhead } from "./scrub-playhead";

import type { Meta, StoryObj } from "@storybook/react-vite";

const TICK_MS = 100;

const meta = {
  argTypes: {
    durationMs: { control: { min: 0, step: 1000, type: "number" } },
    isCutting: { table: { defaultValue: { summary: "false" } } },
    isPlaying: { table: { defaultValue: { summary: "false" } } },
    isRanging: { table: { defaultValue: { summary: "false" } } },
  },
  args: {
    durationMs: 750_000,
    isCutting: false,
    isGifRange: false,
    isPlaying: false,
    isRanging: false,
    onCuttingChange: fn(),
    onPlayingChange: fn(),
    onRangingChange: fn(),
    playhead: createPlayhead(),
  },
  component: EditorTransport,
  decorators: [featureStage({ height: 320, width: 120 })],
  parameters: {
    controls: {
      include: ["durationMs", "isCutting", "isPlaying", "isRanging"],
    },
  },
  // Playing runs a clock that moves the playhead, as the preview would, and
  // stops at the end; the toggles land in the args.
  render: function Render(args) {
    const [, updateArgs] = useArgs<EditorTransportProps>();
    const { durationMs, isPlaying, playhead } = args;
    useEffect(() => {
      if (!isPlaying) return;
      let seconds = 0;
      const unsubscribe = playhead.subscribe((now) => {
        seconds = now;
      });
      const timer = setInterval(() => {
        const next = Math.min(seconds + TICK_MS / 1000, durationMs / 1000);
        playhead.publish(next, durationMs > 0 ? next / (durationMs / 1000) : 0);
        if (next * 1000 >= durationMs) updateArgs({ isPlaying: false });
      }, TICK_MS);
      return () => {
        clearInterval(timer);
        unsubscribe();
      };
    }, [durationMs, isPlaying, playhead, updateArgs]);
    return (
      <EditorTransport
        {...args}
        onCuttingChange={(isCutting) => {
          args.onCuttingChange(isCutting);
          updateArgs({ isCutting, isRanging: false });
        }}
        onPlayingChange={(isPlaying) => {
          args.onPlayingChange(isPlaying);
          updateArgs({ isPlaying });
        }}
        onRangingChange={(isRanging) => {
          args.onRangingChange(isRanging);
          updateArgs({ isCutting: false, isRanging });
        }}
      />
    );
  },
  title: "Features/Editor/Timeline/Transport",
} satisfies Meta<typeof EditorTransport>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// The Export island making a GIF: the range tool stays in hand over the
// locked transport.
export const MakingGif: Story = {
  args: { isGifRange: true, isLocked: true, isRanging: true },
};
