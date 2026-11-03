// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { UpdateIsland } from "./update-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

// Shaped like GitHub's sanitized `body_html`: headings, lists, inline code, a
// task list and a link.
const releaseNotesHtml = `
  <h2>Highlights</h2>
  <p>Recording is steadier on every display, and exports finish sooner.</p>
  <ul>
    <li>Capture windows and regions more reliably.</li>
    <li>Added smoother cursor movement to exported recordings.</li>
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
    <a href="https://github.com/overpolish/screenwide/commits/v1.1.0">https://github.com/overpolish/screenwide/commits/v1.1.0</a>
  </p>
`;

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    currentVersion: "1.0.0",
    downloadProgress: null,
    error: null,
    isNotesOpen: false,
    isShown: true,
    onInstall: fn(),
    onLater: fn(),
    onNotesOpenChange: fn(),
    onSkip: fn(),
    releaseDate: "2026-10-28T12:00:00Z",
    releaseNotes: releaseNotesHtml,
    status: "available",
    updateVersion: "1.1",
  },
  component: UpdateIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch === "off" ? null : PREVIEW_NOTCH,
        },
      }),
    featureStage({ height: 480, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: args.notch ? 0 : ISLAND.screenGap }}
      >
        <UpdateIsland {...args} />
      </div>
      {args.notch ? <StoryNotch notch={args.notch} /> : null}
    </div>
  ),
  title: "Features/Updates/Island/Update",
} satisfies Meta<typeof UpdateIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Available: Story = {};

export const WhatsNewOpen: Story = { args: { isNotesOpen: true } };

export const Downloading: Story = {
  args: { downloadProgress: 0.42, status: "downloading" },
};

export const Error: Story = {
  args: {
    error: "The downloaded update could not be verified.",
    status: "error",
  },
};

export const Installing: Story = {
  args: { downloadProgress: 1, status: "downloading" },
};
