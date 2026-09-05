// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import logoUrl from "../../../assets/screenwide-mark.svg";

import { WindowHeader } from "./window-header";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
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

export const Compact: Story = {
  args: { variant: "compact" },
};

export const Display: Story = {
  args: {
    leadingSection: (
      <img
        alt="Screenwide"
        className="brightness-0 dark:invert"
        draggable={false}
        src={logoUrl}
      />
    ),
    title: "Permissions",
  },
};

export const WindowControls: Story = {
  args: {
    isMaximized: false,
    leadingSection: (
      <img
        alt="Screenwide"
        className="brightness-0 dark:invert"
        draggable={false}
        src={logoUrl}
      />
    ),
    onMinimize: () => undefined,
    onToggleMaximize: () => undefined,
    title: "Settings",
  },
};
