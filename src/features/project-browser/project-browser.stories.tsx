// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { ProjectBrowser } from "./project-browser";
import {
  DELETED,
  RECENT,
  type ProjectLocation,
  type ProjectSummary,
  type ProjectTracks,
  type ScrubStrip,
} from "./types";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The projects folder, a folder beside it, one on an external drive, and
 * one on a drive that is not connected, which the sidebar shows disabled. */
const locations: ProjectLocation[] = [
  {
    available: true,
    isDefault: true,
    name: "Screenwide",
    onOtherDrive: false,
    path: "/Users/demo/Movies/Screenwide",
  },
  {
    available: true,
    isDefault: false,
    name: "Tutorials",
    onOtherDrive: false,
    path: "/Users/demo/Documents/Tutorials",
  },
  {
    available: true,
    isDefault: false,
    name: "Client work",
    onOtherDrive: true,
    path: "/Volumes/Archive/Client work",
  },
  {
    available: false,
    isDefault: false,
    name: "Old drive",
    onOtherDrive: true,
    path: "/Volumes/Old drive/Recordings",
  },
];

/** Edit times relative to today, so the date groups read the same whenever
 * the story is opened. */
const daysAgo = (days: number, hour: number) => {
  const now = new Date();
  return new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() - days,
    hour,
  ).getTime();
};

/** What each fixture captured: one of each combination worth seeing. */
const tracks = (...present: (keyof ProjectTracks)[]): ProjectTracks => ({
  camera: present.includes("camera"),
  microphone: present.includes("microphone"),
  screen: present.includes("screen"),
  systemAudio: present.includes("systemAudio"),
});

const project = (
  title: string,
  overrides: Partial<ProjectSummary> = {},
): ProjectSummary => ({
  available: true,
  durationMs: 94_000,
  expiresMs: null,
  file: `/Users/demo/Movies/Screenwide/${title}/${title}.screenwide`,
  kind: "screen",
  modifiedMs: daysAgo(0, 10),
  replay: false,
  sizeBytes: 148_000_000,
  title,
  tracks: tracks("screen", "systemAudio", "microphone"),
  ...overrides,
});

const projects = [
  project("Product demo"),
  project("Onboarding walkthrough", {
    durationMs: 612_000,
    modifiedMs: daysAgo(0, 9),
    replay: true,
    tracks: tracks("screen", "camera", "systemAudio", "microphone"),
  }),
  project("Screenwide 2026-10-06 at 16.00.54", {
    kind: "camera",
    modifiedMs: daysAgo(1, 16),
    tracks: tracks("camera", "microphone"),
  }),
  project("Podcast intro", {
    durationMs: 45_000,
    kind: "audio",
    modifiedMs: daysAgo(3, 11),
    tracks: tracks("microphone"),
  }),
  project("Release notes", {
    available: false,
    durationMs: null,
    kind: null,
    modifiedMs: daysAgo(12, 14),
    sizeBytes: null,
    tracks: null,
  }),
  project("Pricing page", {
    durationMs: null,
    kind: "screenshot",
    modifiedMs: daysAgo(20, 15),
    sizeBytes: 2_400_000,
    tracks: null,
  }),
  project("Launch teaser", {
    durationMs: 31_000,
    modifiedMs: daysAgo(45, 12),
    sizeBytes: 61_000_000,
    tracks: tracks("screen"),
  }),
  project("Bug report", {
    durationMs: 18_000,
    modifiedMs: daysAgo(400, 9),
    sizeBytes: 22_000_000,
  }),
];

const svg = (body: string, width = 480) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${String(width)}" height="270">${body}</svg>`,
  )}`;

/** A light and a dark still, so the badges are seen over both. */
const still = (from: string, to: string) =>
  svg(
    `<defs><linearGradient id="g" x2="1" y2="1"><stop offset="0" stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="480" height="270" fill="url(#g)"/>`,
  );

/** A scrub strip: twelve frames side by side, a dot crossing each further
 * than the last, so moving the pointer across the card visibly scrubs. */
const strip = (background: string, dot: string): ScrubStrip => ({
  frameHeight: 270,
  frameWidth: 480,
  frames: 12,
  src: svg(
    Array.from(
      { length: 12 },
      (_, index) =>
        `<rect x="${String(index * 480)}" width="480" height="270" fill="${background}"/><circle cx="${String(index * 480 + 48 + index * 35)}" cy="135" r="28" fill="${dot}"/>`,
    ).join(""),
    12 * 480,
  ),
});

const thumbnails = {
  [projects[0].file]: still("#f5f5f7", "#c7d2fe"),
  [projects[1].file]: still("#1e293b", "#0f172a"),
  [projects[6].file]: still("#fde68a", "#f59e0b"),
};

const scrubStrips = {
  [projects[0].file]: strip("#e0e7ff", "#4f46e5"),
  [projects[1].file]: strip("#0f172a", "#38bdf8"),
};

const meta = {
  args: {
    deleted: null,
    error: null,
    locations,
    onAddLocation: () => undefined,
    onClose: () => undefined,
    onDelete: () => undefined,
    onDuplicate: () => undefined,
    onEmpty: () => undefined,
    onForget: () => undefined,
    onMinimize: () => undefined,
    onMove: () => undefined,
    onOpen: () => undefined,
    onOpenFile: () => undefined,
    onOpenLocation: () => undefined,
    onRemoveLocation: () => undefined,
    onRename: () => Promise.resolve(),
    onRestore: () => undefined,
    onReveal: () => undefined,
    onSelect: () => undefined,
    onTrash: () => undefined,
    onUndoDelete: () => undefined,
    projects,
    scrubStrips,
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

/** Grouped by when each was last edited. Point at the first two to scrub
 * through them; tick a card, or Command-click it, to start choosing. */
export const Recent: Story = {};

/** A recent project that is no longer where it was, such as one deleted or
 * moved outside the app. It cannot be opened, but can be taken off Recent
 * from its menu. */
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

/** Just after a choice was deleted, while its notice can undo it. */
export const Deleted: Story = {
  args: {
    deleted: { count: 2, title: projects[0].title },
    projects: projects.slice(2),
  },
};

/** Projects deleted in the last 30 days, each with the time it has left
 * before it goes to the Trash. They are restored or sent on, not opened. */
export const RecentlyDeleted: Story = {
  args: {
    projects: [
      { ...projects[0], expiresMs: daysAgo(-29, 10) },
      { ...projects[3], expiresMs: daysAgo(-12, 11) },
      { ...projects[6], expiresMs: daysAgo(-1, 9) },
    ],
    selected: DELETED,
  },
};

/** Projects on their way to another drive, copied before the originals go. */
export const Moving: Story = {
  args: {
    copying: {
      count: 3,
      destination: "Client work",
      fraction: 0.42,
      kind: "move",
      title: projects[0].title,
    },
  },
};

export const Empty: Story = {
  args: { projects: [], selected: "/Users/demo/Movies/Screenwide" },
};

export const ActionFailed: Story = {
  args: {
    error: "That name cannot be used",
  },
};
