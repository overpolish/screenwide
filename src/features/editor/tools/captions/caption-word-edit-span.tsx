// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ComponentProps } from "react";

import { TranscriptEditSpan } from "../transcript/transcript-edit-span";

import { captionWordParts } from "./caption-word-parts";

/** A word being edited in place: the word alone in the edit field, its
 * punctuation either side kept out of it, as the edit keeps it. */
export function CaptionWordEditSpan({
  text,
  ...edit
}: Omit<ComponentProps<typeof TranscriptEditSpan>, "className">) {
  const { after, before, word } = captionWordParts(text);
  return (
    <>
      {before}
      <TranscriptEditSpan
        {...edit}
        className="outline-none select-text"
        text={word}
      />
      {after}
    </>
  );
}
