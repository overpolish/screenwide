// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ProjectKind } from "../../bindings/ProjectKind";
import type { ProjectLocation } from "../../bindings/ProjectLocation";
import type { ProjectSource } from "../../bindings/ProjectSource";
import type { ProjectSummary } from "../../bindings/ProjectSummary";
import type { ProjectTracks } from "../../bindings/ProjectTracks";
import type { ScrubStripFile } from "../../bindings/ScrubStripFile";

export type {
  ProjectKind,
  ProjectLocation,
  ProjectSource,
  ProjectSummary,
  ProjectTracks,
  ScrubStripFile,
};

/** A recording's scrub strip as a card shows it: the strip file's frames,
 * with the file as an address the webview can load. */
export type ScrubStrip = Omit<ScrubStripFile, "path"> & { src: string };

/** How a location is drawn, in the sidebar and in Move To: the projects
 * folder as the root it is, a folder on another drive as that drive, and
 * any other folder as a folder. */
export const locationGlyph = ({ isDefault, onOtherDrive }: ProjectLocation) =>
  isDefault ? "folder-root" : onOtherDrive ? "hard-drive" : "folder";

/** The sidebar's selection: the recent projects, Recently Deleted, or a
 * location's path. */
export const RECENT = "recent";
export const DELETED = "deleted";

export const sourceFor = (selected: string): ProjectSource =>
  selected === RECENT
    ? { kind: "recent" }
    : selected === DELETED
      ? { kind: "deleted" }
      : { kind: "folder", path: selected };
