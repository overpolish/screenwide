// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { AudioMeter } from "./audio-meter";

import type { Meta, StoryObj } from "@storybook/react-vite";

const orientations: ComponentProps<typeof AudioMeter>["orientation"][] = [
  "horizontal",
  "vertical",
];

const meta = {
  argTypes: {
    decibels: {
      control: { max: 2, min: -65, step: 0.01, type: "range" },
    },
    orientation: {
      control: "inline-radio",
      options: orientations,
      table: { defaultValue: { summary: "horizontal" } },
    },
    peak: {
      control: { max: 2, min: -65, step: 0.01, type: "range" },
    },
    thickness: {
      control: { max: 24, min: 2, step: 1, type: "range" },
    },
  },
  args: {
    "aria-label": "Level",
    decibels: -10,
    orientation: "horizontal",
    peak: -6,
    thickness: 10,
  },
  component: AudioMeter,
  parameters: {
    controls: { exclude: ["length"] },
    layout: "centered",
  },
  // The meter fills its length, so it gets a box that is long in that
  // direction only.
  render: (args) => (
    <div className={args.orientation === "vertical" ? "h-40" : "w-64"}>
      <AudioMeter {...args} />
    </div>
  ),
  title: "Primitives/Audio Meter",
} satisfies Meta<typeof AudioMeter>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = {
  args: { disabled: true },
};

export const Orientations: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex h-40 items-center gap-section">
      <div className="w-64">
        <AudioMeter {...args} orientation="horizontal" />
      </div>
      {/* Stretched, so the vertical meter has a height to fill. */}
      <div className="self-stretch">
        <AudioMeter {...args} orientation="vertical" />
      </div>
    </div>
  ),
};
