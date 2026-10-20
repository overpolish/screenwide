// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { ExportWindow } from "./export-window";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    kind: {
      control: "inline-radio",
      options: ["recording", "screenshot", "sound"],
      table: { defaultValue: { summary: "recording" } },
    },
    saving: {
      control: "inline-radio",
      options: [undefined, "progress", "confirm-cancel"],
    },
  },
  component: ExportWindow,
  decorators: [
    featureStage(({ args }) => ({
      window:
        args.kind === "screenshot" ? "export-screenshot" : "export-recording",
    })),
  ],
  parameters: { layout: "fullscreen" },
  title: "Explorations/Export",
} satisfies Meta<typeof ExportWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Video: Story = {};

export const Customised: Story = {
  args: { isCustomised: true },
};

export const SeparateCamera: Story = {
  args: { hasSeparateCamera: true },
};

export const Untranscribed: Story = {
  args: { hasTranscript: false },
};

export const SoundOnly: Story = {
  args: { kind: "sound" },
};

export const Screenshot: Story = {
  args: { kind: "screenshot" },
};

export const Saving: Story = {
  args: { saving: "progress" },
};

export const ConfirmCancel: Story = {
  args: { saving: "confirm-cancel" },
};
