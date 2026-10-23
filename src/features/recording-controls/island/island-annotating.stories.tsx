// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { fn } from "storybook/test";

import {
  PREVIEW_ANNOTATE_SETTINGS,
  PREVIEW_SAVED_COLORS,
} from "../../annotate/island/annotate-preview";
import { useAnnotateRows } from "../../annotate/island/annotate-rows";

import { IslandStage, type IslandStageProps } from "./island-stage";
import { islandMeta } from "./island-story-meta";

import type { AnnotateSection } from "../../annotate/island/use-annotate-parts";
import type { Meta, StoryObj } from "@storybook/react-vite";

/** Logs the annotate actions the story does not carry out. */
const onClear = fn();
const onUndo = fn();

/** The stage recording, with the annotate tools laid under its row. Their
 * choices are kept by the story; Close puts them away, as leaving annotate
 * does, and the island folds back to the recording's one row. */
function AnnotatingStage({
  initialSection,
  ...props
}: IslandStageProps & { initialSection?: AnnotateSection }) {
  const [settings, setSettings] = useState(PREVIEW_ANNOTATE_SETTINGS);
  const [isAnnotating, setIsAnnotating] = useState(true);
  const rows = useAnnotateRows({
    initialSection,
    onChange: (patch) => {
      setSettings((current) => ({ ...current, ...patch }));
    },
    onClear,
    onDone: () => {
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
  title: "Features/Recording Island/Annotating",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Arrow: Story = {};

export const Colours: Story = {
  render: (args) => <AnnotatingStage {...args} initialSection="colors" />,
};

export const Paused: Story = { args: { initialPhase: "paused" } };
