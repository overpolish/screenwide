// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// One-shot editor window and export-destination commands.

import { invoke } from "@tauri-apps/api/core";

import {
  normalizedScreenshotWorkspaceOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import { EditorKind } from "../types";

export const copyEditorToClipboard = async (
  screenshotOutput: ScreenshotWorkspaceOutputSettings,
) => {
  await invoke<null>("copy_editor_to_clipboard", {
    screenshotOutput: normalizedScreenshotWorkspaceOutput(screenshotOutput),
  });
};

export const setScreenshotRadius = async (radiusPercent: number) => {
  await invoke<null>("set_screenshot_radius", { radiusPercent });
};

export const setScreenshotBackgroundRadius = async (radiusPercent: number) => {
  await invoke<null>("set_screenshot_background_radius", { radiusPercent });
};

export const discardEditor = async () => {
  await invoke<null>("discard_editor");
};

export const cancelExportJob = () => invoke<boolean>("cancel_export_job");

/** Named explicitly: the recording bar asks on another window's behalf. */
export const focusEditorWindow = async (kind: EditorKind) => {
  await invoke<null>("focus_editor_window", { kind });
};

export const browseExportDirectory = () =>
  invoke<string | null>("browse_export_directory");

export const setExportDirectory = async (directory: string) => {
  await invoke<null>("set_export_directory", { directory });
};
