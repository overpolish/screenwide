// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { ActionScript } from "./action-script";
import { FisheyeTimeline } from "./fisheye-timeline";
import { KeyboardMap } from "./keyboard-map";
import { LivingSettings } from "./living-settings";
import { LookPens } from "./look-pens";
import { PointAnnotate } from "./point-annotate";
import { Squeeze } from "./squeeze";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  decorators: [featureStage({ height: 820, width: 1180 })],
  parameters: { layout: "fullscreen" },
  title: "Explorations/Ideas",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

export const LooksAndPens: Story = { render: () => <LookPens /> };

export const PointDontDraw: Story = { render: () => <PointAnnotate /> };

export const SettingsYouCanSee: Story = { render: () => <LivingSettings /> };

export const ShortcutsKeyboardMap: Story = { render: () => <KeyboardMap /> };

export const ActionScriptEditing: Story = { render: () => <ActionScript /> };

export const SqueezeTime: Story = { render: () => <Squeeze /> };

export const FisheyeScrubbing: Story = { render: () => <FisheyeTimeline /> };
