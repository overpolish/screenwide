// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { ShortcutSources } from "./shortcut-entries";
import type { ShortcutEntry } from "./shortcut-entry";

/** The system's keys, while they run Screenwide's tools, each named after
 * the tool among `tools` it opens: never offered to another shortcut, and
 * shown on the map as the tool's. Clearing one gives them all back. */
export function systemKeyEntries(
  tools: readonly ShortcutEntry[],
  sources: Pick<
    ShortcutSources,
    "defaults" | "onChangeReplaceSystemKeys" | "shortcuts"
  >,
): ShortcutEntry[] {
  if (!sources.shortcuts?.replaceSystemKeys) return [];
  const taken = sources.defaults?.systemShortcuts ?? [];
  return taken.flatMap(({ action, shortcut }) => {
    const tool = tools.find((entry) => entry.id === action);
    if (!tool) return [];
    return {
      captureMode: "shortcut",
      group: t("settings-shortcuts-system-keys"),
      icon: tool.icon,
      id: `system ${shortcut}`,
      isClearable: false,
      isDisabled: true,
      isOff: tool.isOff,
      isSystem: true,
      kind: "combo",
      label: tool.label,
      onChange: (value) => {
        if (value === null) sources.onChangeReplaceSystemKeys(false);
      },
      scope: "anywhere",
      value: shortcut,
    } satisfies ShortcutEntry;
  });
}
