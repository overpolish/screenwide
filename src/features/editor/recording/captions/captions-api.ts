// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { Caption } from "../../../../bindings/Caption";
import type { CaptionPreviewStrip } from "../../../../bindings/CaptionPreviewStrip";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { FontFamily } from "../../../../bindings/FontFamily";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** Makes the captions `edit` asks of the recording open in the editor, set in
 * a `width` by `height` canvas: the preview draws them from then on while
 * they are on. The answer holds them on or off, and none without a
 * transcript. From `src-tauri/src/editor/captions.rs`. */
export const makeRecordingCaptions = (
  artifactId: number,
  edit: RecordingTimelineEdit,
  { height, width }: { height: number; width: number },
) =>
  invoke<Caption[] | null>("make_recording_captions", {
    artifactId,
    edit,
    height: Math.max(1, Math.round(height)),
    width: Math.max(1, Math.round(width)),
  });

/** Sent to every window each time the captions are made again, naming the
 * recording they are of. */
export const RECORDING_CAPTIONS_EVENT = "editor://recording-captions";

/** The captions last made of the recording `artifactId`, on or off: what
 * the Captions panel lays its text out from, in its own window. */
export const recordingCaptions = (artifactId: number) =>
  invoke<Caption[]>("recording_captions", { artifactId });

let builtInTemplates: Promise<CaptionStyle[]> | null = null;

/** Every caption template as it ships, in the order the panel offers them.
 * They never change while the app runs, so they are asked for once. From
 * `src-tauri/src/editor/caption_styles.rs`. */
export const captionTemplateStyles = () =>
  (builtInTemplates ??= invoke<CaptionStyle[]>("caption_templates").catch(
    (error: unknown) => {
      builtInTemplates = null;
      throw error;
    },
  ));

let fontFamilies: Promise<FontFamily[]> | null = null;

/** The font families installed on the system. Reading them takes a moment,
 * so the answer is kept for the window's life. */
export const installedFontFamilies = () =>
  (fontFamilies ??= invoke<FontFamily[]>("installed_font_families").catch(
    (error: unknown) => {
      fontFamilies = null;
      throw error;
    },
  ));

/** Each of `styles` previewed saying the `sample` words, drawn by the
 * compositor: a strip of frames, in order. */
export const renderCaptionPreviews = (styles: CaptionStyle[], sample: string) =>
  invoke<CaptionPreviewStrip[]>("render_caption_previews", { sample, styles });
