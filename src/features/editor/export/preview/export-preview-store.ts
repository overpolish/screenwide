// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createWorkspaceMirror } from "../../workspace-mirror";

import type { ExportPreviewFrame } from "../../../../bindings/ExportPreviewFrame";

/** The frame the export window compares, as the editor last made it. */
export type ExportPreview = {
  /** Whether the export has a picture to preview at all. */
  available: boolean;
  /** The frame as the export will write it. */
  exported: ExportPreviewFrame | null;
  /** The latest frames could not be made. */
  failed: boolean;
  /** Newer frames are being made; the ones held are of earlier settings. */
  isUpdating: boolean;
  /** The same frame at Original size and compression. */
  original: ExportPreviewFrame | null;
};

/** Published by the editor, which owns the playhead and the settings, and
 * read by the options window. Kept apart from the options mirror, which
 * the options form republishes on every change. */
const mirror = createWorkspaceMirror<ExportPreview>(
  "screenwide-export-preview",
);

export const useExportPreviewStore = mirror.useMirror;
export const synchronizeExportPreviewStore = mirror.synchronize;
