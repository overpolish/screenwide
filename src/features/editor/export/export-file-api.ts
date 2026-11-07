// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { EditorKind } from "../types";

// The finished export's file, acted on by Rust from the file it kept: never
// a path the page names. Reveal, Copy and the drag are asked from the Export
// island's window, which the drag starts from.

/** Puts the finished island away. The editor stays on the project, unless
 * the export moved it to Recently Deleted. */
export const putAwayExport = () => invoke<null>("put_away_export");

/** Lets go of the compared pictures still being made, once the island is
 * away. */
export const stopExportPreviews = () => invoke<null>("stop_export_previews");

export const revealExportFile = (workspace: EditorKind) =>
  invoke<null>("reveal_export_file", { workspace });

export const copyExportFile = (workspace: EditorKind) =>
  invoke<null>("copy_export_file", { workspace });

/** Starts dragging the file out of this window, from the press under way. */
export const dragExportFile = (workspace: EditorKind) =>
  invoke<null>("drag_export_file", { workspace });
