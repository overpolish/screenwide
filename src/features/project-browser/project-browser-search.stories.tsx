// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { projectBrowserMeta } from "./project-browser-story-meta";

import type { ProjectBrowser } from "./project-browser";
import type { Meta, StoryObj } from "@storybook/react-vite";

// Search reads inside the recordings: what was said, and the moments flagged
// while recording with what their voice notes say, as well as names.

const meta = {
  ...projectBrowserMeta,
  title: "Features/Project Browser/Search",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Moments: Story = { args: { initialQuery: "pricing" } };

export const ByName: Story = { args: { initialQuery: "launch" } };

export const NoResults: Story = { args: { initialQuery: "zebra" } };
