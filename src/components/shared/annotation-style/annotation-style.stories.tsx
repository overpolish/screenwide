// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { AnnotationColorGrid } from "./annotation-color-grid";
import { AnnotationHeadGroup } from "./annotation-head-group";
import { AnnotationWidthSlider } from "./annotation-width-slider";
import { AnnotationHead } from "./types";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: { layout: "centered" },
  title: "Components/Annotation Style",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

/** The palette with two colours of your own after it, which is the grid the
 * editor's arrow panel shows. */
function ColorGridExample() {
  const [color, setColor] = useState("#ffcc00");
  return (
    <div className="w-64">
      <AnnotationColorGrid
        onChange={setColor}
        onContextMenuColor={() => undefined}
        onSettled={setColor}
        savedColors={["#2ec4b6", "#8b5e34"]}
        value={color}
      />
    </div>
  );
}

function WidthSliderExample() {
  const [width, setWidth] = useState(16);
  return <AnnotationWidthSlider onChange={setWidth} value={width} />;
}

function HeadGroupExample() {
  const [head, setHead] = useState<AnnotationHead>("end");
  return <AnnotationHeadGroup onChange={setHead} value={head} />;
}

export const Colours: Story = { render: () => <ColorGridExample /> };
export const Widths: Story = { render: () => <WidthSliderExample /> };

export const Heads: Story = { render: () => <HeadGroupExample /> };
