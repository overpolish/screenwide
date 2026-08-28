// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { permissionsPreviewSnapshot } from "./permissions-preview";
import { PermissionsWindow } from "./permissions-window";
import { PermissionSnapshot } from "./types";

const applyPreviewTheme = (theme: unknown) => {
  const selectedTheme = theme === "light" ? "light" : "dark";
  document.documentElement.classList.remove("dark", "light");
  document.documentElement.classList.add(selectedTheme);
};

const allGranted: PermissionSnapshot = {
  accessibility: { canRequest: false, granted: true },
  camera: { canRequest: false, granted: true },
  microphone: { canRequest: false, granted: true },
  screenRecording: { canRequest: false, granted: true },
};

const firstRun: PermissionSnapshot = {
  accessibility: { canRequest: true, granted: false },
  camera: { canRequest: true, granted: false },
  microphone: { canRequest: true, granted: false },
  screenRecording: { canRequest: true, granted: false },
};

const meta = {
  args: {
    onClose: () => undefined,
    onGrant: () => undefined,
    onRestart: () => undefined,
    permissions: permissionsPreviewSnapshot,
  },
  component: PermissionsWindow,
  decorators: [
    (Story, context) => {
      applyPreviewTheme(context.globals.theme);
      return (
        <FeatureStoryStage height={388} viewMode={context.viewMode} width={540}>
          <Story />
        </FeatureStoryStage>
      );
    },
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Permissions",
} satisfies Meta<typeof PermissionsWindow>;

export default meta;
type Story = StoryObj<typeof PermissionsWindow>;

export const ReviewStates: Story = {
  name: "Review States",
};

export const FirstRun: Story = {
  args: { permissions: firstRun },
  name: "First Run",
};

export const AllGranted: Story = {
  args: { permissions: allGranted },
  name: "All Granted",
};
