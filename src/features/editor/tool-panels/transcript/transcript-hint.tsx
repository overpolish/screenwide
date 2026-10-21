// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { hotkeyKeys } from "../../../../composites/hotkey-field/hotkey";
import { t } from "../../../../i18n/i18n";
import { Keyboard } from "../../../../primitives/keyboard/keyboard";

/** Stands in for the keycap while the message is translated, so the whole
 * sentence stays one message and the key sits wherever a language puts it. */
const KEY_MARK = "\uE000";

/** The help line under the transcript, with the modifier a click takes drawn
 * as a key: Command on macOS, Ctrl on Windows. */
export function TranscriptHint() {
  const [before, after = ""] = t("editor-panels-transcript-hint", {
    key: KEY_MARK,
  })
    // Fluent isolates a placeable's direction; the keycap does that itself.
    .replace(`\u2068${KEY_MARK}\u2069`, KEY_MARK)
    .split(KEY_MARK);
  return (
    <p className="text-caption text-fg-secondary">
      {before}
      {hotkeyKeys("CommandOrControl").map((key) => (
        <Keyboard className="align-middle" key={key}>
          {key}
        </Keyboard>
      ))}
      {after}
    </p>
  );
}
