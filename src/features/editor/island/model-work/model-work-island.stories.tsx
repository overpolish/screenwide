// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import {
  modelWorkIsland,
  type ModelWorkIslandProps,
} from "./model-work-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function ModelWorkIsland({
  isShown,
  ...props
}: ModelWorkIslandProps & EditorIslandStoryProps) {
  const content = modelWorkIsland(props);
  return content ? (
    <EditorIslandStage content={content} isShown={isShown} />
  ) : null;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    isShown: true,
    // Restoring has gone long enough to say how long it has left; finding
    // the vocals has only just begun.
    jobs: [
      { fraction: 0.42, secondsLeft: 150, work: "restoreVoice" },
      { fraction: 0.02, secondsLeft: null, work: "smartDucking" },
      { fraction: 0.8, secondsLeft: 20, work: "followFace" },
    ],
    onCancel: fn(),
  },
  component: ModelWorkIsland,
  decorators: [featureStage({ height: 280, width: 480 })],
  // A cancelled line leaves, as the app takes the work off its list.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ModelWorkIslandProps>();
    return (
      <ModelWorkIsland
        {...args}
        onCancel={(work) => {
          args.onCancel(work);
          updateArgs({ jobs: args.jobs.filter((job) => job.work !== work) });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Model Work",
} satisfies Meta<typeof ModelWorkIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Working: Story = {};
