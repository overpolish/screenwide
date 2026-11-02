// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../../../i18n/i18n";
import {
  type HotkeyModifier,
  hotkeyKey,
  hotkeyKeys,
  hotkeyValue,
  isGlobalChord,
} from "../../../../lib/hotkey";
import { keyLabel } from "../../../../primitives/keyboard/key-label";

import type { ShortcutConflict } from "../shortcut-conflicts";

/** A key pressed alone, and the modifiers chosen to go with it. */
export type ShortcutPick = {
  code: string;
  /** Combinations held down that clashed, kept in view among the offers. */
  held: HotkeyModifier[][];
  modifiers: HotkeyModifier[];
};

export type PickStatus = { text: string; tone: "free" | "warning" };

/** What taking the combination in hand would do, in a line. */
export function pickStatus(
  pick: ShortcutPick,
  conflict: ShortcutConflict | null,
): PickStatus {
  const warning = (text: string): PickStatus => ({ text, tone: "warning" });
  if (!isGlobalChord(pick.modifiers))
    return warning(
      t("hotkey-field-pick-modifier", {
        key: keyLabel(hotkeyKey(pick.code)),
        platform: platformArg(),
      }),
    );
  if (!conflict) return { text: t("hotkey-field-pick-free"), tone: "free" };
  if (conflict.kind === "taken")
    return warning(t("hotkey-field-pick-taken", { action: conflict.label }));
  const keys = hotkeyKeys(hotkeyValue(pick.code, pick.modifiers))
    .map(keyLabel)
    .join(" + ");
  return warning(
    conflict.by === "system"
      ? t("hotkey-field-pick-reserved-system", {
          keys,
          platform: platformArg(),
          use: conflict.label,
        })
      : t("hotkey-field-pick-reserved-apps", { keys, use: conflict.label }),
  );
}
