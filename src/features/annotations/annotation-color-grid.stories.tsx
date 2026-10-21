// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import { AnnotationColorGrid } from "./annotation-color-grid";
import { withAnnotationColor, withoutAnnotationColor } from "./palette";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    offersCustom: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    isDisabled: false,
    offersCustom: true,
    onChange: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    value: "#7a4fd1",
  },
  component: AnnotationColorGrid,
  parameters: {
    controls: { include: ["isDisabled", "offersCustom"] },
    layout: "centered",
  },
  // The palette with two colours of your own after it, as the editor's
  // annotation panel shows it. A colour chosen behind Custom shows on its
  // tile; a right click keeps it, and two right clicks on a kept one forget
  // it.
  render: function Render(args) {
    const [color, setColor] = useState(args.value);
    const [saved, setSaved] = useState(["#2ec4b6", "#8b5e34"]);
    return (
      <div className="w-64">
        <AnnotationColorGrid
          {...args}
          onChange={(next) => {
            args.onChange(next);
            setColor(next);
          }}
          onRemoveColor={(removed) => {
            args.onRemoveColor?.(removed);
            setSaved((current) => withoutAnnotationColor(current, removed));
          }}
          onSaveColor={(kept) => {
            args.onSaveColor?.(kept);
            setSaved((current) => withAnnotationColor(current, kept));
          }}
          savedColors={saved}
          value={color}
        />
      </div>
    );
  },
  title: "Features/Annotations/Color Grid",
} satisfies Meta<typeof AnnotationColorGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
