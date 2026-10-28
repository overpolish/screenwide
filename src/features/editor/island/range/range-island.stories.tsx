// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { rangeIsland, type RangeIslandProps } from "./range-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function RangeIsland({
  isShown,
  ...props
}: RangeIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={rangeIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: { id: "range", isShown: true, onCut: fn() },
  component: RangeIsland,
  decorators: [featureStage({ height: 160, width: 480 })],
  title: "Features/Editor/Island/Range",
} satisfies Meta<typeof RangeIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
