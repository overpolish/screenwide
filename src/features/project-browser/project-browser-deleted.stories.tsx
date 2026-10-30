// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { deletedProjects } from "./project-browser-fixtures";
import { projectBrowserMeta } from "./project-browser-story-meta";
import { DELETED } from "./types";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  args: {
    ...projectBrowserMeta.args,
    projects: deletedProjects,
    selected: DELETED,
  },
  title: "Legacy/Project Browser/Recently Deleted",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Waiting: Story = {};

export const Empty: Story = {
  args: { projects: [] },
};
