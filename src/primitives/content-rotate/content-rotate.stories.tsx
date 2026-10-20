// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ContentRotate } from "./content-rotate";

import type { Meta, StoryObj } from "@storybook/react-vite";

const counts = ["3", "2", "1"];

const meta = {
  argTypes: {
    contentKey: {
      control: "inline-radio",
      options: counts,
      table: { defaultValue: { summary: "3" } },
    },
  },
  args: { children: null, contentKey: "3" },
  component: ContentRotate,
  parameters: {
    controls: { include: ["contentKey"] },
    layout: "centered",
  },
  // The shown count follows the key, so picking another value plays the turn.
  render: ({ contentKey }) => (
    <ContentRotate
      className="text-title text-fg tabular-nums"
      contentKey={contentKey}
    >
      {contentKey}
    </ContentRotate>
  ),
  title: "Primitives/Content Rotate",
} satisfies Meta<typeof ContentRotate>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
