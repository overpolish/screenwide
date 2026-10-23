// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createElement } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { StoryNotch } from "../../../storybook/story-notch";

import { PREVIEW_NOTCH } from "./island-preview";
import { IslandStage } from "./island-stage";

import type { Meta } from "@storybook/react-vite";

// What every island story shares: the stage, its events, and the notch from
// the toolbar, so one switch shows every story with and without it. Each
// story sets the state it shows; Controls offer the inputs' levels, to try
// the meters at any level, and the countdown, confidence checks and size
// settings, to try the island each way. The native preview has no toolbar
// and lays out for the notch.

const level = {
  control: { max: 1, min: 0, step: 0.01, type: "range" },
} as const;

const setting = (isOn: boolean) =>
  ({
    control: "boolean",
    table: { defaultValue: { summary: String(isOn) } },
  }) as const;

export const islandMeta = {
  argTypes: {
    hasConfidenceChecks: setting(false),
    hasCountdown: setting(true),
    microphoneLevel: { ...level, table: { defaultValue: { summary: "0.6" } } },
    showsSize: setting(false),
    systemAudioLevel: {
      ...level,
      table: { defaultValue: { summary: "0.35" } },
    },
  },
  args: {
    hasConfidenceChecks: false,
    hasCountdown: true,
    initialMode: "screen",
    initialPhase: "idle",
    isScreenLocked: false,
    microphoneLevel: 0.6,
    onCancelWindowPick: fn(),
    onChooseWindow: fn(),
    onOpenPermissions: fn(),
    onScreenshot: fn(),
    showsSize: false,
    systemAudioLevel: 0.35,
  },
  component: IslandStage,
  decorators: [
    // The notch is the screen's own in a real window; here it is drawn over
    // the island, as a Mac with one shows it.
    (Story, { args, globals }) => {
      const notch = globals.notch === "off" ? null : PREVIEW_NOTCH;
      return createElement(
        "div",
        { className: "relative size-full" },
        Story({ args: { ...args, notch } }),
        notch ? createElement(StoryNotch, { notch }) : null,
      );
    },
    featureStage({ height: 520, width: 720 }),
  ],
  parameters: {
    controls: {
      include: [
        "hasConfidenceChecks",
        "hasCountdown",
        "microphoneLevel",
        "showsSize",
        "systemAudioLevel",
      ],
    },
    layout: "fullscreen",
    notch: true,
  },
} satisfies Meta<typeof IslandStage>;
