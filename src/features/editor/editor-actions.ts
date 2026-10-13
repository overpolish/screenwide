// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One-shot editor window and export-destination commands.

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";

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

/** Closes this window's editor and sets its project aside in Recently
 * Deleted. */
export const deleteOpenProject = async () => {
  await invoke<null>("delete_open_project");
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

/** `act` on this editor's window, reporting what it could not do. */
const windowAction =
  (act: (window: Window) => Promise<void>, what: string) => () => {
    act(getCurrentWindow()).catch((cause: unknown) => {
      console.error(`Could not ${what} the editor window`, cause);
    });
  };

/** Asks to close the window rather than closing it outright: Rust owns the
 * close request, and that is where the confirmation is. */
export const closeEditorWindow = windowAction(
  (window) => window.close(),
  "close",
);
export const minimizeEditorWindow = windowAction(
  (window) => window.minimize(),
  "minimize",
);
export const toggleEditorWindowMaximized = windowAction(
  (window) => window.toggleMaximize(),
  "maximize or restore",
);
