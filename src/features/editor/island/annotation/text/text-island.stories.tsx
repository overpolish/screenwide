// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../../storybook/feature-story-stage";
import { titleTemplates } from "../../../tool-panels/annotation/text/text-look";
import {
  STORY_TEXT_TEMPLATES,
  storyTextStyle,
} from "../../../tool-panels/text-style/text-template-fixtures";
import { EditorIslandStage } from "../../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../../editor-island-story-meta";
import { storyPinAfter } from "../annotation-pin-story-fixtures";

import { textIsland, type TextIslandProps } from "./text-island";

import type { FontFamily } from "../../../../../bindings/FontFamily";
import type { TextStyle } from "../../../../../bindings/TextStyle";
import type { Meta, StoryObj } from "@storybook/react-vite";

function TextIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & TextIslandProps) {
  return <EditorIslandStage content={textIsland(props)} isShown={isShown} />;
}

// A template's look as a text placed by hand takes it up.
const titleLook = (template: string): TextStyle =>
  titleTemplates(STORY_TEXT_TEMPLATES).styles.find(
    (style) => style.template === template,
  ) ?? storyTextStyle(template);

// Where no system can be asked, so the font list has more than one face.
const FAMILIES: FontFamily[] = [
  { italic: true, name: "Georgia", weights: [400, 700] },
  { italic: true, name: "Helvetica Neue", weights: [300, 400, 500, 700] },
  { italic: false, name: "Menlo", weights: [400, 700] },
  { italic: true, name: "Times New Roman", weights: [400, 700] },
];

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    emphasis: [],
    families: FAMILIES,
    id: "text",
    isShown: true,
    look: titleLook("label"),
    onDelete: fn(),
    onEditGestureEnd: fn(),
    onEditGestureStart: fn(),
    onEmphasisChange: fn(),
    onLookChange: fn(),
    onPinAction: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    onStyleChange: fn(),
    pin: null,
    savedColors: [],
    style: { align: "center", isAnimated: true, size: 24 },
    templates: STORY_TEXT_TEMPLATES,
    text: "Ship it on Friday",
  },
  component: TextIsland,
  decorators: [featureStage({ height: 640, width: 560 })],
  // Every choice changes the story's args, as it would the text.
  render: function Render(args) {
    const [, updateArgs] = useArgs<TextIslandProps>();
    return (
      <TextIsland
        {...args}
        onEmphasisChange={(emphasis) => {
          args.onEmphasisChange(emphasis);
          updateArgs({ emphasis });
        }}
        onLookChange={(patch) => {
          args.onLookChange(patch);
          updateArgs({ look: { ...args.look, ...patch } });
        }}
        onPinAction={(action) => {
          args.onPinAction?.(action);
          updateArgs({ pin: storyPinAfter(args.pin, action) });
        }}
        onRemoveColor={(color) => {
          args.onRemoveColor(color);
          updateArgs({
            savedColors: args.savedColors.filter((kept) => kept !== color),
          });
        }}
        onSaveColor={(color) => {
          args.onSaveColor(color);
          if (!args.savedColors.includes(color))
            updateArgs({ savedColors: [...args.savedColors, color] });
        }}
        onStyleChange={(style) => {
          args.onStyleChange(style);
          updateArgs({ style: { ...args.style, ...style } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Text",
} satisfies Meta<typeof TextIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A template that sets a word apart in an accent.
export const Accent: Story = {
  args: {
    look: titleLook("editorial"),
    text: "Ship it on Friday\nthen tell the whole team\nwhat changed and why",
  },
};

// A screenshot is one instant: nothing grows in or comes on word by word.
export const Screenshot: Story = {
  args: { canAnimate: false, pin: undefined },
};
