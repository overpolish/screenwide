// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DELETED,
  type ProjectLocation,
  type ProjectSummary,
  type ProjectTracks,
  type ScrubStrip,
} from "./types";

// Sample projects for the browser's stories: a library of every kind of
// project, with stills drawn as small pictures of what each recorded.

/** The projects folder, a folder beside it, one on an external drive, and
 * one on a drive that is not connected, which the sidebar shows disabled. */
export const locations: ProjectLocation[] = [
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
export const daysAgo = (days: number, hour: number) => {
  const now = new Date();
  return new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() - days,
    hour,
  ).getTime();
};

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

export const projects = [
  project("Product demo"),
  project("Onboarding walkthrough", {
    durationMs: 612_000,
    modifiedMs: daysAgo(0, 9),
    replay: true,
    sizeBytes: 812_000_000,
    tracks: tracks("screen", "camera", "systemAudio", "microphone"),
  }),
  project("Weekly update", {
    durationMs: 245_000,
    kind: "camera",
    modifiedMs: daysAgo(1, 16),
    sizeBytes: 320_000_000,
    tracks: tracks("camera", "microphone"),
  }),
  project("Settings redesign", {
    durationMs: 71_000,
    modifiedMs: daysAgo(1, 11),
    sizeBytes: 96_000_000,
  }),
  project("Podcast intro", {
    durationMs: 45_000,
    kind: "audio",
    modifiedMs: daysAgo(3, 11),
    sizeBytes: 4_200_000,
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

/** Projects deleted in the last 30 days, each with the time it has left
 * before it goes to the Trash. */
export const deletedProjects = [
  { ...projects[3], expiresMs: daysAgo(-29, 10) },
  { ...projects[4], expiresMs: daysAgo(-12, 11) },
  { ...projects[7], expiresMs: daysAgo(-1, 9) },
];

/** What each source in the sidebar lists. */
export const projectsFor = (source: string): ProjectSummary[] => {
  if (source === DELETED) return deletedProjects;
  if (source === locations[0].path)
    return projects.filter(({ available }) => available);
  if (source === locations[1].path) return [projects[3], projects[6]];
  if (source === locations[2].path) return [];
  return projects;
};

const W = 480;
const H = 270;

const svg = (body: string, width = W) =>
  `data:image/svg+xml,${encodeURIComponent(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${String(width)}" height="${String(H)}">${body}</svg>`,
  )}`;

type Palette = {
  accent: string;
  desk: [string, string];
  line: string;
  panel: string;
  side: string;
};

/** A desktop with an app window on it: a sidebar, rows of content, and a
 * button in the accent; `step` moves the pointer and scrolls the rows, so a
 * strip of steps plays as a recording. */
const desktop = (p: Palette, step = 0, id = "d") => {
  const rows = Array.from({ length: 6 }, (_, row) => {
    const y = 78 + row * 26 - ((step * 6) % 26);
    const width = 150 + ((row * 37 + step * 11) % 110);
    return `<rect x="150" y="${String(y)}" width="${String(width)}" height="9" rx="4.5" fill="${p.line}"/>`;
  }).join("");
  const cursorX = 170 + step * 18;
  const cursorY = 200 - step * 9;
  return `<defs><linearGradient id="${id}" x2="1" y2="1"><stop offset="0" stop-color="${p.desk[0]}"/><stop offset="1" stop-color="${p.desk[1]}"/></linearGradient><clipPath id="${id}c"><rect x="130" y="58" width="300" height="180"/></clipPath></defs>
<rect width="${String(W)}" height="${String(H)}" fill="url(#${id})"/>
<rect x="40" y="26" width="400" height="222" rx="12" fill="${p.panel}"/>
<circle cx="58" cy="42" r="4.5" fill="#ff5f57"/><circle cx="72" cy="42" r="4.5" fill="#febc2e"/><circle cx="86" cy="42" r="4.5" fill="#28c840"/>
<rect x="40" y="58" width="90" height="190" fill="${p.side}"/>
${[0, 1, 2, 3].map((i) => `<rect x="52" y="${String(72 + i * 20)}" width="${String(56 - i * 6)}" height="8" rx="4" fill="${p.line}"/>`).join("")}
<g clip-path="url(#${id}c)">${rows}</g>
<rect x="356" y="212" width="66" height="22" rx="11" fill="${p.accent}"/>
<path d="M${String(cursorX)} ${String(cursorY)} l0 18 l5 -5 l7 11 l3 -2 l-7 -10 l7 -1 z" fill="#111" stroke="#fff" stroke-width="1.5"/>`;
};

const light: Palette = {
  accent: "#ff7a3d",
  desk: ["#f6d5c3", "#c9d7f7"],
  line: "#e3e1df",
  panel: "#ffffff",
  side: "#f4f2f0",
};
const dark: Palette = {
  accent: "#7aa2ff",
  desk: ["#1d2547", "#0b0f1e"],
  line: "#3a3f4b",
  panel: "#1c1f26",
  side: "#16181d",
};
const mint: Palette = {
  accent: "#1f9d6f",
  desk: ["#cdeee0", "#9fd5d1"],
  line: "#e2ebe7",
  panel: "#fbfdfc",
  side: "#eff5f2",
};
const sunset: Palette = {
  accent: "#c2410c",
  desk: ["#fde68a", "#f59e0b"],
  line: "#efe7da",
  panel: "#fffaf2",
  side: "#f7efe2",
};

/** Someone talking to camera: a figure against a softly lit wall. */
const camera = svg(
  `<defs><radialGradient id="l" cx="0.3" cy="0.3" r="0.9"><stop offset="0" stop-color="#e8c9a8"/><stop offset="1" stop-color="#5b4636"/></radialGradient></defs>
<rect width="${String(W)}" height="${String(H)}" fill="url(#l)"/>
<rect x="330" y="40" width="90" height="120" rx="6" fill="#3d2f25" opacity="0.35"/>
<circle cx="240" cy="118" r="46" fill="#2a211b"/>
<path d="M140 270 q100 -120 200 0 z" fill="#2a211b"/>`,
);

/** A web page, as a screenshot holds one. */
const page = svg(
  `<rect width="${String(W)}" height="${String(H)}" fill="#fafaf9"/>
<rect x="0" y="0" width="${String(W)}" height="36" fill="#f0efed"/>
<rect x="160" y="62" width="160" height="16" rx="8" fill="#1c1917"/>
<rect x="190" y="88" width="100" height="8" rx="4" fill="#a8a29e"/>
${[0, 1, 2].map((i) => `<rect x="${String(40 + i * 140)}" y="120" width="120" height="120" rx="12" fill="${i === 1 ? "#1c1917" : "#ffffff"}" stroke="#e7e5e4"/><rect x="${String(56 + i * 140)}" y="140" width="50" height="10" rx="5" fill="${i === 1 ? "#ff7a3d" : "#d6d3d1"}"/>`).join("")}`,
);

/** The ribbon an audio project's card shows: drawn white for the card to
 * mask and paint, as the editor composes it. */
const waveform = svg(
  Array.from({ length: 64 }, (_, i) => {
    const height = 18 + Math.abs(Math.sin(i * 0.45) * Math.cos(i * 0.13)) * 150;
    return `<rect x="${String(i * 7.5 + 1)}" y="${String((H - height) / 2)}" width="4" height="${String(height)}" rx="2" fill="#fff"/>`;
  }).join(""),
);

const FRAMES = 12;

/** A scrub strip: frames of the recording side by side, the pointer moving
 * and the content scrolling from one to the next. */
const strip = (palette: Palette): ScrubStrip => ({
  frameHeight: H,
  frameWidth: W,
  frames: FRAMES,
  src: svg(
    Array.from(
      { length: FRAMES },
      (_, index) =>
        `<g transform="translate(${String(index * W)} 0)">${desktop(palette, index, `f${String(index)}`)}</g>`,
    ).join(""),
    FRAMES * W,
  ),
});

export const thumbnails: Record<string, string | null> = {
  [projects[0].file]: svg(desktop(light)),
  [projects[1].file]: svg(desktop(dark)),
  [projects[2].file]: camera,
  [projects[3].file]: svg(desktop(mint)),
  [projects[4].file]: waveform,
  [projects[6].file]: page,
  [projects[7].file]: svg(desktop(sunset)),
  [projects[8].file]: null,
};

export const scrubStrips: Record<string, ScrubStrip | null> = {
  [projects[0].file]: strip(light),
  [projects[1].file]: strip(dark),
  [projects[3].file]: strip(mint),
};
