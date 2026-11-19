// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useMemo, useState } from "react";
import { fn } from "storybook/test";

import { featureStage } from "../../storybook/feature-story-stage";

import { scriptText } from "./script-words";
import { TeleprompterPreferences } from "./teleprompter-preferences";
import { TeleprompterView } from "./teleprompter-view";

import type { Meta, StoryObj } from "@storybook/react-vite";

const MARKDOWN = [
  "# Launch video",
  "",
  "Hi, I'm Dom. Today we look at the **new** editor, and the way it keeps your place while you work.",
  "",
  "- First, the timeline.",
  "- Then the islands, and how they move with you as you edit.",
  "",
  "Finally, exporting: one click, and the file is ready to share.",
].join("\n");

const PREFERENCES: TeleprompterPreferences = {
  columnWidth: null,
  isMirrored: false,
  mode: "voice",
  textSize: 32,
  wordsPerMinute: 140,
};

const meta = {
  args: {
    isFullscreen: false,
    isPlaying: false,
    markdown: MARKDOWN,
    onClose: fn(),
    onEdit: fn(),
    onFullscreenChange: fn(),
    onOpenModels: fn(),
    onPlayingChange: fn(),
    onPreferencesChange: fn(),
    onRename: fn(),
    onRestart: fn(),
    onScrollTo: fn(),
    place: 9,
    preferences: PREFERENCES,
    script: scriptText(MARKDOWN),
    title: "Launch video script",
    voice: { kind: "idle" },
  },
  component: TeleprompterView,
  decorators: [featureStage({ height: 280, width: 680 })],
  parameters: {
    controls: { include: ["isPlaying", "title"] },
  },
  // Typing, scrolling and the bar's settings change the story's script,
  // place and preferences as they would the teleprompter's, in the story's
  // own state: args reach the story a moment late, which would put a field
  // typed into back between keys.
  render: function Render(args) {
    const [markdown, setMarkdown] = useState(args.markdown);
    const [place, setPlace] = useState(args.place);
    const [preferences, setPreferences] = useState(args.preferences);
    const script = useMemo(() => scriptText(markdown), [markdown]);
    return (
      <TeleprompterView
        {...args}
        markdown={markdown}
        onEdit={(next) => {
          args.onEdit(next);
          setMarkdown(next);
        }}
        onPreferencesChange={(patch) => {
          args.onPreferencesChange(patch);
          setPreferences((current) => ({ ...current, ...patch }));
        }}
        onScrollTo={(next) => {
          args.onScrollTo(next);
          setPlace(next);
        }}
        place={place}
        preferences={preferences}
        script={script}
      />
    );
  },
  title: "Features/Teleprompter/Window",
} satisfies Meta<typeof TeleprompterView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Paused: Story = {};

export const Steady: Story = {
  args: { preferences: { ...PREFERENCES, mode: "steady" } },
};

export const NarrowColumn: Story = {
  args: { preferences: { ...PREFERENCES, columnWidth: 420 } },
};

export const Mirrored: Story = {
  args: { preferences: { ...PREFERENCES, isMirrored: true } },
};

export const NoModel: Story = {
  args: { voice: { kind: "problem", problem: "noModel" } },
};

export const Empty: Story = {
  args: { markdown: "", place: 0, script: scriptText("") },
};
