// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../storybook/feature-story-stage";

import { permissionsPreviewSnapshot } from "./permissions-preview";
import { PermissionsWindow } from "./permissions-window";
import { PermissionSnapshot } from "./types";

import type { Meta, StoryObj } from "@storybook/react-vite";

const firstRun: PermissionSnapshot = {
  accessibility: { canRequest: true, granted: false },
  camera: { canRequest: true, granted: false },
  microphone: { canRequest: true, granted: false },
  screenRecording: { canRequest: true, granted: false },
};

const allGranted: PermissionSnapshot = {
  accessibility: { canRequest: false, granted: true },
  camera: { canRequest: false, granted: true },
  microphone: { canRequest: false, granted: true },
  screenRecording: { canRequest: false, granted: true },
};

const meta = {
  args: {
    onClose: fn(),
    onGrant: fn(),
    onRestart: fn(),
    // Required granted, Camera still askable, Microphone only in Settings:
    // every kind of row at once, with Restart offered.
    permissions: permissionsPreviewSnapshot,
  },
  component: PermissionsWindow,
  // Built in Rust at 480pt wide, its height fitted to the list.
  decorators: [featureStage({ width: 480 })],
  parameters: {
    controls: {
      exclude: ["onClose", "onGrant", "onRestart", "permissions"],
    },
    layout: "fullscreen",
  },
  title: "Features/Permissions/Permissions Window",
} satisfies Meta<typeof PermissionsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const FirstRun: Story = {
  args: { permissions: firstRun },
};

export const AllGranted: Story = {
  args: { permissions: allGranted },
};
