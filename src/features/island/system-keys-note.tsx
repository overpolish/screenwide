// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";
import { splitAtPlaceable } from "../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../lib/hotkey";
import { cn } from "../../lib/styling";
import { Keyboard, Shortcut } from "../../primitives/keyboard/keyboard";
import { Text } from "../../primitives/text/text";

/** A hint on the island, said once: the system's keys just pressed now open
 * Screenwide, and where to give them back. On a line under the row it wraps
 * at a reading width rather than widening the island to one line; in a
 * section it wraps at the row's width, starting on the text column. */
export function SystemKeysNote({
  keys,
  layout = "line",
}: {
  keys: string;
  layout?: "line" | "section";
}) {
  const [before, after] = splitAtPlaceable((mark) =>
    t("settings-system-keys-announce", { keys: mark }),
  );
  return (
    <Text
      as="p"
      className={cn(
        "text-fg-secondary",
        layout === "line" ? "w-max max-w-80 px-inset" : "px-inset-sm",
      )}
      role="status"
      variant="caption"
    >
      {before}
      <Shortcut className="align-middle">
        {hotkeyKeys(keys).map((key) => (
          <Keyboard key={key}>{key}</Keyboard>
        ))}
      </Shortcut>
      {after}
    </Text>
  );
}
