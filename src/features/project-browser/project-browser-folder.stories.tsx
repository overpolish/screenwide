// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { locations, projectsFor } from "./project-browser-fixtures";
import { projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  title: "Legacy/Project Browser/Folder",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ProjectsFolder: Story = {
  args: {
    projects: projectsFor(locations[0].path),
    selected: locations[0].path,
  },
};

export const AddedFolder: Story = {
  args: {
    projects: projectsFor(locations[1].path),
    selected: locations[1].path,
  },
};

export const Empty: Story = {
  args: { projects: [], selected: locations[2].path },
};
