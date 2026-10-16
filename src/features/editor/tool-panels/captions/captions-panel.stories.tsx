// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../../popup-panel/layout";
import { DEFAULT_CAPTION_SETTINGS } from "../../recording/captions/recording-captions";
import {
  STORY_TEXT_TEMPLATES,
  storyTextStyle,
} from "../text-style/text-template-fixtures";
import { ToolPanel } from "../tool-panel";
import { seedToolPanel } from "../tool-panel-story-seed";
import {
  sampleTranscriptsModel,
  sampleTranscriptWords,
} from "../transcript/transcript-panel-fixtures";

import { rememberCaptionPanelTab } from "./caption-panel-tab";
import { withBreak, withChoice } from "./caption-word-choices";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { ToolPanelTranscript } from "../tool-panel-transcript";
import type { Meta, StoryObj } from "@storybook/react-vite";

const transcript = (
  status: ToolPanelTranscript["status"],
  model = sampleTranscriptsModel("downloaded"),
): ToolPanelTranscript => ({
  allowedWords: [],
  artifactId: 1,
  canRestore: false,
  corrections: [],
  durationMs: 60_000,
  error: null,
  model,
  played: [],
  progress: 0.4,
  removed: [],
  revision: 0,
  scenes: [],
  silences: null,
  status,
});

const captions = (
  change: Partial<RecordingCaptions> = {},
): RecordingCaptions => ({
  enabled: true,
  settings: DEFAULT_CAPTION_SETTINGS,
  style: storyTextStyle("box"),
  ...change,
});

/** The panel showing `shown` captions with a transcript `status` has, open
 * on Words whatever tab a story before it left open. */
const seed = (
  shown: RecordingCaptions,
  status: ToolPanelTranscript["status"] = "ready",
  model = sampleTranscriptsModel("downloaded"),
) => {
  rememberCaptionPanelTab("words");
  seedToolPanel({
    captions: shown,
    textTemplates: STORY_TEXT_TEMPLATES,
    transcript: transcript(status, model),
  });
};

const meta = {
  args: { tool: "captions", workspace: "recording" },
  component: ToolPanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Captions Panel",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Captions shown, as a new recording's are set. */
export const Shown: Story = {
  beforeEach: () => {
    seed(captions());
  },
};

/** The Style tab, with the Pop template in another font and colour and the
 * word being said coloured: the template sets a few big words at a time in
 * capitals. */
export const PopRestyled: Story = {
  beforeEach: () => {
    seed(
      captions({
        style: {
          ...storyTextStyle("pop"),
          font: { family: "Georgia", italic: false, weight: 700 },
          highlightWords: true,
          textColor: "#ff5a5a",
        },
      }),
    );
    rememberCaptionPanelTab("style");
  },
};

/** The Editorial template: each caption's key word, bold in the text, is
 * set apart in a serif italic. */
export const KeyWords: Story = {
  beforeEach: () => {
    seed(captions({ style: storyTextStyle("editorial") }));
  },
};

/** The Style tab for the Stack template: its words stacked, the key word
 * on a line of its own in an accent font, size and colour. */
export const StackAccent: Story = {
  beforeEach: () => {
    seed(captions({ style: storyTextStyle("stack") }));
    rememberCaptionPanelTab("style");
  },
};

/** The Style tab for the Shadow template: which way its words' shadow
 * falls, and the shadow's colour beside the type's. */
export const ShadowAngle: Story = {
  beforeEach: () => {
    seed(captions({ style: storyTextStyle("shadow") }));
    rememberCaptionPanelTab("style");
  },
};

/** Profanity hidden, with how it shows. */
export const HidingProfanity: Story = {
  beforeEach: () => {
    seed(
      captions({
        settings: { ...DEFAULT_CAPTION_SETTINGS, censorProfanity: true },
      }),
    );
  },
};

/** A break placed by hand and a word hidden by hand, which the button under
 * the text takes away together. */
export const PlacedByHand: Story = {
  beforeEach: () => {
    const words = sampleTranscriptWords;
    const durationMs = 60_000;
    seed(
      captions({
        breaks: withBreak(
          [],
          { after: words[4], before: words[3], durationMs },
          "split",
        ),
        wordChoices: withChoice([], { durationMs, word: words[9] }, "hidden"),
      }),
    );
  },
};

/** Turned off: everything stays as it was set, and waits. */
export const Off: Story = {
  beforeEach: () => {
    seed(captions({ enabled: false }));
  },
};

/** No transcript yet: the panel offers to make the one captions come from. */
export const NeedsTranscript: Story = {
  beforeEach: () => {
    seed(captions(), "none");
  },
};

/** No transcription model on this computer yet. */
export const NeedsModel: Story = {
  beforeEach: () => {
    seed(captions(), "none", sampleTranscriptsModel("available"));
  },
};

/** Opened before the editor has sent what the panel shows. */
export const Loading: Story = {
  beforeEach: () => {
    seedToolPanel({ captions: null, textTemplates: null, transcript: null });
  },
};
