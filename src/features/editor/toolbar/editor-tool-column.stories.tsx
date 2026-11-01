// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";

import {
  EditorToolColumn,
  type EditorToolColumnProps,
} from "./editor-tool-column";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The column's own width, with room for the stage to frame it. */
const STAGE_WIDTH = 120;

const meta = {
  argTypes: {
    isModeOpen: { table: { defaultValue: { summary: "false" } } },
    kind: {
      control: "inline-radio",
      options: ["recording", "screenshot", "audio"],
      table: { defaultValue: { summary: "recording" } },
    },
    tool: {
      control: "select",
      options: [
        "select",
        "crop",
        "image",
        "arrow",
        "text",
        "redact",
        "draw",
        "shape",
        "highlight",
        "counter",
        "focus",
        "magnify",
      ],
      table: { defaultValue: { summary: "select" } },
    },
  },
  args: {
    isModeOpen: false,
    kind: "recording",
    onToolChange: fn(),
    tool: "select",
  },
  component: EditorToolColumn,
  parameters: { controls: { include: ["isModeOpen", "kind", "tool"] } },
  // A tool chosen lands in the args, as the editor keeps it: taking up a
  // tool puts the open mode away.
  render: function Render(args) {
    const [, updateArgs] = useArgs<EditorToolColumnProps>();
    return (
      <EditorToolColumn
        {...args}
        onToolChange={(tool) => {
          args.onToolChange(tool);
          updateArgs({ isModeOpen: false, tool });
        }}
      />
    );
  },
  title: "Features/Editor/Toolbar/Tool Column",
} satisfies Meta<typeof EditorToolColumn>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [featureStage({ height: 520, width: STAGE_WIDTH })],
};

export const ShortWindow: Story = {
  decorators: [featureStage({ height: 300, width: STAGE_WIDTH })],
};
