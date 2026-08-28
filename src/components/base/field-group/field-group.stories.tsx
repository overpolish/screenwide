// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Mic, Scan, ZapOff } from "lucide-react";

import { AudioMeter } from "../../../features/audio-inputs/components/audio-meter";
import { IconToggleButton } from "../button/icon-button";
import { ListBoxItem } from "../listbox-item/listbox-item";
import { Select } from "../select/select";

import { FieldGroup, FieldGroupAction, FieldGroupFooter } from "./field-group";

const meta = {
  component: FieldGroup,
  parameters: {
    layout: "centered",
  },
  title: "Primitives/Field Group",
} satisfies Meta<typeof FieldGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

export const TrailingAction: Story = {
  render: () => (
    <FieldGroup className="flex w-52 items-center">
      <div className="min-w-0 flex-1">
        <Select
          aria-label="Camera resolution"
          className="w-full"
          clearable={false}
          defaultValue="1080"
          leftSection={<Scan size={14} />}
          size="compact"
        >
          <ListBoxItem id="1080">1920 × 1080</ListBoxItem>
          <ListBoxItem id="720">1280 × 720</ListBoxItem>
        </Select>
      </div>
      <FieldGroupAction>
        <IconToggleButton
          aria-label="Anti-flicker"
          defaultSelected
          size="compact"
        >
          <ZapOff className="transform-gpu" size={14} />
        </IconToggleButton>
      </FieldGroupAction>
    </FieldGroup>
  ),
};

export const DefaultTrailingAction: Story = {
  render: () => (
    <FieldGroup className="flex w-52 items-center">
      <div className="min-w-0 flex-1">
        <Select
          aria-label="Camera resolution"
          className="w-full"
          clearable={false}
          defaultValue="1080"
          leftSection={<Scan size={16} />}
        >
          <ListBoxItem id="1080">1920 × 1080</ListBoxItem>
          <ListBoxItem id="720">1280 × 720</ListBoxItem>
        </Select>
      </div>
      <FieldGroupAction>
        <IconToggleButton aria-label="Anti-flicker" defaultSelected>
          <ZapOff className="transform-gpu" size={14} />
        </IconToggleButton>
      </FieldGroupAction>
    </FieldGroup>
  ),
};

export const SupportingMeter: Story = {
  render: () => (
    <FieldGroup className="gap-control flex w-52 flex-col">
      <Select
        aria-label="Microphone"
        className="w-full"
        clearable={false}
        defaultValue="built-in"
        leftSection={<Mic size={14} />}
        size="compact"
      >
        <ListBoxItem id="built-in">Built-in Microphone</ListBoxItem>
        <ListBoxItem id="usb">USB Microphone</ListBoxItem>
      </Select>
      <FieldGroupFooter>
        <AudioMeter
          decibels={-18}
          height={5}
          hidePeakTick
          hideTicks
          width="100%"
        />
      </FieldGroupFooter>
    </FieldGroup>
  ),
};
