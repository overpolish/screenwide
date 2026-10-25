// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { SettingsApiContext } from "../../../features/settings/settings-api-context";
import { createPreviewApi } from "../../../features/settings/settings-window-preview";
import { featureStage } from "../../feature-story-stage";

import { ExploredSettingsWindow } from "./settings-window";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: ExploredSettingsWindow,
  decorators: [featureStage({ window: "settings" })],
  parameters: {
    controls: { exclude: ["initialSection"] },
    layout: "fullscreen",
  },
  // The Settings stories' in-memory API, so every pane reads and writes the
  // same settings and nothing reaches the app.
  render: function Render(args) {
    const [api] = useState(() => createPreviewApi());
    return (
      <SettingsApiContext value={api}>
        <ExploredSettingsWindow {...args} />
      </SettingsApiContext>
    );
  },
  title: "Explorations/Settings",
} satisfies Meta<typeof ExploredSettingsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const General: Story = { args: { initialSection: "general" } };

export const Recording: Story = { args: { initialSection: "recording" } };

export const Tools: Story = { args: { initialSection: "tools" } };

export const Moments: Story = { args: { initialSection: "moments" } };

export const Models: Story = { args: { initialSection: "models" } };

export const Shortcuts: Story = { args: { initialSection: "shortcuts" } };
