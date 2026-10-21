// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingDock } from "./recording-dock";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: RecordingDock,
  parameters: { layout: "centered" },
  title: "Explorations/Recording",
} satisfies Meta<typeof RecordingDock>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

export const Paused: Story = { args: { isPaused: true } };
