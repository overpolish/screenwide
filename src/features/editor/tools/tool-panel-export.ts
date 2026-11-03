// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CameraKey } from "../../../bindings/CameraKey";
import type { ExportPreviewCanvas } from "../../../bindings/ExportPreviewCanvas";
import type { ExportSubtitles } from "../../../bindings/ExportSubtitles";
import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";
import type { CameraOutput } from "../export/camera-output";
import type { ExportLook } from "../export/export-looks";
import type { ExportPhase } from "../export/use-export-progress";

/** One of the four looks, with what it makes: the screen's size, and the
 * file's estimated size once measured. */
export type ToolPanelExportLook = {
  /** In bytes; null until measured, or where it cannot be. */
  bytes: number | null;
  height: number;
  id: ExportLook;
  width: number;
};

/** How good the video comes out: one of four looks. */
export type ToolPanelExportQuality = {
  /** The look the settings match; null for the moment before a look is
   * set where they matched none. */
  look: ExportLook | null;
  looks: ToolPanelExportLook[];
};

/** A picture compared, at its own pixel size, as the island's window
 * loads it. */
export type ToolPanelComparedFrame = {
  /** Where the look's canvas lies in the picture, in its pixels; the two
   * pictures are lined up by it, an export drawn smaller fitting the canvas
   * at one scale into sides rounded to even. */
  canvas: ExportPreviewCanvas;
  height: number;
  src: string;
  width: number;
};

/** The picture as it comes out of the export beside it as drawn: a
 * recording's frame at the playhead at the chosen look, or a screenshot as
 * its JPEG or WebP. */
export type ToolPanelExportCompare = {
  /** As the export writes it; null until made. */
  exported: ToolPanelComparedFrame | null;
  /** The latest pictures could not be made. */
  failed: boolean;
  /** Newer pictures are being made; the ones held are of earlier
   * settings. */
  isUpdating: boolean;
  /** As drawn, before any encoder; null until made. */
  original: ToolPanelComparedFrame | null;
  /** The screenshot has see-through areas, which its JPEG fills with
   * white. Always false for a recording. */
  seeThrough: boolean;
};

/** The save under way. */
export type ToolPanelExportProgress = {
  /** Whether it can be stopped: a screenshot is written in one go. */
  canCancel: boolean;
  /** How long it still has, in seconds, once that can be measured. */
  etaSeconds: number | null;
  isCanceling: boolean;
  /** Whole percent, or null while the save reports no figure. */
  percent: number | null;
  phase: ExportPhase;
};

/**
 * What the Export island shows, published by the editor while it is out. Its
 * options exist only where the export holds what they act on: null where
 * there is nothing to choose.
 */
export type ToolPanelExport = {
  /** A recording's camera on its own, with its background taken away: what
   * fills the room. */
  cameraKey: CameraKey | null;
  /** One video or separate files, with a screen and a camera both in. */
  cameraOutput: CameraOutput | null;
  canExport: boolean;
  /** Two audio tracks or more in: whether they are mixed into one. */
  collapseAudio: boolean | null;
  /** Null where there is no picture to compare: audio, or a screenshot
   * saved losslessly as a PNG. */
  compare: ToolPanelExportCompare | null;
  deleteProjectAfterExport: boolean;
  /** Where the file is written; null before a folder is known. */
  directory: string | null;
  /** The export the last save wrote, while the island shows it done. */
  exported: { bytes: number; name: string } | null;
  /** The file's extension, which its format names. */
  extension: string;
  /** Why the last save failed, in the system's words. */
  failure: string | null;
  fileStem: string;
  kind: "audio" | "screenshot" | "video";
  /** The save under way, or null while there is none. */
  progress: ToolPanelExportProgress | null;
  /** Null for a screenshot, audio, or where compression is not available. */
  quality: ToolPanelExportQuality | null;
  /** Whether the file manager shows the file once it is written: a general
   * setting, which every export shares. */
  revealWhenDone: boolean;
  /** A screenshot's picture format; null for a recording. */
  screenshotFormat: ScreenshotFormat | null;
  /** The subtitle file beside the video, made of the transcript; null while
   * there is no transcript to make it of. */
  subtitles: ExportSubtitles | null;
};

/** What the Export island asks the editor for: a setting at a time, or a
 * step of the export. */
export type ToolPanelExportPatch = Partial<
  Pick<
    ToolPanelExport,
    "deleteProjectAfterExport" | "fileStem" | "revealWhenDone" | "subtitles"
  >
> & {
  /** Choose where the file is written. */
  browse?: true;
  cameraKey?: CameraKey;
  cameraOutput?: CameraOutput;
  /** Stop the save under way, which throws the partial file away. */
  cancel?: true;
  collapseAudio?: boolean;
  /** Put the island away: before the export, after it, never during it. */
  dismiss?: true;
  /** Write the file. */
  export?: true;
  look?: ExportLook;
  screenshotFormat?: ScreenshotFormat;
};
