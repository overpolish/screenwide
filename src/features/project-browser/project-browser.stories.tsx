// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { ProjectBrowser } from "./project-browser";
import { RECENT, type ProjectLocation, type ProjectSummary } from "./types";

import type { Meta, StoryObj } from "@storybook/react-vite";

const EDITED = Date.UTC(2026, 9, 6, 14, 47);

const locations: ProjectLocation[] = [
  {
    available: true,
    isDefault: true,
    name: "Screenwide",
    path: "/Users/demo/Movies/Screenwide",
  },
  {
    available: true,
    isDefault: false,
    name: "Client work",
    path: "/Volumes/Archive/Client work",
  },
  {
    available: false,
    isDefault: false,
    name: "Old drive",
    path: "/Volumes/Old drive/Recordings",
  },
];

const project = (
  title: string,
  overrides: Partial<ProjectSummary> = {},
): ProjectSummary => ({
  available: true,
  durationMs: 94_000,
  file: `/Users/demo/Movies/Screenwide/${title}/${title}.screenwide`,
  kind: "screen",
  modifiedMs: EDITED,
  replay: false,
  sizeBytes: 148_000_000,
  title,
  ...overrides,
});

const projects = [
  project("Product demo"),
  project("Onboarding walkthrough", { durationMs: 612_000, replay: true }),
  project("Screenwide 2026-10-06 at 16.00.54", { kind: "camera" }),
  project("Podcast intro", { durationMs: 45_000, kind: "audio" }),
  project("Release notes", {
    available: false,
    durationMs: null,
    kind: null,
    sizeBytes: null,
  }),
  project("Pricing page", {
    durationMs: null,
    kind: "screenshot",
    sizeBytes: 2_400_000,
  }),
];

/** A light and a dark still, so the badges are seen over both. */
const still = (from: string, to: string) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="480" height="270"><defs><linearGradient id="g" x2="1" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="480" height="270" fill="url(#g)"/></svg>`,
  )}`;

const thumbnails = {
  [projects[0].file]: still("#f5f5f7", "#c7d2fe"),
  [projects[1].file]: still("#1e293b", "#0f172a"),
};

const meta = {
  args: {
    error: null,
    locations,
    onAddLocation: () => undefined,
    onClose: () => undefined,
    onForget: () => undefined,
    onMinimize: () => undefined,
    onOpen: () => undefined,
    onOpenFile: () => undefined,
    onOpenLocation: () => undefined,
    onRemoveLocation: () => undefined,
    onRename: () => Promise.resolve(),
    onReveal: () => undefined,
    onSelect: () => undefined,
    onTrash: () => undefined,
    projects,
    selected: RECENT,
    thumbnails,
  },
  component: ProjectBrowser,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={560} viewMode={context.viewMode} width={880}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Project Browser",
} satisfies Meta<typeof ProjectBrowser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recent: Story = {};

/** A recent project that is no longer where it was, such as one deleted or
 * moved outside the app. It cannot be opened, but can be taken off Recent. */
export const MissingRecent: Story = {
  args: { projects: [projects[0], projects[4]] },
};

/** A folder added to the sidebar. Its right-click menu opens it or takes it
 * off the sidebar. */
export const AddedLocation: Story = {
  args: {
    projects: projects.slice(0, 2),
    selected: "/Volumes/Archive/Client work",
  },
};

/** A folder on a drive that is not connected. */
export const LocationNotAvailable: Story = {
  args: { projects: [], selected: "/Volumes/Old drive/Recordings" },
};

export const Empty: Story = {
  args: { projects: [], selected: "/Users/demo/Movies/Screenwide" },
};

export const ActionFailed: Story = {
  args: {
    error: "That name cannot be used",
  },
};
