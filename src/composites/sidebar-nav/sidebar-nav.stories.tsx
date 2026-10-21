// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Clock,
  Folder,
  Info,
  Keyboard,
  LayoutGrid,
  Settings,
  Trash2,
} from "lucide-react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { SidebarNav } from "./sidebar-nav";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    isExpanded: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    "aria-label": "Settings sections",
    className: "h-72",
    isDisabled: false,
    isExpanded: true,
    items: [
      { icon: <Settings />, id: "general", label: "General" },
      { icon: <LayoutGrid />, id: "glide", label: "Glide" },
      { icon: <Keyboard />, id: "hotkeys", label: "Hotkeys" },
      { icon: <Info />, id: "about", label: "About" },
    ],
    onExpandedChange: fn(),
    onItemContextMenu: fn(),
    onSelectionChange: fn(),
    selected: "general",
  },
  component: SidebarNav,
  // A frame as wide as the expanded sidebar, so collapsing it keeps it at the
  // start, as in a window, rather than re-centring it in the canvas.
  decorators: [
    (Story) => (
      <div className="flex w-48">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: { include: ["isDisabled", "isExpanded"] },
    layout: "centered",
  },
  // Choosing a row and toggling the width keep the controls in step.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <SidebarNav
        {...args}
        onExpandedChange={(isExpanded) => {
          args.onExpandedChange?.(isExpanded);
          updateArgs({ isExpanded });
        }}
        onSelectionChange={(selected) => {
          args.onSelectionChange(selected);
          updateArgs({ selected });
        }}
      />
    );
  },
  title: "Composites/Sidebar Nav",
} satisfies Meta<typeof SidebarNav>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// A source list: a row that cannot be chosen, and one kept at the foot.
export const PinnedAndDisabled: Story = {
  args: {
    "aria-label": "Locations",
    isExpandable: false,
    items: [
      { icon: <Clock />, id: "recent", label: "Recent" },
      { icon: <Folder />, id: "movies", label: "Movies" },
      {
        icon: <Folder />,
        id: "drive",
        isDisabled: true,
        label: "Backup drive",
      },
      { icon: <Trash2 />, id: "trash", isPinned: true, label: "Trash" },
    ],
    selected: "recent",
  },
  parameters: { controls: { disable: true } },
};
