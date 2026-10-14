// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { Caption } from "../../../../bindings/Caption";
import type { CaptionPreviewStrip } from "../../../../bindings/CaptionPreviewStrip";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../../../bindings/CaptionTemplate";
import type { CaptionTemplateFont } from "../../../../bindings/CaptionTemplateFont";
import type { CaptionTemplateId } from "../../../../bindings/CaptionTemplateId";
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

let templates: Promise<CaptionTemplate[]> | null = null;
let fontFamilies: Promise<FontFamily[]> | null = null;

/** Sent to every window when your templates change: one imported or
 * removed. From `src-tauri/src/editor/caption_template_files.rs`. */
export const CAPTION_TEMPLATES_EVENT = "editor://caption-templates";

/** Every caption template, the app's own first in the order the panel
 * offers them, then yours, each with what its manifest says of it. Kept
 * until [`CAPTION_TEMPLATES_EVENT`] says they changed. From
 * `src-tauri/src/editor/caption_styles.rs`. */
export const captionTemplates = () =>
  (templates ??= invoke<CaptionTemplate[]>("caption_templates").catch(
    (error: unknown) => {
      templates = null;
      throw error;
    },
  ));

/** Asks for them again the next time, as they have changed. */
export const forgetCaptionTemplates = () => {
  templates = null;
};

/** Calls `onChange` each time your templates change, until the returned
 * function is called. A story has no app to say they changed. */
export const followCaptionTemplates = (onChange: () => void) => {
  if (!isTauri()) return () => undefined;
  let following = true;
  let stop: (() => void) | undefined;
  listen(CAPTION_TEMPLATES_EVENT, onChange)
    .then((unlisten) => {
      if (following) stop = unlisten;
      else unlisten();
    })
    .catch((cause: unknown) => {
      console.error("Could not follow the caption templates", cause);
    });
  return () => {
    following = false;
    stop?.();
  };
};

/** The files of the faces already added to this window's fonts. */
const addedFonts = new Set<string>();

/** Adds the fonts your templates bring to this window's, so a family one
 * brings is named in its own type. The app's own are in its style sheet.
 * A face is only fetched once something is set in it. */
export const addCaptionTemplateFonts = async () => {
  if (!isTauri()) return;
  const fonts = await invoke<CaptionTemplateFont[]>("caption_template_fonts");
  for (const font of fonts) {
    if (addedFonts.has(font.path)) continue;
    addedFonts.add(font.path);
    document.fonts.add(
      new FontFace(font.family, `url("${convertFileSrc(font.path)}")`, {
        style: font.italic ? "italic" : "normal",
        weight: [Math.min(...font.weights), Math.max(...font.weights)].join(
          " ",
        ),
      }),
    );
  }
};

/** Asks for a template's zip or its folder's manifest and makes it one of
 * yours: the template, or nothing where none was imported. The app says
 * why in an alert where one could not be. */
export const importCaptionTemplate = () =>
  invoke<CaptionTemplate | null>("import_caption_template");

/** Asks where to save template `id` as a zip, and saves it there. */
export const exportCaptionTemplate = (id: CaptionTemplateId) =>
  invoke<null>("export_caption_template", { id });

/** Moves your template `id` to the Trash. */
export const removeCaptionTemplate = (id: CaptionTemplateId) =>
  invoke<null>("remove_caption_template", { id });

/** The font families installed on the system or brought by a template.
 * Reading the system's takes a moment, so the answer is kept until your
 * templates change. */
export const installedFontFamilies = () =>
  (fontFamilies ??= invoke<FontFamily[]>("installed_font_families").catch(
    (error: unknown) => {
      fontFamilies = null;
      throw error;
    },
  ));

/** Asks for the families again the next time, as your templates, and so
 * the families they bring, have changed. */
export const forgetFontFamilies = () => {
  fontFamilies = null;
};

/** Each of `styles` previewed saying the `sample` words, drawn by the
 * compositor: a strip of frames, in order. */
export const renderCaptionPreviews = (styles: CaptionStyle[], sample: string) =>
  invoke<CaptionPreviewStrip[]>("render_caption_previews", { sample, styles });
