// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "../../../../components/base/keyboard/keyboard";
import { Text } from "../../../../components/base/text/text";
import { hotkeyKeys } from "../../../../components/shared/hotkey-field/hotkey";
import { t } from "../../../../i18n/i18n";

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
    <Text variant="footnote">
      {before}
      {hotkeyKeys("CommandOrControl").map((key) => (
        <Keyboard className="align-middle" key={key}>
          {key}
        </Keyboard>
      ))}
      {after}
    </Text>
  );
}
