// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { scriptText } from "../script-words";

// A script's card shows the start of what it says, small, as the
// teleprompter shows it: as written, its Markdown dimmed, running off the
// card's foot under a fade.

export function ScriptCardPage({ markdown }: { markdown: string }) {
  let at = 0;
  return (
    <span
      aria-hidden
      className="absolute inset-0 overflow-hidden px-inset py-inset-sm text-start text-caption break-words whitespace-pre-wrap text-fg-secondary mask-b-from-60% mask-b-to-95%"
      dir="auto"
    >
      {scriptText(markdown).pieces.map((piece) => {
        const key = at;
        at += piece.text.length;
        return piece.kind === "mark" ? (
          <span className="text-fg-tertiary" key={key}>
            {piece.text}
          </span>
        ) : (
          <span key={key}>{piece.text}</span>
        );
      })}
    </span>
  );
}
