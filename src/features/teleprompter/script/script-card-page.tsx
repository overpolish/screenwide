// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

// A script's card shows the start of what it says, small, as the
// teleprompter shows it, running off the card's foot under a fade.

export function ScriptCardPage({ text }: { text: string }) {
  return (
    <span
      aria-hidden
      className="absolute inset-0 overflow-hidden px-inset py-inset-sm text-start text-caption break-words whitespace-pre-wrap text-fg-secondary mask-b-from-60% mask-b-to-95%"
      dir="auto"
    >
      {text}
    </span>
  );
}
