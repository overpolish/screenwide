// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Info, Keyboard, LayoutGrid, Settings } from "lucide-react";
import { useState } from "react";

import { SidebarNav } from "./sidebar-nav";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Settings sections",
    items: [
      { icon: <Settings />, id: "general", label: "General" },
      { icon: <LayoutGrid />, id: "glide", label: "Glide" },
      { icon: <Keyboard />, id: "hotkeys", label: "Hotkeys" },
      { icon: <Info />, id: "about", isDisabled: true, label: "About" },
    ],
    onSelectionChange: () => undefined,
    selected: "general",
  },
  component: SidebarNav,
  decorators: [
    (Story) => (
      <div className="w-52">
        <Story />
      </div>
    ),
  ],
  parameters: {
    controls: {
      exclude: [
        "items",
        "onSelectionChange",
        "className",
        "selected",
        "defaultExpanded",
      ],
    },
    layout: "centered",
  },
  render: function Render(args) {
    const [selected, setSelected] = useState(args.selected);
    return (
      <SidebarNav
        {...args}
        className="h-80"
        onSelectionChange={(id) => {
          setSelected(id);
          args.onSelectionChange(id);
        }}
        selected={selected}
      />
    );
  },
  title: "Primitives/Sidebar Nav",
} satisfies Meta<typeof SidebarNav>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
