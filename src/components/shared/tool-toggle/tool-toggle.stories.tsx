// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MoveUpRight, Scissors } from "lucide-react";
import { useState } from "react";

import { ToolToggle } from "./tool-toggle";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: { layout: "centered" },
  title: "Components/Tool Toggle",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

/** A row of tools with one in hand, as the Editor's timeline toolbar and the
 * annotation overlay's toolbar show them. */
function ToolRow() {
  const [chosen, setChosen] = useState("arrow");
  return (
    <div className="flex items-center gap-control">
      <ToolToggle
        isSelected={chosen === "arrow"}
        label="Arrow"
        name="Arrow tool"
        onSelectedChange={() => {
          setChosen("arrow");
        }}
        shortcut="A"
      >
        <MoveUpRight />
      </ToolToggle>
      <ToolToggle
        isSelected={chosen === "blade"}
        label="Blade"
        name="Blade tool"
        onSelectedChange={() => {
          setChosen("blade");
        }}
        shortcut="B"
      >
        <Scissors />
      </ToolToggle>
    </div>
  );
}

export const Default: Story = { render: () => <ToolRow /> };
