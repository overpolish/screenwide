// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { ProjectLocation, ProjectSource, ProjectSummary } from "./types";

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

export const trashProject = (file: string) =>
  invoke<null>("trash_project", { file });

/** Takes a project off Recent, leaving it where it is. */
export const forgetRecentProject = (file: string) =>
  invoke<null>("forget_recent_project", { file });

/** Resolves to the folder added, or null when none was chosen. */
export const addProjectLocation = () =>
  invoke<string | null>("add_project_location");

export const removeProjectLocation = (path: string) =>
  invoke<null>("remove_project_location", { path });

export const hideProjectBrowser = () => invoke<null>("hide_project_browser");
