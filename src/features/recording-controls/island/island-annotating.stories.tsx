// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import {
  PREVIEW_ANNOTATE_SETTINGS,
  PREVIEW_SAVED_COLORS,
} from "../../annotate/island/annotate-preview";
import { useAnnotateRows } from "../../annotate/island/annotate-rows";

import { IslandStage } from "./island-stage";
import { islandMeta } from "./island-story-meta";

import type { IslandStageProps } from "./island-stage-props";
import type { AnnotateSection } from "../../annotate/island/use-annotate-parts";
import type { Meta, StoryObj } from "@storybook/react-vite";

// Logs the annotate actions, including those the story also carries out.
const onChange = fn();
const onClear = fn();
const onDone = fn();
const onUndo = fn();

// Keeps the tools' choices; Close puts them away, as leaving annotate does.
function AnnotatingStage({
  initialSection,
  ...props
}: IslandStageProps & { initialSection?: AnnotateSection }) {
  const [settings, setSettings] = useState(PREVIEW_ANNOTATE_SETTINGS);
  const [isAnnotating, setIsAnnotating] = useState(true);
  const rows = useAnnotateRows({
    initialSection,
    onChange: (patch) => {
      onChange(patch);
      setSettings((current) => ({ ...current, ...patch }));
    },
    onClear,
    onDone: () => {
      onDone();
      setIsAnnotating(false);
    },
    onUndo,
    savedColors: PREVIEW_SAVED_COLORS,
    settings,
  });
  return (
    <IslandStage {...props} annotate={(isAnnotating && rows) || undefined} />
  );
}

const meta = {
  ...islandMeta,
  args: { ...islandMeta.args, initialPhase: "recording" },
  render: (args) => <AnnotatingStage {...args} />,
  title: "Features/Recording Controls/Island/Annotating",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Arrow: Story = {};

export const Colours: Story = {
  render: (args) => <AnnotatingStage {...args} initialSection="colors" />,
};

export const Paused: Story = { args: { initialPhase: "paused" } };
