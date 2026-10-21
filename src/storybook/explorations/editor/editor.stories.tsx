// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { EditorWindow } from "./editor-window";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  component: EditorWindow,
  decorators: [featureStage({ window: "editor-recording" })],
  parameters: { layout: "fullscreen" },
  title: "Explorations/Editor",
} satisfies Meta<typeof EditorWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ArrowSelected: Story = { args: { hasPanel: true } };

export const Magnetic: Story = {
  args: { hasPanel: true, isMagnetic: true },
};

export const MagneticSoundOnly: Story = {
  args: { isMagnetic: true, kind: "audio" },
};

export const TracksMenu: Story = { args: { isTracksOpen: true } };

export const AnnotationRing: Story = { args: { isRingOpen: true } };

export const MomentNote: Story = { args: { isNoteOpen: true } };

export const Screenshot: Story = { args: { kind: "screenshot" } };

export const SoundOnly: Story = { args: { kind: "audio" } };
