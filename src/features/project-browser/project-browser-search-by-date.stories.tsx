// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { locations, projects } from "./project-browser-fixtures";
import { projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

// Search reads when the projects were recorded, said back above what it
// found. The sample library is dated from today, so each story finds its
// projects whenever it is opened.

const meta = {
  ...projectBrowserMeta,
  args: {
    ...projectBrowserMeta.args,
    projects: projects.filter(({ available }) => available),
    selected: locations[0].path,
  },
  title: "Features/Project Browser/Search/By Date",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Yesterday: Story = { args: { initialQuery: "yesterday" } };

// The podcast was recorded on the latest Monday at ten past three.
const monday = new Date();
monday.setDate(monday.getDate() - ((monday.getDay() + 6) % 7));
monday.setHours(15, 10, 0, 0);

export const MondayAfternoon: Story = {
  args: {
    initialQuery: "monday 3pm",
    projects: meta.args.projects.map((project) =>
      project.title === "Podcast intro"
        ? { ...project, createdMs: monday.getTime() }
        : project,
    ),
  },
};

export const AnHourAgo: Story = {
  args: {
    initialQuery: "an hour ago",
    projects: meta.args.projects.map((project) =>
      project.title === "Product demo"
        ? { ...project, createdMs: Date.now() - 20 * 60 * 1_000 }
        : project,
    ),
  },
};

export const DateAndWords: Story = {
  args: { initialQuery: "yesterday pricing" },
};
