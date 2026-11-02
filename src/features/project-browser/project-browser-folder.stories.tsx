// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { locations, projects, projectsFor } from "./project-browser-fixtures";
import { goToCard, projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  title: "Features/Project Browser/Folder",
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

// Every project recorded today, so the cards fill the pane's width.
export const FullDay: Story = {
  args: {
    projects: projects.map((project) => ({
      ...project,
      createdMs: projects[0].createdMs,
    })),
    selected: locations[0].path,
  },
};

// Going to a card the search hides clears the search.
export const GoToHiddenCard: Story = {
  args: { ...FullDay.args, initialQuery: projects[0].title },
  play: goToCard(projects[8].title),
};

export const Empty: Story = {
  args: { projects: [], selected: locations[2].path },
};
