// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { AnnotationColorGrid } from "./annotation-color-grid";
import { AnnotationHeadGroup } from "./annotation-head-group";
import { withAnnotationColor, withoutAnnotationColor } from "./palette";
import { AnnotationHead } from "./types";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  parameters: { layout: "centered" },
  title: "Components/Annotation Style",
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

/** The palette with two colours of your own after it, which is the grid the
 * editor's arrow panel shows. A colour chosen behind Custom shows on its tile;
 * a right click keeps it, and two right clicks on a kept one forget it. */
function ColorGridExample() {
  const [color, setColor] = useState("#7a4fd1");
  const [saved, setSaved] = useState(["#2ec4b6", "#8b5e34"]);
  return (
    <div className="w-64">
      <AnnotationColorGrid
        offersCustom
        onChange={setColor}
        onRemoveColor={(removed) => {
          setSaved((current) => withoutAnnotationColor(current, removed));
        }}
        onSaveColor={(kept) => {
          setSaved((current) => withAnnotationColor(current, kept));
        }}
        savedColors={saved}
        value={color}
      />
    </div>
  );
}

function HeadGroupExample() {
  const [head, setHead] = useState<AnnotationHead>("end");
  return <AnnotationHeadGroup onChange={setHead} value={head} />;
}

export const Colours: Story = { render: () => <ColorGridExample /> };

export const Heads: Story = { render: () => <HeadGroupExample /> };
