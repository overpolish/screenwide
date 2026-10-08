// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SearchField } from "./search-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: { "aria-label": "Search projects", placeholder: "Search" },
  component: SearchField,
  decorators: [
    (Story) => (
      <div className="w-48">
        <Story />
      </div>
    ),
  ],
  parameters: { layout: "centered" },
  title: "Primitives/Search Field",
} satisfies Meta<typeof SearchField>;
export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {};

export const WithQuery: Story = { args: { defaultValue: "demo" } };
