// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import {
  stripGroupIsland,
  type StripGroupIslandProps,
} from "./strip-group-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function StripGroupIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & StripGroupIslandProps) {
  return (
    <EditorIslandStage content={stripGroupIsland(props)} isShown={isShown} />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    ...editorIslandStoryMeta.argTypes,
    count: { control: { min: 2, step: 1, type: "number" } },
  },
  args: { count: 3, id: "group", isShown: true, onCut: fn() },
  component: StripGroupIsland,
  decorators: [featureStage({ height: 160, width: 480 })],
  parameters: {
    ...editorIslandStoryMeta.parameters,
    controls: { include: ["isShown", "count"] },
  },
  title: "Features/Editor/Island/Strip Group",
} satisfies Meta<typeof StripGroupIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
