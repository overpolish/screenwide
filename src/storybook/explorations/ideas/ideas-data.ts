// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CSSProperties } from "react";

// One mock recording that every idea edits: a 48-second walkthrough of
// making a project, with what was said, what was done, and the screen it
// was done on. The picture's elements double as what a pointer can snap to.

export const RECORDING_S = 48;

export type Segment =
  | { end: number; kind: "pause"; start: number }
  | { end: number; kind: "speech"; start: number; text: string };

export const SEGMENTS: Segment[] = [
  { end: 4.2, kind: "speech", start: 0, text: "Let's set up a new project." },
  { end: 6.8, kind: "pause", start: 4.2 },
  {
    end: 11.5,
    kind: "speech",
    start: 6.8,
    text: "Click New Project in the sidebar.",
  },
  { end: 12.4, kind: "pause", start: 11.5 },
  {
    end: 17.9,
    kind: "speech",
    start: 12.4,
    text: "Give it a name, then pick a template.",
  },
  { end: 23.6, kind: "pause", start: 17.9 },
  {
    end: 29,
    kind: "speech",
    start: 23.6,
    text: "Templates lay out your pages for you.",
  },
  { end: 30.1, kind: "pause", start: 29 },
  {
    end: 36.4,
    kind: "speech",
    start: 30.1,
    text: "Press Command S to save it.",
  },
  { end: 40.9, kind: "pause", start: 36.4 },
  {
    end: 46.2,
    kind: "speech",
    start: 40.9,
    text: "And that's your first project, ready to share.",
  },
  { end: 48, kind: "pause", start: 46.2 },
];

export type Element = {
  /** Position and size in percent of the picture. */
  box: { h: number; w: number; x: number; y: number };
  id: string;
  kind: "button" | "card" | "field" | "nav" | "primary" | "title";
  label: string;
  /** Corner radius in px at the picture's drawn size. */
  radius: number;
};

export const ELEMENTS: Element[] = [
  {
    box: { h: 5.5, w: 30, x: 35, y: 2 },
    id: "search",
    kind: "field",
    label: "Search",
    radius: 7,
  },
  {
    box: { h: 5.5, w: 9, x: 88, y: 2 },
    id: "share",
    kind: "primary",
    label: "Share",
    radius: 7,
  },
  {
    box: { h: 5.5, w: 17, x: 2.5, y: 13 },
    id: "projects",
    kind: "nav",
    label: "Projects",
    radius: 6,
  },
  {
    box: { h: 5.5, w: 17, x: 2.5, y: 20 },
    id: "recent",
    kind: "nav",
    label: "Recent",
    radius: 6,
  },
  {
    box: { h: 5.5, w: 17, x: 2.5, y: 27 },
    id: "shared",
    kind: "nav",
    label: "Shared",
    radius: 6,
  },
  {
    box: { h: 7, w: 17, x: 2.5, y: 88 },
    id: "new",
    kind: "primary",
    label: "New Project",
    radius: 8,
  },
  {
    box: { h: 6, w: 22, x: 26, y: 13 },
    id: "heading",
    kind: "title",
    label: "Templates",
    radius: 2,
  },
  {
    box: { h: 30, w: 21, x: 26, y: 24 },
    id: "blank",
    kind: "card",
    label: "Blank",
    radius: 12,
  },
  {
    box: { h: 30, w: 21, x: 50, y: 24 },
    id: "starter",
    kind: "card",
    label: "Starter",
    radius: 12,
  },
  {
    box: { h: 30, w: 21, x: 74, y: 24 },
    id: "gallery",
    kind: "card",
    label: "Gallery",
    radius: 12,
  },
  {
    box: { h: 7, w: 45, x: 26, y: 62 },
    id: "name",
    kind: "field",
    label: "Project name",
    radius: 8,
  },
  {
    box: { h: 7, w: 21, x: 74, y: 62 },
    id: "create",
    kind: "primary",
    label: "Create",
    radius: 8,
  },
];

export type Action = {
  at: number;
  kind: "click" | "idle" | "keys" | "switch" | "type";
  label: string;
  /** The element the action happened on, when it had one. */
  element?: string;
  /** How long it lasted, for idle stretches. */
  seconds?: number;
};

export const ACTIONS: Action[] = [
  { at: 8.9, element: "new", kind: "click", label: "New Project" },
  { at: 13.4, element: "name", kind: "type", label: "Onboarding" },
  { at: 16.2, element: "starter", kind: "click", label: "Starter" },
  { at: 18.2, kind: "idle", label: "Nothing happens", seconds: 5.2 },
  { at: 26.1, element: "create", kind: "click", label: "Create" },
  { at: 32.9, kind: "keys", label: "⌘S" },
  { at: 37.4, kind: "switch", label: "Safari" },
  { at: 43.1, element: "share", kind: "click", label: "Share" },
];

export const elementById = (id: string | undefined) =>
  ELEMENTS.find((element) => element.id === id);

/** The centre of an element, in percent of the picture. */
export const centreOf = ({ box }: Element) => ({
  x: box.x + box.w / 2,
  y: box.y + box.h / 2,
});

export const clock = (seconds: number) => {
  const whole = Math.max(0, seconds);
  const minutes = Math.floor(whole / 60);
  const rest = whole - minutes * 60;
  return `${String(minutes)}:${rest.toFixed(1).padStart(4, "0")}`;
};

/** An element's place on the picture, as a style. */
export const boxStyle = ({ box }: Element): CSSProperties => ({
  height: `${String(box.h)}%`,
  insetInlineStart: `${String(box.x)}%`,
  top: `${String(box.y)}%`,
  width: `${String(box.w)}%`,
});

/** Minutes and seconds, as a recording counts them. */
export const mmss = (seconds: number) =>
  `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
