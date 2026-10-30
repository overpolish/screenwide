// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { projects } from "./project-browser-fixtures";
import { projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  title: "Legacy/Project Browser/Recent",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Grouped: Story = {};

export const MissingProject: Story = {
  args: { projects: [projects[0], projects[5]] },
};

export const Loading: Story = {
  args: { projects: null },
};

export const Empty: Story = {
  args: { projects: [] },
};
