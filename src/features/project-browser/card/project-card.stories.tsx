// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { PSEUDO } from "../../../i18n/i18n";
import {
  featureStage,
  type StageSize,
} from "../../../storybook/feature-story-stage";
import { readings } from "../project-browser-activity-fixtures";
import {
  daysAgo,
  projects,
  scrubStrips,
  thumbnails,
} from "../project-browser-fixtures";

import { ProjectCard } from "./project-card";

import type { Meta, StoryObj } from "@storybook/react-vite";

const events = {
  onExtendSelection: fn(),
  onHoverChange: fn(),
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
    reading: readings[projects[0].file],
    scrubStrip: scrubStrips[projects[0].file],
    thumbnail: thumbnails[projects[0].file],
  },
  component: ProjectCard,
  // A card at its width in the grid; a story laying cards side by side
  // gives its own `parameters.stage`.
  decorators: [
    featureStage(
      ({ parameters }) =>
        (parameters.stage as StageSize | undefined) ?? { width: 240 },
    ),
  ],
  parameters: {
    controls: {
      exclude: [
        ...Object.keys(events),
        "project",
        "reading",
        "scrubStrip",
        "hits",
      ],
    },
    layout: "fullscreen",
  },
  title: "Features/Project Browser/Card",
} satisfies Meta<typeof ProjectCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

export const Selected: Story = {
  args: { isSelected: true, isSelectionMode: true },
};

export const SearchFinds: Story = {
  args: { hits: [18.4, 60] },
};

export const Kinds: Story = {
  parameters: { controls: { disable: true }, stage: { width: 768 } },
  render: (args) => (
    <div className="grid grid-cols-3 gap-x-separate gap-y-group">
      {[1, 2, 4, 6, 5, 8].map((index) => (
        <ProjectCard
          key={projects[index].file}
          {...args}
          project={projects[index]}
          reading={readings[projects[index].file]}
          scrubStrip={scrubStrips[projects[index].file]}
          thumbnail={thumbnails[projects[index].file] ?? null}
        />
      ))}
    </div>
  ),
};

// Today, this week, this year and a past year: one of each date style.
const recordedAt = () => {
  const now = new Date();
  return [
    daysAgo(0, 9),
    daysAgo(4, 16),
    new Date(now.getFullYear(), 0, 1, 11).getTime(),
    daysAgo(400, 9),
  ];
};

export const RecordedDates: Story = {
  parameters: { controls: { disable: true }, stage: { width: 1024 } },
  render: (args) => (
    <div className="grid grid-cols-4 gap-x-separate">
      {recordedAt().map((createdMs) => (
        <ProjectCard
          key={createdMs}
          {...args}
          project={{ ...args.project, createdMs }}
        />
      ))}
    </div>
  ),
};

export const RecordedDatesPseudoLocale: Story = {
  ...RecordedDates,
  globals: { locale: PSEUDO },
};
