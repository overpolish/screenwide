// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";
import { useEffect, useState } from "react";

import { RecordingDock } from "./recording-dock";

/** Ticks a real second at a time so the digit roll can be watched, growing
 * the size on disk by `bytesPerSecond` when it is given. */
function TickingDock({
  bytesPerSecond,
  fromBytes = 0,
  fromMs,
}: {
  fromMs: number;
  bytesPerSecond?: number;
  fromBytes?: number;
}) {
  const [elapsedMs, setElapsedMs] = useState(fromMs);

  useEffect(() => {
    const interval = window.setInterval(() => {
      setElapsedMs((current) => current + 1_000);
    }, 1_000);

    return () => {
      window.clearInterval(interval);
    };
  }, []);

  return (
    <RecordingDock
      elapsedMs={elapsedMs}
      status="recording"
      writtenBytes={
        bytesPerSecond === undefined
          ? undefined
          : fromBytes + ((elapsedMs - fromMs) / 1_000) * bytesPerSecond
      }
    />
  );
}

/** 1080p at 30 fps: the encoder's 6.2 Mbit/s and a stereo audio track. */
const HD_BYTES_PER_SECOND = 800_000;

const meta = {
  component: RecordingDock,
  decorators: [
    // The dock is as wide as its content, so the stage only fixes the pill's
    // production height and lets the width follow the state on show.
    (Story) => (
      <div className="flex h-10 w-max">
        <Story />
      </div>
    ),
  ],
  parameters: {
    layout: "centered",
  },
  title: "Features/Recording Controls/Recording Dock",
} satisfies Meta<typeof RecordingDock>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Idle: Story = {
  args: { status: "idle" },
};

export const Starting: Story = {
  args: { status: "starting" },
};

export const Countdown: Story = {
  args: { countdownSeconds: 3, status: "starting" },
};

export const Recording: Story = {
  args: { elapsedMs: 42_000, status: "recording" },
};

export const RecordingWithConfidenceChecks: Story = {
  args: {
    elapsedMs: 42_000,
    monitor: {
      cameraCanvasRef: { current: null },
      cameraFrameSize: { height: 54, width: 96 },
      hasCamera: true,
      hasCameraFrame: false,
      hasMicrophone: true,
      hasSystemAudio: true,
      microphoneDecibels: -14,
      noteDecibels: -60,
      systemAudioDecibels: -24,
    },
    status: "recording",
  },
};

export const RecordingForHours: Story = {
  args: { elapsedMs: 4_215_000, status: "recording" },
};

/** A Funny moment just placed: its kind holds the timer's place briefly. */
export const MomentPlaced: Story = {
  args: {
    elapsedMs: 42_000,
    moment: {
      color: "#ffcc00",
      key: 1,
      name: "Funny",
      noMicrophone: false,
      noteMs: null,
    },
    status: "recording",
  },
};

/** The key still held past a tap: the note's own timer runs, with the level
 * of what it hears. */
export const RecordingVoiceNote: Story = {
  args: {
    elapsedMs: 42_000,
    moment: {
      color: "#ffcc00",
      key: 1,
      name: "Funny",
      noMicrophone: false,
      noteMs: 4_200,
    },
    monitor: {
      cameraCanvasRef: { current: null },
      cameraFrameSize: null,
      hasCamera: false,
      hasCameraFrame: false,
      hasMicrophone: true,
      hasSystemAudio: false,
      microphoneDecibels: -14,
      noteDecibels: -14,
      systemAudioDecibels: -60,
    },
    status: "recording",
  },
};

/** Held for a note with no microphone to record it: the moment is still
 * placed. */
export const VoiceNoteWithoutMicrophone: Story = {
  args: {
    elapsedMs: 42_000,
    moment: {
      color: "#ffcc00",
      key: 1,
      name: "Funny",
      noMicrophone: true,
      noteMs: null,
    },
    status: "recording",
  },
};

/* ---------------------------- Digit boundaries ---------------------------- */

/** 00:00:59, the tick before the minute rolls over. */
export const BeforeMinuteRollover: Story = {
  args: { elapsedMs: 59_000, status: "recording" },
};

/** 00:01:00, straight after the minute rolled over. */
export const AfterMinuteRollover: Story = {
  args: { elapsedMs: 60_000, status: "recording" },
};

/** 00:59:59, the tick before the hour rolls over. */
export const BeforeHourRollover: Story = {
  args: { elapsedMs: 3_599_000, status: "recording" },
};

/** 01:00:00. The hours field is always rendered, so nothing resizes here. */
export const AfterHourRollover: Story = {
  args: { elapsedMs: 3_600_000, status: "recording" },
};

/** Rolls 00:00:57 through the minute boundary, one second at a time. */
export const RollingOverMinute: Story = {
  render: () => <TickingDock fromMs={57_000} />,
};

/** Rolls 00:59:57 through the hour boundary, one second at a time. */
export const RollingOverHour: Story = {
  render: () => <TickingDock fromMs={3_597_000} />,
};

/** The discard button after its first press, waiting to be confirmed. */
export const DiscardArmed: Story = {
  args: { elapsedMs: 42_000, status: "recording" },
  play: ({ canvasElement }) => {
    canvasElement
      .querySelector<HTMLButtonElement>('[aria-label="Discard recording"]')
      ?.click();
  },
};

export const Paused: Story = {
  args: { elapsedMs: 42_000, status: "paused" },
};

export const Finishing: Story = {
  args: { elapsedMs: 42_000, status: "stopping" },
};

/* ------------------------------ Size on disk ------------------------------ */

/** The size written so far, before the timer. */
export const RecordingWithSize: Story = {
  args: { elapsedMs: 83_000, status: "recording", writtenBytes: 66_400_000 },
};

/** Before the first measurement a spinner holds the size's place. */
export const SizeNotYetMeasured: Story = {
  args: { elapsedMs: 0, status: "recording", writtenBytes: null },
};

export const SizeWithConfidenceChecks: Story = {
  args: { ...RecordingWithConfidenceChecks.args, writtenBytes: 33_600_000 },
};

/** The widest size it shows, with two decimals of a gigabyte. */
export const SizeInGigabytes: Story = {
  args: {
    elapsedMs: 4_215_000,
    status: "recording",
    writtenBytes: 12_340_000_000,
  },
};

export const PausedWithSize: Story = {
  args: { elapsedMs: 83_000, status: "paused", writtenBytes: 66_400_000 },
};

/** Grows from the start at a 1080p recording's rate, through kilobytes into
 * megabytes. */
export const SizeGrowing: Story = {
  render: () => (
    <TickingDock
      bytesPerSecond={HD_BYTES_PER_SECOND}
      fromBytes={0}
      fromMs={0}
    />
  ),
};

/** Grows through 999.9 MB into gigabytes, one second at a time. */
export const SizeRollingIntoGigabytes: Story = {
  render: () => (
    <TickingDock
      bytesPerSecond={HD_BYTES_PER_SECOND}
      fromBytes={996_800_000}
      fromMs={1_246_000}
    />
  ),
};
