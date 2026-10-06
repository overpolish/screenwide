// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type ProjectKind = "audio" | "camera" | "screen" | "screenshot";

/** A project as the browser lists it, read from its manifest alone. */
export type ProjectSummary = {
  /** False for a project that is not there, such as one on a drive that is
   * not connected. It is listed but cannot be opened. */
  available: boolean;
  durationMs: number | null;
  /** The project's `.screenwide` manifest, which also names it. */
  file: string;
  kind: ProjectKind | null;
  /** When the project was last edited. */
  modifiedMs: number | null;
  /** Kept from the replay buffer rather than recorded start to stop. */
  replay: boolean;
  /** Everything in the project's folder, in bytes: what keeping it costs. */
  sizeBytes: number | null;
  title: string;
};

/** A folder the browser lists projects from. */
export type ProjectLocation = {
  available: boolean;
  /** The projects folder new recordings go into. */
  isDefault: boolean;
  name: string;
  path: string;
};

export type ProjectSource =
  { kind: "recent" } | { kind: "folder"; path: string };

/** The sidebar's selection: the recent projects, or a location's path. */
export const RECENT = "recent";

export const sourceFor = (selected: string): ProjectSource =>
  selected === RECENT ? { kind: "recent" } : { kind: "folder", path: selected };
