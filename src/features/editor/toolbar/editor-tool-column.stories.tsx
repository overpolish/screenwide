// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";

import {
  EditorToolColumn,
  type EditorToolColumnProps,
} from "./editor-tool-column";

import type { EditorKind } from "./editor-modes";
import type { Meta, StoryObj } from "@storybook/react-vite";

const KINDS: EditorKind[] = ["recording", "screenshot", "audio"];

/** The column's own width, with room for the stage to frame it. */
const STAGE_WIDTH = 120;

const meta = {
  argTypes: {
    hasCamera: { table: { defaultValue: { summary: "true" } } },
    kind: {
      control: "inline-radio",
      options: KINDS,
      table: { defaultValue: { summary: "recording" } },
    },
    mode: {
      control: "select",
      options: [
        null,
        "frame",
        "screen",
        "camera",
        "scene",
        "cursor",
        "keyboard",
        "captions",
        "transcript",
        "sound",
      ],
    },
    tool: {
      control: "select",
      options: [
        "select",
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
    hasCamera: true,
    kind: "recording",
    mode: null,
    onModeChange: fn(),
    onToolChange: fn(),
    tool: "select",
  },
  component: EditorToolColumn,
  parameters: { controls: { include: ["hasCamera", "kind", "mode", "tool"] } },
  // A tool or mode chosen lands in the args, as the editor keeps it: taking
  // up a tool puts the open mode away.
  render: function Render(args) {
    const [, updateArgs] = useArgs<EditorToolColumnProps>();
    return (
      <EditorToolColumn
        {...args}
        onModeChange={(mode) => {
          args.onModeChange(mode);
          updateArgs({ mode });
        }}
        onToolChange={(tool) => {
          args.onToolChange(tool);
          updateArgs({ mode: null, tool });
        }}
      />
    );
  },
  title: "Features/Editor/Toolbar/Tool Column",
} satisfies Meta<typeof EditorToolColumn>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [featureStage({ height: 680, width: STAGE_WIDTH })],
};

export const ShortWindow: Story = {
  decorators: [featureStage({ height: 440, width: STAGE_WIDTH })],
};

export const Kinds: Story = {
  decorators: [featureStage({ width: 240 })],
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      {KINDS.map((kind) => (
        <StoryGridRow key={kind} label={kind}>
          <EditorToolColumn {...args} kind={kind} />
        </StoryGridRow>
      ))}
    </StoryGrid>
  ),
};
