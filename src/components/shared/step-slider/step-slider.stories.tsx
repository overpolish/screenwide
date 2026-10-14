// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ANNOTATION_WIDTHS } from "../annotation-style/widths";

import { StepSlider } from "./step-slider";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: { layout: "centered" },
  title: "Components/Step Slider",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

function WidthsExample() {
  const [width, setWidth] = useState(16);
  return (
    <StepSlider
      aria-label="Width"
      onChange={setWidth}
      steps={ANNOTATION_WIDTHS}
      value={width}
    />
  );
}

function SizesExample() {
  const [size, setSize] = useState("medium");
  return (
    <StepSlider
      aria-label="Size"
      onChange={setSize}
      steps={["small", "medium", "large", "extra-large"]}
      value={size}
    />
  );
}

/** An arrow's stroke, over steps that are not evenly spaced. */
export const Widths: Story = { render: () => <WidthsExample /> };

/** A caption's size, over named steps. */
export const Sizes: Story = { render: () => <SizesExample /> };
