// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { projects, scrubStrips, thumbnails } from "../project-browser-fixtures";

import { ProjectCard } from "./project-card";

import type { Meta, StoryObj } from "@storybook/react-vite";

const events = {
  onExtendSelection: fn(),
  onMenu: fn(),
  onOpen: fn(),
  onRename: fn(() => Promise.resolve()),
  onScrubRequest: fn(),
  onToggleSelection: fn(),
};

const meta = {
  args: {
    ...events,
    isSelected: false,
    isSelectionMode: false,
    project: projects[0],
    scrubStrip: scrubStrips[projects[0].file],
    thumbnail: thumbnails[projects[0].file],
  },
  component: ProjectCard,
  parameters: {
    controls: { exclude: [...Object.keys(events), "project", "scrubStrip"] },
    layout: "centered",
  },
  render: (args) => (
    <div className="w-60">
      <ProjectCard {...args} />
    </div>
  ),
  title: "Features/Project Browser/Card",
} satisfies Meta<typeof ProjectCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

export const Selected: Story = {
  args: { isSelected: true, isSelectionMode: true },
};

export const Kinds: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="grid w-[48rem] grid-cols-3 gap-x-3 gap-y-6">
      {[1, 2, 4, 6, 5, 8].map((index) => (
        <ProjectCard
          key={projects[index].file}
          {...args}
          project={projects[index]}
          scrubStrip={scrubStrips[projects[index].file]}
          thumbnail={thumbnails[projects[index].file] ?? null}
        />
      ))}
    </div>
  ),
};
