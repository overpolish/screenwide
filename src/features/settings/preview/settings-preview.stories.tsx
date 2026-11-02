// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../storybook/feature-story-stage";

import { SettingsPreview } from "./settings-preview";

import type { Meta, StoryObj } from "@storybook/react-vite";

// Storybook has no app to draw the scene, so each story shows the desktop
// the scene is drawn on, as the box does before the app's first frame. Run
// it with `pnpm dev:storybook-native` to see the scene itself.

const meta = {
  args: { isOn: true, scene: { kind: "ruler" } },
  component: SettingsPreview,
  decorators: [featureStage({ width: 560 })],
  title: "Features/Settings/Preview",
} satisfies Meta<typeof SettingsPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Ruler: Story = {};

export const TextRecognition: Story = {
  args: { scene: { kind: "textRecognition" } },
};

export const Annotate: Story = {
  args: {
    scene: {
      color: "#ff6b35",
      counterAngle: 0,
      counterSize: 28,
      keep: true,
      kind: "annotate",
    },
  },
};

export const AutoZoom: Story = {
  args: { scene: { kind: "autoZoom", on: true } },
};

export const FrameRate: Story = {
  args: { scene: { fps: 30, kind: "frameRate" } },
};

export const IncludeScreenwide: Story = {
  args: { scene: { included: true, kind: "includeScreenwide" } },
};

export const Off: Story = {
  args: { isOn: false },
};
