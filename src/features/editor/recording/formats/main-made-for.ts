// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { frameSizePreset } from "../../tools/frame/frame-sizes";

import { laidDocument } from "./format-document";
import { removedPlatform } from "./format-sharing";
import { platformClearArea } from "./platform-clear-area";
import {
  formatsOfShape,
  formatsOrNew,
  isMainCanvas,
  keptFormats,
} from "./recording-formats";

import type { FormatDocument } from "./format-document";
import type { FrameFormats } from "../../../../bindings/FrameFormats";

// The main canvas can be made for a platform, as a recording made for
// TikTok alone is: chosen from the same templates the platforms are, it
// takes the template's size and keeps clear of the app's interface, so the
// phone and the app preview over it too. It stays the canvas sized freely:
// a size typed in, or the frame dragged to another, makes it a canvas made
// for none again (`madeForOf`).

type Changed = { document: FormatDocument; formats: FrameFormats | null };

/** The template the main canvas of `formats`, a canvas `size` large, is
 * made for: none once its size is no longer the template's. */
export const madeForOf = (
  formats: FrameFormats | null,
  size: { height: number; width: number },
) => {
  const template = frameSizePreset(formats?.list[0]?.madeFor);
  return template &&
    template.width === size.width &&
    template.height === size.height
    ? template.id
    : null;
};

/** `document` keeping clear of nothing. */
const unclear = (document: FormatDocument): FormatDocument => {
  const { clearArea: _was, ...primary } = document.recordingOutput.primary;
  return {
    ...document,
    recordingOutput: { ...document.recordingOutput, primary },
  };
};

/**
 * The main canvas, in hand, made for the template `preset`: a platform of
 * that template added beside it is taken away, as the main canvas is that
 * platform now; then it takes the template's size, its screen and camera
 * laid out on it clear of the app's interface, as a platform added is
 * (`laidDocument`). `preset` null makes it a canvas made for none, keeping
 * its size and keeping clear of nothing. Null where nothing changes.
 */
export const withMainMadeFor = (
  formats: FrameFormats | null,
  document: FormatDocument,
  preset: string | null,
): Changed | null => {
  if (!isMainCanvas(formats)) return null;
  const template = preset === null ? undefined : frameSizePreset(preset);
  if (!template) {
    if (!formats?.list[0]?.madeFor) return null;
    const [{ madeFor: _was, ...main }, ...rest] = formats.list;
    return {
      document: unclear(document),
      formats: keptFormats({ ...formats, list: [main, ...rest] }),
    };
  }
  const start = formatsOrNew(formats);
  const [main] = start.list;
  const isPlatform = start.list
    .slice(1)
    .some((shape) => formatsOfShape(shape).includes(template.id));
  const merged = isPlatform
    ? removedPlatform(start, document, template.id)
    : { document, formats: start };
  // Its last platform gone, the recording is one shape again, still the
  // same main canvas.
  const left = merged.formats ?? { active: main.id, list: [{ id: main.id }] };
  const [, ...platforms] = left.list;
  return {
    document: laidDocument(
      merged.document,
      template,
      platformClearArea([template.id]),
    ),
    formats: keptFormats({
      ...left,
      list: [{ ...left.list[0], madeFor: template.id }, ...platforms],
    }),
  };
};
