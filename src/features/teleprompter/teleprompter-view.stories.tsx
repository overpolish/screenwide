// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useMemo, useState } from "react";
import { fn } from "storybook/test";

import poster from "../../site/public/recordings/edit.jpg";
import soundPoster from "../../site/public/recordings/sound.jpg";
import { featureStage } from "../../storybook/feature-story-stage";

import { movedTo } from "./cues/place-cue";
import { scriptText } from "./script-words";
import { TeleprompterPreferences } from "./teleprompter-preferences";
import { TeleprompterView } from "./teleprompter-view";

import type { ToolPanelModel } from "../editor/tools/tool-panel-selection";
import type { Meta, StoryObj } from "@storybook/react-vite";

const SCRIPT = [
  "Launch video",
  "",
  "Hi, I'm Dom. Today we look at the new editor, and the way it keeps your place while you work.",
  "",
  "First, the timeline. Then the islands, and how they move with you as you edit.",
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

/** Dictation's English model, downloaded, at its real size. */
const VOICE_MODEL: ToolPanelModel = {
  progress: 0,
  sizeBytes: 1_471_720_736,
  status: "downloaded",
};

const meta = {
  args: {
    cuePlace: { kind: "top" },
    cues: [],
    isFullscreen: false,
    isPlaying: false,
    onClose: fn(),
    onCuesMove: fn(),
    onDownloadVoiceModel: fn(),
    onEdit: fn(),
    onFullscreenChange: fn(),
    onPlace: fn(),
    onPlayingChange: fn(),
    onPreferencesChange: fn(),
    onRename: fn(),
    onRestart: fn(),
    onScrollTo: fn(),
    onTextFrameChange: fn(),
    onWordMenu: fn(),
    place: 9,
    placing: null,
    preferences: PREFERENCES,
    script: scriptText(SCRIPT),
    shownCue: null,
    text: SCRIPT,
    title: "Launch video script",
    voice: { kind: "idle" },
    voiceModel: VOICE_MODEL,
  },
  component: TeleprompterView,
  decorators: [featureStage({ height: 280, width: 680 })],
  parameters: {
    controls: { include: ["isPlaying", "title"] },
  },
  // Typing, scrolling, dragging a cue and the bar's settings change the
  // story's script, place, cues and preferences as they would the
  // teleprompter's, in the story's own state: args reach the story a moment
  // late, which would put a field typed into back between keys.
  render: function Render(args) {
    const [text, setText] = useState(args.text);
    const [place, setPlace] = useState(args.place);
    const [cues, setCues] = useState(args.cues);
    const [preferences, setPreferences] = useState(args.preferences);
    const script = useMemo(() => scriptText(text), [text]);
    return (
      <TeleprompterView
        {...args}
        cues={cues}
        onCuesMove={(moved) => {
          args.onCuesMove(moved);
          setCues((current) => movedTo(current, moved));
        }}
        onEdit={(next) => {
          args.onEdit(next);
          setText(next);
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
        text={text}
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
  args: {
    voiceModel: { ...VOICE_MODEL, status: "available" },
  },
};

export const DownloadingModel: Story = {
  args: {
    voiceModel: { ...VOICE_MODEL, progress: 0.4, status: "downloading" },
  },
};

export const Empty: Story = {
  args: { place: 0, script: scriptText(""), text: "" },
};

const CUE = {
  cues: [
    {
      at: SCRIPT.indexOf("timeline"),
      id: "timeline",
      src: poster,
      title: "Editor walkthrough",
    },
    {
      at: SCRIPT.indexOf("exporting"),
      id: "exporting",
      src: soundPoster,
      title: "Export walkthrough",
    },
  ],
  place: SCRIPT.slice(0, SCRIPT.indexOf("timeline")).split(/\s+/).length - 2,
  shownCue: { id: "timeline", src: poster, title: "Editor walkthrough" },
};

export const CueInMargin: Story = {
  args: {
    ...CUE,
    cuePlace: { kind: "margin", textSize: PREFERENCES.textSize, width: 170 },
    preferences: { ...PREFERENCES, columnWidth: 340 },
  },
};

export const CueOverText: Story = {
  args: CUE,
};

export const PlacingCue: Story = {
  args: { placing: { picture: poster } },
};
