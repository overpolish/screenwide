// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { UpdatePrompt } from "./update-prompt";

const previewWidth = 620;
const previewHeight = 520;

// Shaped like GitHub's sanitized `body_html`: headings, paragraphs, lists and
// inline code, so the capped type scale is visible in the story.
const releaseNotesHtml = `
  <h2>Highlights</h2>
  <p>Recording is steadier on every display, and exports finish sooner.</p>
  <ul>
    <li>Capture windows and regions more reliably.</li>
    <li>
      Added smoother cursor movement to exported recordings.
      <ul>
        <li>Cursor size follows the <strong>zoom level</strong>.</li>
      </ul>
    </li>
    <li>Press <code>CommandOrControl+Shift+R</code> to start recording.</li>
  </ul>
  <h2>Fixes</h2>
  <ol>
    <li>Fixed occasional blank frames at the start of recordings.</li>
    <li>Fixed window capture when an application changes size.</li>
  </ol>
  <ul class="contains-task-list">
    <li class="task-list-item">
      <input aria-label="Incomplete task" class="task-list-item-checkbox" disabled type="checkbox" />
      Windows parity for glide
    </li>
    <li class="task-list-item">
      <input aria-label="Completed task" checked class="task-list-item-checkbox" disabled type="checkbox" />
      Anti-flicker toggle
    </li>
  </ul>
  <p>
    <strong>Full Changelog</strong>:
    <a href="https://github.com/overpolish/screenwide/commits/v1.0.0">https://github.com/overpolish/screenwide/commits/v1.0.0</a>
  </p>
`;

const meta = {
  args: {
    currentVersion: "0.1.0",
    downloadProgress: null,
    error: null,
    onInstall: () => undefined,
    onRemindLater: () => undefined,
    onSkipVersion: () => undefined,
    releaseDate: "2026-08-18T12:00:00Z",
    releaseNotes: releaseNotesHtml,
    status: "available" as const,
    updateVersion: "1.0.0",
  },
  component: UpdatePrompt,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage
        height={previewHeight}
        viewMode={context.viewMode}
        width={previewWidth}
      >
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Update Prompt",
} satisfies Meta<typeof UpdatePrompt>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Available: Story = {};

export const NoReleaseNotes: Story = {
  args: { releaseNotes: null },
};

export const Downloading: Story = {
  args: { downloadProgress: null, status: "downloading" },
};

export const DownloadingWithProgress: Story = {
  args: { downloadProgress: 0.42, status: "downloading" },
};

export const Error: Story = {
  args: {
    error: "The downloaded update could not be verified.",
    status: "error",
  },
};
