// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindow,
  Hourglass,
  Keyboard,
  MousePointerClick,
  TextCursorInput,
} from "lucide-react";

import { ACTIONS, type Action, RECORDING_S, SEGMENTS } from "./ideas-data";

// The script's lines: what was said and what was done, in order, each owning
// the time up to the next line.

export type Line = {
  at: number;
  end: number;
  id: string;
  text: string;
  action?: Action;
};

export const LINES: Line[] = (() => {
  const raw = [
    ...SEGMENTS.flatMap((segment) =>
      segment.kind === "speech"
        ? [
            {
              at: segment.start,
              id: `s${String(segment.start)}`,
              text: segment.text,
            },
          ]
        : [],
    ),
    ...ACTIONS.map((action) => ({
      action,
      at: action.at,
      id: `a${String(action.at)}`,
      text: action.label,
    })),
  ].sort((first, second) => first.at - second.at);
  return raw.map((line, index) => ({
    ...line,
    end: index + 1 < raw.length ? raw[index + 1].at : RECORDING_S,
  }));
})();

/** Where the work moved on: each becomes a chapter. */
export const STEPS = [
  { at: 0, title: "Make the project" },
  { at: 37.4, title: "Share it" },
];

export const ICONS = {
  click: MousePointerClick,
  idle: Hourglass,
  keys: Keyboard,
  switch: AppWindow,
  type: TextCursorInput,
};

export const describe = ({ kind, label, seconds }: Action) => {
  switch (kind) {
    case "click":
      return `Clicked “${label}”`;
    case "type":
      return `Typed “${label}”`;
    case "keys":
      return `Pressed ${label}`;
    case "switch":
      return `Switched to ${label}`;
    case "idle":
      return `Nothing happens for ${(seconds ?? 0).toFixed(1)} s`;
  }
};
