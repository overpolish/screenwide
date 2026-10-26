// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { ProjectsWindow } from "./projects-window";

import type { Meta, StoryObj } from "@storybook/react-vite";

// The project browser, browsed by what happened in each project. Move across
// a card to walk its steps; press it, or point and press Space, to peek, and
// Return in the peek to grow it into the editor (Escape comes back). Type in
// the search to find moments rather than files: try "Safari", "pricing",
// "npm test". Open the takes on "Settings redesign" and keep one; point at
// the disk strip in the sidebar, and press the line under it.

const meta = {
  component: ProjectsWindow,
  decorators: [featureStage({ window: "projects" })],
  parameters: { controls: { disable: true }, layout: "fullscreen" },
  title: "Explorations/Projects",
} satisfies Meta<typeof ProjectsWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Library: Story = {};

export const SearchingMoments: Story = { args: { initialQuery: "pricing" } };

export const Peeking: Story = {
  args: { initialPeek: { at: 14, id: "walkthrough" } },
};

export const Takes: Story = { args: { initialTakesOpen: true } };

export const FreeingSpace: Story = { args: { initialReclaiming: true } };
