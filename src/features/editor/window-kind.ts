// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { EditorKind } from "./types";

const LABELS: Record<string, EditorKind> = {
  "editor-recording": "recording",
  "editor-screenshot": "screenshot",
};

let resolved: EditorKind | null | undefined;

/**
 * Which workspace this webview is showing, read off its own window label.
 *
 * Rust derives the same thing from the calling window, so no `invoke` has to
 * carry it. Memoized because a window's label never changes, and this is read
 * on every render of the editor window.
 */
export function currentEditorKind(): EditorKind | null {
  resolved ??= LABELS[getCurrentWindow().label] ?? null;

  return resolved;
}
