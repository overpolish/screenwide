// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { AngleDial } from "./angle-dial";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    "aria-label": "Angle",
    onChange: () => undefined,
    value: 0,
  },
  component: AngleDial,
  parameters: {
    controls: { exclude: ["className", "onChange"] },
    layout: "centered",
  },
  render: function Render(args) {
    const [draft, setDraft] = useState({
      source: args.value,
      value: args.value,
    });
    // Accept edits from Controls while keeping pointer-rate changes local.
    if (draft.source !== args.value) {
      setDraft({ source: args.value, value: args.value });
    }
    return (
      <div className="flex items-center gap-control-inset">
        <AngleDial
          {...args}
          onChange={(value) => {
            args.onChange(value);
            setDraft({ source: args.value, value });
          }}
          value={draft.value}
        />
        <span className="w-10 text-body text-content-fg-secondary tabular-nums">
          {draft.value}°
        </span>
      </div>
    );
  },
  title: "Primitives/Angle Dial",
} satisfies Meta<typeof AngleDial>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Turned: Story = {
  args: { value: 135 },
};

export const Disabled: Story = {
  args: { isDisabled: true, value: 45 },
};
