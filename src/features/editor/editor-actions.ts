// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One-shot editor window and export-destination commands.

import { invoke } from "@tauri-apps/api/core";

import {
  normalizedScreenshotWorkspaceOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot/screenshot-output";

/** Copies the screenshot and closes the editor; its project goes to the Trash
 * after when `deleteProjectAfterExport` says so. */
export const copyEditorToClipboard = async (
  screenshotOutput: ScreenshotWorkspaceOutputSettings,
  deleteProjectAfterExport: boolean,
) => {
  await invoke<null>("copy_editor_to_clipboard", {
    deleteProjectAfterExport,
    screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
  });
};

/** Renames the project open in this window's editor, which then opens it
 * again under its new name. */
export const renameOpenProject = async (title: string) => {
  await invoke<null>("rename_open_project", { title });
};

export const setScreenshotRadius = async (radiusPercent: number) => {
  await invoke<null>("set_screenshot_radius", { radiusPercent });
};

export const setScreenshotBackgroundRadius = async (radiusPercent: number) => {
  await invoke<null>("set_screenshot_background_radius", { radiusPercent });
};

export const cancelExportJob = () => invoke<boolean>("cancel_export_job");

export const browseExportDirectory = () =>
  invoke<string | null>("browse_export_directory");

/** Choose a picture to put behind the canvas. Null when nothing was chosen. */
export const browseBackgroundImage = () =>
  invoke<string | null>("browse_background_image");

export const setExportDirectory = async (directory: string) => {
  await invoke<null>("set_export_directory", { directory });
};
