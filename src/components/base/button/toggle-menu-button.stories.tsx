// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react";
import { Lock, Mic, MicOff, TriangleAlert } from "lucide-react";
import { useState } from "react";

import { Text } from "../text/text";

import { ToggleMenuButton } from "./toggle-menu-button";

const meta = {
  argTypes: {
    isMenuDisabled: { control: "boolean" },
    isSelected: { control: "boolean" },
    isToggleDisabled: { control: "boolean" },
    size: { control: "inline-radio", options: ["regular", "capture"] },
    variant: { control: "inline-radio", options: ["filled", "ghost"] },
  },
  args: {
    "aria-label": "Microphone",
    children: <Mic />,
    label: "MacBook Pro Microphone",
    menuLabel: "Choose microphone",
    off: <MicOff />,
    onMenuPress: () => undefined,
  },
  component: ToggleMenuButton,
  parameters: {
    controls: { exclude: ["ref", "className"] },
    // Padded rather than centered: centering lands the control on a
    // fractional pixel, which makes the bezel snap differently.
    layout: "padded",
  },
  title: "Primitives/Toggle Menu Button",
} satisfies Meta<typeof ToggleMenuButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { isSelected: true },
};

export const Off: Story = {};

/** The toggle crossfades between the two glyphs, as an icon toggle does. */
export const Interactive: Story = {
  parameters: { controls: { disable: true } },
  render: function Interactive(args) {
    const [isSelected, setIsSelected] = useState(true);
    return (
      <ToggleMenuButton
        {...args}
        isSelected={isSelected}
        onChange={setIsSelected}
      />
    );
  },
};

/** A badge sits over the toggle's corner: a missing device, or no permission. */
export const Badges: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex flex-col gap-section">
      <div className="flex gap-section items-center">
        <Text className="w-16" variant="footnote">
          warning
        </Text>
        <ToggleMenuButton
          {...args}
          badge={<TriangleAlert className="text-warning" />}
          isSelected
        />
      </div>
      <div className="flex gap-section items-center">
        <Text className="w-16" variant="footnote">
          locked
        </Text>
        <ToggleMenuButton
          {...args}
          badge={<Lock className="text-muted" />}
          label="No Microphone"
        />
      </div>
    </div>
  ),
};

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex flex-col gap-section">
      <div className="flex gap-section items-center">
        <Text className="w-16" variant="footnote">
          toggle off
        </Text>
        <ToggleMenuButton {...args} isSelected isToggleDisabled />
      </div>
      <div className="flex gap-section items-center">
        <Text className="w-16" variant="footnote">
          menu off
        </Text>
        <ToggleMenuButton {...args} isMenuDisabled isSelected />
      </div>
      <div className="flex gap-section items-center">
        <Text className="w-16" variant="footnote">
          both off
        </Text>
        <ToggleMenuButton {...args} isMenuDisabled isToggleDisabled />
      </div>
    </div>
  ),
};

/** The ghost variant carries no bezel in either state; in the recording bar
 * the on glyph is a live meter or preview, which says which inputs are on. */
export const Ghost: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex flex-col gap-section">
      {(["regular", "capture"] as const).map((size) =>
        ([true, false] as const).map((isSelected) => (
          <div
            className="flex gap-section items-center"
            key={`${size}-${isSelected ? "on" : "off"}`}
          >
            <Text className="w-24" variant="footnote">
              {`${size} ${isSelected ? "on" : "off"}`}
            </Text>
            <ToggleMenuButton
              {...args}
              isSelected={isSelected}
              size={size}
              variant="ghost"
            />
          </div>
        )),
      )}
    </div>
  ),
};

/** The regular size, beside the taller one the recording bar carries. */
export const Sizes: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex flex-col gap-section">
      {(["regular", "capture"] as const).map((size) =>
        ([true, false] as const).map((isSelected) => (
          <div
            className="flex gap-section items-center"
            key={`${size}-${isSelected ? "on" : "off"}`}
          >
            <Text className="w-24" variant="footnote">
              {`${size} ${isSelected ? "on" : "off"}`}
            </Text>
            <ToggleMenuButton
              {...args}
              isSelected={isSelected}
              size={size}
              tooltip={args.label}
            />
          </div>
        )),
      )}
    </div>
  ),
};

/** The label is capped rather than fixed, so a short device name gives a
 * shorter control and a long one is truncated. */
export const LabelWidths: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <div className="flex flex-col gap-section items-start">
      {["Mic", "External Microphone", "Scarlett 2i2 USB Audio Interface"].map(
        (label) => (
          <ToggleMenuButton
            key={label}
            {...args}
            isSelected
            label={label}
            size="capture"
            tooltip={label}
          />
        ),
      )}
    </div>
  ),
};

/** A detail sits under the label at the label's width: in the recording bar
 * it is the level meter of an audio input that is on. */
export const WithDetail: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <ToggleMenuButton
      {...args}
      detail={
        <span className="h-[3px] w-full overflow-hidden rounded-full bg-fill">
          <span className="block h-full w-2/3 rounded-full bg-primary-surface" />
        </span>
      }
      isSelected
      label="MacBook Pro Microphone"
      size="capture"
      variant="ghost"
    />
  ),
};
