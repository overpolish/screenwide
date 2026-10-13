// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isEmptyLayout } from "./caption-layout";

import type { CaptionBreak } from "../../../../bindings/CaptionBreak";
import type { CaptionLayout } from "../../../../bindings/CaptionLayout";
import type { CaptionSettings } from "../../../../bindings/CaptionSettings";
import type { CaptionStyle } from "../../../../bindings/CaptionStyle";
import type { CaptionWordChoice } from "../../../../bindings/CaptionWordChoice";
import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** The twin of `CaptionSettings::default` in `screenwide_captions`. */
export const DEFAULT_CAPTION_SETTINGS: CaptionSettings = {
  censorProfanity: false,
  hideFillers: false,
  noGaps: false,
  profanity: "mask",
  removePunctuation: false,
};

/** The fewest and most words a caption may hold, the type size's range in
 * percent, and the weights a face is set at: the twins of the limits in
 * `screenwide_captions`. */
export const CAPTION_WORDS = { maximum: 20, minimum: 1 } as const;
export const CAPTION_SIZE_PERCENT = { maximum: 400, minimum: 25 } as const;

/** What captions start as where the edit has set nothing: the default
 * template's look, with the layout options kept for new captions, and the
 * default rules. */
export type CaptionDefaults = {
  layout: CaptionLayout;
  settings: CaptionSettings;
  style: CaptionStyle;
};

/** The edit's captions, starting from `defaults` for whatever it has not
 * set. */
export const recordingCaptions = (
  edit: RecordingTimelineEdit | null | undefined,
  defaults: CaptionDefaults,
): RecordingCaptions => {
  // Captions the edit has made keep the layout they were given, none set by
  // hand included; only captions not yet made start from the defaults.
  const layout = edit?.captions ? edit.captions.layout : defaults.layout;
  return {
    enabled: edit?.captions?.enabled ?? false,
    settings: {
      ...DEFAULT_CAPTION_SETTINGS,
      ...(edit?.captions?.settings ?? defaults.settings),
    },
    style: { ...defaults.style, ...edit?.captions?.style },
    ...(layout && !isEmptyLayout(layout) ? { layout } : {}),
    ...(edit?.captions?.breaks ? { breaks: edit.captions.breaks } : {}),
    ...(edit?.captions?.wordChoices
      ? { wordChoices: edit.captions.wordChoices }
      : {}),
  };
};

/** A change the Captions panel asks for: whether they show, what they say
 * and how they look, the layout options set by hand, and the breaks and
 * words placed by hand, each list and the layout whole. */
export type CaptionsPatch = {
  breaks?: CaptionBreak[];
  enabled?: boolean;
  layout?: CaptionLayout;
  settings?: Partial<CaptionSettings>;
  style?: Partial<CaptionStyle>;
  wordChoices?: CaptionWordChoice[];
};

/** `captions` with `patch` made to them. A list or layout left empty goes,
 * as the edit keeps none it does not need. */
export function patchCaptions(
  captions: RecordingCaptions,
  { breaks, layout, wordChoices, ...patch }: CaptionsPatch,
): RecordingCaptions {
  const patched: RecordingCaptions = {
    ...captions,
    enabled: patch.enabled ?? captions.enabled,
    settings: { ...captions.settings, ...patch.settings },
    style: { ...captions.style, ...patch.style },
  };
  if (breaks) patched.breaks = breaks;
  if (layout) patched.layout = layout;
  if (wordChoices) patched.wordChoices = wordChoices;
  if (patched.breaks?.length === 0) delete patched.breaks;
  if (patched.layout && isEmptyLayout(patched.layout)) delete patched.layout;
  if (patched.wordChoices?.length === 0) delete patched.wordChoices;
  return patched;
}

/** `edit` with `patch` made to its captions, which start from `defaults`
 * where it has none. */
export const withCaptions = (
  edit: RecordingTimelineEdit,
  patch: CaptionsPatch,
  defaults: CaptionDefaults,
): RecordingTimelineEdit => ({
  ...edit,
  captions: patchCaptions(recordingCaptions(edit, defaults), patch),
});
