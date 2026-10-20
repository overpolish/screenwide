// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../feature-story-stage";

import { CombinedDock } from "./combined-dock";
import { CombinedEditor } from "./combined-editor";
import { CombinedPreflight } from "./combined-preflight";
import { StateBoard } from "./state-board";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: CombinedEditor,
  parameters: { layout: "centered" },
  title: "Explorations/Direction/Combined",
} satisfies Meta<typeof CombinedEditor>;

export default meta;
type Story = StoryObj<typeof meta>;

const editorWindow: Story = {
  decorators: [featureStage({ window: "editor-recording" })],
  parameters: { layout: "fullscreen" },
};

export const ArrowSelected: Story = {
  ...editorWindow,
  args: { hasPanel: true },
};

export const Magnetic: Story = {
  ...editorWindow,
  args: { hasPanel: true, isMagnetic: true },
};

export const MagneticSoundOnly: Story = {
  ...editorWindow,
  args: { isMagnetic: true, kind: "audio" },
};

export const TracksMenu: Story = {
  ...editorWindow,
  args: { isTracksOpen: true },
};

export const AnnotationRing: Story = {
  ...editorWindow,
  args: { isRingOpen: true },
};

export const MomentNote: Story = {
  ...editorWindow,
  args: { isNoteOpen: true },
};

export const Screenshot: Story = {
  ...editorWindow,
  args: { kind: "screenshot" },
};

export const SoundOnly: Story = { ...editorWindow, args: { kind: "audio" } };

export const Recording: Story = { render: () => <CombinedDock /> };

export const Recorder: Story = { render: () => <CombinedPreflight /> };

export const StateMap: Story = { render: () => <StateBoard /> };
