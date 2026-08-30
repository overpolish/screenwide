// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { WindowHeader } from "./window-header";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    description: "This QR code contains an action you can open or copy.",
    onClose: () => undefined,
    title: "Open link",
  },
  component: WindowHeader,
  decorators: [
    (Story) => (
      <div className="window-surface w-[480px] text-content-fg">
        <Story />
      </div>
    ),
  ],
  parameters: { layout: "centered" },
  title: "Components/Window Header",
} satisfies Meta<typeof WindowHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
