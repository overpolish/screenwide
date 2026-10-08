// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type {
  ProjectLocation,
  ProjectSource,
  ProjectSummary,
  ScrubStripFile,
} from "./types";

/** Sent whenever what the browser shows may have changed. */
export const PROJECTS_CHANGED_EVENT = "projects://changed";

export const getProjectLocations = () =>
  invoke<ProjectLocation[]>("get_project_locations");

export const listProjects = (source: ProjectSource) =>
  invoke<ProjectSummary[]>("list_projects", { source });

/** A still for a project's card: a file path, or null when there is none.
 * An audio recording's is its ribbon in white, for the card to tint. */
export const getProjectThumbnail = (file: string) =>
  invoke<string | null>("get_project_thumbnail", { file });

/** The scrub strip the editor composed along a recording's edit, or null
 * while it has none. */
export const getScrubStrip = (file: string) =>
  invoke<ScrubStripFile | null>("get_scrub_strip", { file });

/** Sent with a project's manifest when the editor has drawn it a new still. */
export const PROJECT_STILL_EVENT = "projects://still";

export const openProjectFile = (file: string) =>
  invoke<null>("open_project_file", { file });

export const openOtherProject = () => invoke<null>("open_other_project");

export const revealProject = (file: string) =>
  invoke<null>("reveal_project", { file });

/** Opens one of the browser's folders in Finder or Explorer. */
export const openProjectLocation = (path: string) =>
  invoke<null>("open_project_location", { path });
/** Resolves to the renamed project's manifest. */
export const renameProject = (file: string, title: string) =>
  invoke<string>("rename_project", { file, title });

/** Sets a project aside in Recently Deleted. Resolves to its manifest
 * there, which Undo restores. */
export const deleteProject = (file: string) =>
  invoke<string>("delete_project", { file });

/** Puts projects in Recently Deleted back where they came from. */
export const restoreProjects = (files: string[]) =>
  invoke<null>("restore_projects", { files });

/** Sends projects in Recently Deleted on to the system Trash now. */
export const trashProjects = (files: string[]) =>
  invoke<null>("trash_projects", { files });

/** Sends everything in Recently Deleted on to the system Trash. */
export const emptyRecentlyDeleted = () =>
  invoke<null>("empty_recently_deleted");

/** Sent while projects are moved or duplicated, with how far along. */
export const COPY_PROGRESS_EVENT = "projects://copy-progress";

/** Moves projects into one of the browser's locations. */
export const moveProjects = (files: string[], location: string) =>
  invoke<null>("move_projects", { files, location });

/** Copies each project beside itself. */
export const duplicateProjects = (files: string[]) =>
  invoke<null>("duplicate_projects", { files });

/** Takes a project off Recent, leaving it where it is. */
export const forgetRecentProject = (file: string) =>
  invoke<null>("forget_recent_project", { file });

/** Resolves to the folder added, or null when none was chosen. */
export const addProjectLocation = () =>
  invoke<string | null>("add_project_location");

export const removeProjectLocation = (path: string) =>
  invoke<null>("remove_project_location", { path });

export const hideProjectBrowser = () => invoke<null>("hide_project_browser");
