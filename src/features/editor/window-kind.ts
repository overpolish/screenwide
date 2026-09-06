// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { EditorKind } from "./types";

const EDITOR_LABELS: Record<string, EditorKind> = {
  "editor-recording": "recording",
  "editor-screenshot": "screenshot",
};

const EXPORT_LABELS: Record<string, EditorKind> = {
  "export-recording": "recording",
  "export-screenshot": "screenshot",
};

let resolvedEditor: EditorKind | null | undefined;
let resolvedExport: EditorKind | null | undefined;

/**
 * This webview's own label, or nothing outside the app: stories mount editor
 * components with no Tauri window behind them.
 */
function currentWindowLabel(): string | null {
  try {
    return getCurrentWindow().label;
  } catch {
    return null;
  }
}

/**
 * Which workspace this webview is showing, read off its own window label.
 *
 * Rust derives the same thing from the calling window, so no `invoke` has to
 * carry it. Memoized because a window's label never changes, and this is read
 * on every render of the editor window.
 */
export function currentEditorKind(): EditorKind | null {
  const label = currentWindowLabel();
  resolvedEditor ??= label === null ? null : (EDITOR_LABELS[label] ?? null);

  return resolvedEditor;
}

/** The same reading for an export options window, which has its own label. */
export function currentExportKind(): EditorKind | null {
  const label = currentWindowLabel();
  resolvedExport ??= label === null ? null : (EXPORT_LABELS[label] ?? null);

  return resolvedExport;
}
