// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { IslandFailure } from "./island-types";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Controls/Island/Failures",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

const failure = (id: number, title: string, message: string) =>
  ({ id, message, title }) satisfies IslandFailure;

export const RecordingCouldNotStart: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-start-failed"),
        "The recording produced no frames",
      ),
    ],
  },
};

export const RecordingRanIntoAProblem: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-capture-failed"),
        "The disk is full",
      ),
    ],
    initialPhase: "recording",
  },
};

export const ReplayCouldNotTurnOn: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-replay-start-failed"),
        "The replay buffer did not receive anything to keep",
      ),
    ],
  },
};

export const ReplayCouldNotTurnBackOn: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-replay-restart-failed"),
        "The selected display is no longer connected",
      ),
    ],
  },
};

export const ReplayStoppedWhileRecording: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-replay-stopped"),
        "The camera stopped sending pictures",
      ),
    ],
    initialPhase: "recording",
    replay: { status: "on" },
  },
};

export const ReplayCouldNotBeSaved: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-replay-save-failed"),
        "The projects folder cannot be written to",
      ),
    ],
    replay: { status: "on" },
  },
};

export const SeveralInTurn: Story = {
  args: {
    failures: [
      failure(
        1,
        t("recording-controls-island-capture-failed"),
        "The disk is full",
      ),
      failure(
        2,
        t("recording-controls-island-finish-failed"),
        "The recording could not be written to the projects folder",
      ),
    ],
    microphoneProblem: "missing",
  },
};
