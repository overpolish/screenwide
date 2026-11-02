// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { userEvent, within } from "storybook/test";

import { t } from "../../../i18n/i18n";
import { projects } from "../project-browser-fixtures";
import { projectBrowserMeta } from "../project-browser-story-meta";
import { openFailure } from "../use-open-failures";

import type { ProjectBrowser } from "../project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...projectBrowserMeta,
  title: "Features/Project Browser/Island",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

// Ticks the first cards' boxes.
const choose =
  (count: number): Story["play"] =>
  async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    for (const { title } of projects.slice(0, count)) {
      // By the title alone: the label wraps it in direction isolates.
      await userEvent.click(
        await canvas.findByRole("checkbox", { name: new RegExp(title) }),
      );
    }
  };

export const Selecting: Story = { play: choose(2) };

export const Moving: Story = {
  args: {
    copying: {
      count: 3,
      destination: "Client work",
      fraction: 0.42,
      kind: "move",
      title: projects[0].title,
    },
  },
};

export const Deleted: Story = {
  args: {
    deleted: { count: 2, title: projects[0].title },
    projects: projects.slice(2),
  },
};

export const ActionFailed: Story = {
  args: { error: "That name cannot be used" },
};

// The reasons are the app's own sentences, read while rendering so the
// story shows them in the language chosen.
export const ProjectWouldNotOpen: Story = {
  render: (args) =>
    meta.render({
      ...args,
      error: openFailure(t("project-browser-open-failed-recording-missing")),
    }),
};

export const SeveralWouldNotOpen: Story = {
  render: (args) =>
    meta.render({
      ...args,
      error: [
        openFailure(t("project-browser-open-failed-discarded")),
        openFailure(t("project-browser-open-failed-damaged")),
      ].join("\n"),
    }),
};

export const MovingWhileSelecting: Story = {
  args: Moving.args,
  play: choose(3),
};
