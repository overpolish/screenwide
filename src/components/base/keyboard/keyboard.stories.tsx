// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { TooltipTrigger } from "react-aria-components";

import { Button } from "../button/button";
import { Tooltip } from "../tooltip/tooltip";

import { Keyboard, Shortcut } from "./keyboard";

const shortcutSeparator = "+" as const;

const meta = {
  component: Keyboard,
  parameters: {
    controls: { exclude: ["children", "ref"] },
    layout: "centered",
  },
  title: "Primitives/Keyboard",
} satisfies Meta<typeof Keyboard>;

export default meta;
type Story = StoryObj<typeof meta>;

/* --------------------------------- Stories -------------------------------- */
export const Default: Story = {
  args: {
    children: <>Esc</>,
  },
};

export const ShortcutKeys: Story = {
  name: "Shortcut",
  parameters: { controls: { disable: true } },
  render: () => (
    <Shortcut>
      <Keyboard>Command</Keyboard>
      {shortcutSeparator}
      <Keyboard>Shift</Keyboard>
      {shortcutSeparator}
      <Keyboard>4</Keyboard>
    </Shortcut>
  ),
};

export const InverseSurface: Story = {
  name: "Inverse Surface",
  parameters: {
    controls: { disable: true },
    docs: { story: { height: "140px", inline: false } },
  },
  render: () => (
    <TooltipTrigger isOpen>
      <Button>Blade tool</Button>
      <Tooltip placement="bottom">
        <span className="gap-control-inset flex items-center">
          Blade
          <Shortcut>
            <Keyboard>B</Keyboard>
          </Shortcut>
        </span>
      </Tooltip>
    </TooltipTrigger>
  ),
};

export const Modifiers: Story = {
  parameters: { controls: { disable: true } },
  render: () => (
    <div className="gap-section flex items-center">
      <Keyboard>Command</Keyboard>
      <Keyboard>Control</Keyboard>
      <Keyboard>Shift</Keyboard>
      <Keyboard>Meta</Keyboard>
    </div>
  ),
};
