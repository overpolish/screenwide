// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { marqueeIsland, type MarqueeIslandProps } from "./marquee-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function MarqueeIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & MarqueeIslandProps) {
  return <EditorIslandStage content={marqueeIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    ...editorIslandStoryMeta.argTypes,
    count: { control: { min: 2, step: 1, type: "number" } },
  },
  args: { count: 3, isShown: true, onDelete: fn() },
  component: MarqueeIsland,
  decorators: [featureStage({ height: 160, width: 480 })],
  parameters: {
    ...editorIslandStoryMeta.parameters,
    controls: { include: ["isShown", "count"] },
  },
  title: "Features/Editor/Island/Marquee",
} satisfies Meta<typeof MarqueeIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
