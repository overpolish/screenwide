// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";

import { READING_LINE, readingLineOffset } from "../reading-line";

import { type CardCue, type CardPlace } from "./cue-card";
import { columnMargin, MARGIN_MIN } from "./cue-room";
import { useCueBeside } from "./use-cue-beside";

import type { ShownTeleprompterCue } from "./cue-api";
import type { TextFrame } from "../use-text-frame";

/**
 * Where the cue being read towards stands: in the margin the text column
 * leaves at its end, level with the words being read, where it leaves
 * room; else in its window beside the teleprompter, on its end side and
 * level with them too; else, with no room beside, at the top of the text.
 * The cue the teleprompter draws itself, and where, for the first and last.
 */
export function useCuePlace(
  cue: ShownTeleprompterCue | null,
  {
    columnWidth,
    isMirrored,
    textSize,
  }: { columnWidth: number | null; isMirrored: boolean; textSize: number },
) {
  const [frame, setFrame] = useState<TextFrame | null>(null);
  const margin = frame ? columnMargin(frame.width, columnWidth) : 0;
  const isInMargin = margin >= MARGIN_MIN;
  const isBeside = useCueBeside(isInMargin ? null : cue, isMirrored, {
    endIsLeft: document.documentElement.dir === "rtl",
    frameTop: frame?.top ?? 0,
    offset: readingLineOffset(textSize),
    share: READING_LINE,
  });
  const drawn: CardCue | null =
    cue && (isInMargin || !isBeside)
      ? { id: cue.id, src: convertFileSrc(cue.path), title: cue.title }
      : null;
  const place: CardPlace = isInMargin
    ? { kind: "margin", textSize, width: margin }
    : { kind: "top" };
  return { drawn, onTextFrameChange: setFrame, place };
}
