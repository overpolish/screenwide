// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { projects } from "./project-browser-fixtures";
import { goToCard, projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  title: "Features/Project Browser/Recent",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Library: Story = {};

export const MissingProject: Story = {
  args: { projects: [projects[0], projects[5]] },
};

// The window scrolls to the card, focuses it and outlines it for a moment.
export const GoToCard: Story = { play: goToCard(projects[7].title) };

export const LongNames: Story = {
  args: {
    projects: [
      {
        ...projects[1],
        title:
          "Quarterly onboarding walkthrough for the enterprise customers in the northern region",
      },
      {
        ...projects[3],
        title:
          "Settings redesign, second pass, with the new sidebar and its groups",
      },
      projects[2],
    ],
  },
};

export const Loading: Story = {
  args: { projects: null },
};

export const Empty: Story = {
  args: { projects: [] },
};
