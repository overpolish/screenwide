// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ComponentProps } from "react";
import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { StoryGrid, StoryGridRow } from "../../storybook/story-grid";

import { ColorGrid } from "./color-grid";
import { withAnnotationColor, withoutAnnotationColor } from "./palette";

import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";

// The width the grid is given in its popup panel, with the panel's inset, so
// the rows wrap where they do in place.
const panel = "w-66.5 p-inset-sm";

const panelWidth: Decorator = (Story) => (
  <div className={panel}>
    <Story />
  </div>
);

const savedColors = ["#2ec4b6", "#8b5e34"];

const meta = {
  argTypes: {
    columns: { control: { min: 1, type: "number" } },
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    offersCustom: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
    value: { control: "color" },
  },
  args: {
    isDisabled: false,
    offersCustom: true,
    onChange: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    savedColors,
    value: "#7a4fd1",
  },
  component: ColorGrid,
  parameters: {
    controls: {
      include: [
        "columns",
        "isDisabled",
        "offersCustom",
        "savedColors",
        "value",
      ],
    },
    layout: "centered",
  },
  // Picks and colours kept or forgotten land in the args, as the window that
  // owns the list would take them.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    const saved = args.savedColors ?? [];
    return (
      <ColorGrid
        {...args}
        onChange={(next) => {
          args.onChange(next);
          updateArgs({ value: next });
        }}
        onRemoveColor={(removed) => {
          args.onRemoveColor?.(removed);
          updateArgs({ savedColors: withoutAnnotationColor(saved, removed) });
        }}
        onSaveColor={(kept) => {
          args.onSaveColor?.(kept);
          updateArgs({ savedColors: withAnnotationColor(saved, kept) });
        }}
      />
    );
  },
  title: "Composites/Color Grid",
} satisfies Meta<typeof ColorGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = { decorators: [panelWidth] };

type Handlers = Pick<
  ComponentProps<typeof ColorGrid>,
  "onChange" | "onRemoveColor" | "onSaveColor"
>;

// Which tile the chosen colour is: one of the palette's, a saved one, or the
// Custom tile, painted with a colour that matches no other.
function StateGrid({
  isDisabled,
  ...handlers
}: Handlers & { isDisabled?: boolean }) {
  return (
    <StoryGrid>
      {[
        { label: "palette", value: "#0088ff" },
        { label: "saved", value: savedColors[0] },
        { label: "custom", value: "#7a4fd1" },
      ].map(({ label, value }) => (
        <StoryGridRow key={label} label={label}>
          <div className={panel}>
            <ColorGrid
              {...handlers}
              isDisabled={isDisabled}
              offersCustom
              savedColors={savedColors}
              value={value}
            />
          </div>
        </StoryGridRow>
      ))}
    </StoryGrid>
  );
}

export const States: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange, onRemoveColor, onSaveColor }) => (
    <StateGrid
      onChange={onChange}
      onRemoveColor={onRemoveColor}
      onSaveColor={onSaveColor}
    />
  ),
};

export const Disabled: Story = {
  parameters: { controls: { disable: true } },
  render: ({ onChange, onRemoveColor, onSaveColor }) => (
    <StateGrid
      isDisabled
      onChange={onChange}
      onRemoveColor={onRemoveColor}
      onSaveColor={onSaveColor}
    />
  ),
};
