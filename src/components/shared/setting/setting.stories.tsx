// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { PillGroup } from "../../base/pill-group/pill-group";
import { Switch } from "../../base/switch/switch";
import { HotkeyField } from "../hotkey-field/hotkey-field";
import { PathField } from "../path-field/path-field";
import { SliderNumberField } from "../slider-number-field/slider-number-field";

import { Setting } from "./setting";

import type { SettingControlProps } from "./setting";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    children: (controlProps) => <Switch {...controlProps} defaultSelected />,
    description: "Show the containing folder after a successful export.",
    title: "Open location after export",
  },
  component: Setting,
  parameters: { layout: "centered" },
  render: (args) => (
    <div className="w-xl max-w-full">
      <Setting {...args} />
    </div>
  ),
  title: "Components/Setting",
} satisfies Meta<typeof Setting>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Composite controls keep their own action labels. The surrounding group links
// the whole control to the setting title and description.
function GapControl(props: SettingControlProps) {
  const [value, setValue] = useState(12);
  return (
    <div {...props} role="group">
      <SliderNumberField
        aria-label="Window gap"
        className="w-64"
        maxValue={200}
        minValue={0}
        onChange={setValue}
        rightSection="px"
        sliderMaxValue={50}
        value={value}
      />
    </div>
  );
}

function LocationControl(props: SettingControlProps) {
  const [value, setValue] = useState<string | null>(null);
  return (
    <div {...props} role="group">
      <PathField
        aria-label="Recording location"
        emptyLabel="System default"
        onBrowse={() => {
          setValue("/Users/demo/Documents/Screenwide/Recordings");
        }}
        secondaryAction={{
          label: "Use system default",
          onPress: () => {
            setValue(null);
          },
          type: "reset",
        }}
        value={value}
      />
    </div>
  );
}

function ShortcutControl(props: SettingControlProps) {
  const [value, setValue] = useState<string | null>(
    "CommandOrControl+Shift+KeyR",
  );
  return (
    <div {...props} role="group">
      <HotkeyField
        aria-label="Show recording bar shortcut"
        onChange={setValue}
        value={value}
      />
    </div>
  );
}

function FormatControl(props: SettingControlProps) {
  const [selected, setSelected] = useState("png");
  return (
    <div {...props} role="group">
      <PillGroup
        aria-label="Screenshot format"
        display="label"
        items={[
          { id: "png", label: "PNG" },
          { id: "jpeg", label: "JPEG" },
          { id: "webp", label: "WebP" },
        ]}
        onSelectionChange={setSelected}
        selected={selected}
      />
    </div>
  );
}

export const WithSliderNumberField: Story = {
  args: {
    children: (props) => <GapControl {...props} />,
    description: "Space between arranged windows.",
    title: "Window gap",
  },
};

export const WithPathField: Story = {
  args: {
    children: (props) => <LocationControl {...props} />,
    description: "The folder new recording exports start in.",
    title: "Recording location",
  },
};

export const WithHotkeyField: Story = {
  args: {
    children: (props) => <ShortcutControl {...props} />,
    description: "Choose a shortcut to open the recording controls.",
    title: "Show recording bar",
  },
};

export const WithPillGroup: Story = {
  args: {
    children: (props) => <FormatControl {...props} />,
    description: "The default format for exported screenshots.",
    title: "Screenshot format",
  },
};
