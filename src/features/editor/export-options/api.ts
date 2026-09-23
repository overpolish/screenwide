// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

import { EditorKind } from "../types";

/** What Rust tells an editor about its own options window. */
const EXPORT_OPTIONS_OPENED_EVENT = "export-options://opened";
const EXPORT_OPTIONS_CLOSED_EVENT = "export-options://closed";

type ExportOptionsVisibility = { kind: EditorKind };

/**
 * Opens the calling editor's export options window. Rust reads the workspace
 * off the calling window, so nothing has to be passed.
 */
export const showExportOptions = async () => {
  await invoke<null>("show_export_options");
};

/** Closes the calling workspace's export options, from the options window or
 * its editor, and returns to the editor. */
export const hideExportOptions = async () => {
  await invoke<null>("hide_export_options");
};

/** Fits this options window to the height its content asks for. */
export const resizeExportOptions = async (height: number) => {
  await invoke<null>("resize_export_options", { height });
};

const listenToVisibility = async (
  event: string,
  onVisibility: (kind: EditorKind) => void,
): Promise<UnlistenFn> =>
  listen<ExportOptionsVisibility>(event, ({ payload }) => {
    onVisibility(payload.kind);
  });

/**
 * Announced once the options window is on screen, to its editor and to the
 * options window itself - which uses it to restate its height, the request
 * that also reveals it.
 */
export const listenToExportOptionsOpened = async (
  onOpened: (kind: EditorKind) => void,
) => listenToVisibility(EXPORT_OPTIONS_OPENED_EVENT, onOpened);

/** Announced by every route that puts the options window away. */
export const listenToExportOptionsClosed = async (
  onClosed: (kind: EditorKind) => void,
) => listenToVisibility(EXPORT_OPTIONS_CLOSED_EVENT, onClosed);
