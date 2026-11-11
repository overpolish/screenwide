// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Ref, useState } from "react";

import { platformArg, t } from "../../../i18n/i18n";
import {
  type HotkeyModifier,
  hotkeyKey,
  hotkeyKeys,
  parseHotkey,
} from "../../../lib/hotkey";
import { isWindows } from "../../../lib/platform";
import { keyLabel } from "../../../primitives/keyboard/key-label";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { SettingsPane } from "../panes/settings-pane";

import { KeyDetail, type MapNotice } from "./keyboard-map/key-detail";
import { KeyboardMap } from "./keyboard-map/keyboard-map";
import { useHeldModifiers } from "./keyboard-map/use-held-modifiers";
import { assignFromMap } from "./map-assignment";
import { clashes, conflictOf, type ShortcutScope } from "./shortcut-conflicts";
import { shortcutEntries, type ShortcutSources } from "./shortcut-entries";
import { ShortcutList } from "./shortcut-list";
import { preferredModifiers } from "./shortcut-suggestions";
import { SystemKeysGroup } from "./system-keys-group";

import type { KeyDef } from "./keyboard-map/keyboard-layout";

// Shortcuts: the keyboard above, every shortcut below. The map shows one
// scope at a time, since a tool's own keys only work while it is open and
// never meet the ones that work anywhere; choosing a shortcut in the list
// shows its scope. Both halves show the same shortcuts: pointing at a row
// rings its key, clicking a key chooses its row, and a chosen row takes a
// new key from a click on the map.

const keysText = (value: string) => hotkeyKeys(value).map(keyLabel).join(" + ");

export function ShortcutsPane({
  onCaptureChange,
  paneRef,
  revealing,
  ...sources
}: ShortcutSources & {
  onCaptureChange: (capturing: boolean) => Promise<void>;
  paneRef?: Ref<HTMLElement>;
  /** The title of the row a search result is bringing into view. */
  revealing?: string | null;
}) {
  const isMac = !isWindows();
  const entries = shortcutEntries(sources);
  // Opened on a shortcut's row, by a search result or a moment's keys, the
  // pane starts on that shortcut's scope, so its row is in the list.
  const [scope, setScope] = useState<ShortcutScope>(
    () =>
      entries.find((entry) => entry.label === revealing)?.scope ?? "anywhere",
  );
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [pointedId, setPointedId] = useState<string | null>(null);
  const [hoveredKey, setHoveredKey] = useState<string | null>(null);
  const [notice, setNotice] = useState<MapNotice | null>(null);
  // The key the map last gave a shortcut, which pulses there once.
  const [assigned, setAssigned] = useState<{
    code: string;
    count: number;
  } | null>(null);
  const [latched, setLatched] = useState<ReadonlySet<HotkeyModifier>>(
    () => new Set(scope === "anywhere" ? preferredModifiers(isMac) : []),
  );

  const clashing = clashes(entries, isMac);
  // The scope chosen shows its own shortcuts, on the map and in the list.
  const shown = entries.filter((entry) => entry.scope === scope);
  const selected = entries.find((entry) => entry.id === selectedId) ?? null;
  const conflictFor = (entry: (typeof entries)[number]) => (value: string) =>
    conflictOf({ isMac, shortcuts: entries, subject: entry, value });

  // Choosing a shortcut shows its scope and latches its modifiers, so its
  // key is lit to be seen.
  const select = (id: string | null) => {
    setSelectedId(id);
    setNotice(null);
    const entry = entries.find((candidate) => candidate.id === id);
    if (!entry) return;
    setScope(entry.scope);
    const parsed =
      entry.kind === "combo" ? parseHotkey(entry.value, isMac) : null;
    if (parsed) setLatched(new Set(parsed.modifiers));
  };
  useHeldModifiers({
    onEscape: () => {
      select(null);
    },
    onHeld: setLatched,
  });

  const pressKey = (key: KeyDef) => {
    if (!selected || selected.isDisabled) {
      const bound = shown.find((entry) => {
        const parsed = parseHotkey(entry.value, isMac);
        return (
          parsed?.code === key.code &&
          parsed.modifiers.length === latched.size &&
          parsed.modifiers.every((modifier) => latched.has(modifier))
        );
      });
      select(bound?.id ?? null);
      return;
    }
    const result = assignFromMap({
      code: key.code,
      conflictOf: conflictFor(selected),
      isMac,
      latched: [...latched],
      shortcut: selected,
    });
    if (result.kind === "assign") {
      selected.onChange(result.value);
      setAssigned({ code: key.code, count: (assigned?.count ?? 0) + 1 });
      if (selected.kind === "combo") setLatched(new Set(result.modifiers));
      setNotice({
        text: t("settings-shortcuts-assigned", {
          keys: keysText(result.value),
          shortcut: selected.label,
        }),
        tone: "done",
      });
    } else if (result.kind === "refuse") {
      const keys = keysText(result.value);
      setNotice({
        text:
          result.conflict.kind === "taken"
            ? t("settings-shortcuts-refused-taken", {
                action: result.conflict.label,
                keys,
              })
            : result.conflict.by === "system"
              ? t("settings-shortcuts-refused-reserved-system", {
                  keys,
                  platform: platformArg(),
                  use: result.conflict.label,
                })
              : t("settings-shortcuts-refused-reserved-apps", {
                  keys,
                  use: result.conflict.label,
                }),
        tone: "refused",
      });
    } else
      setNotice({
        text: t("settings-shortcuts-refused-none", {
          key: keyLabel(hotkeyKey(key.code)),
        }),
        tone: "refused",
      });
  };

  return (
    <SettingsPane
      label={t("settings-section-shortcuts")}
      paneRef={paneRef}
      stage={
        <div className="flex flex-col gap-inline">
          <PillGroup
            aria-label={t("settings-shortcuts-scope")}
            className="self-start"
            items={[
              { id: "anywhere", label: t("settings-shortcuts-scope-anywhere") },
              { id: "ruler", label: t("settings-shortcuts-scope-ruler") },
              { id: "ocr", label: t("settings-shortcuts-scope-ocr") },
            ]}
            onSelectionChange={(next) => {
              setScope(next as ShortcutScope);
              setSelectedId(null);
              setNotice(null);
              setLatched(
                new Set(next === "anywhere" ? preferredModifiers(isMac) : []),
              );
            }}
            selected={scope}
          />
          <KeyboardMap
            assigned={assigned}
            clashing={new Set(clashing.keys())}
            entries={shown}
            isMac={isMac}
            latched={latched}
            onHoverKey={setHoveredKey}
            onPressKey={pressKey}
            onToggleModifier={(modifier) => {
              const next = new Set(latched);
              if (next.has(modifier)) next.delete(modifier);
              else next.add(modifier);
              setLatched(next);
            }}
            pointedId={pointedId}
            scope={scope}
            selectedId={selectedId}
          />
          <KeyDetail
            entries={shown}
            hoveredKey={hoveredKey}
            isMac={isMac}
            latched={latched}
            notice={notice}
            scope={scope}
            selected={selected}
          />
        </div>
      }
    >
      {scope === "anywhere" &&
      (sources.defaults?.systemShortcuts.length ?? 0) > 0 ? (
        <SystemKeysGroup
          entries={shown.filter((entry) => entry.isSystem)}
          isOn={sources.shortcuts?.replaceSystemKeys ?? false}
          isSaving={!sources.shortcuts || sources.savingSystemKeys}
          onChange={sources.onChangeReplaceSystemKeys}
          onPoint={setPointedId}
        />
      ) : null}
      <ShortcutList
        clashes={clashing}
        conflictOf={conflictFor}
        entries={shown.filter((entry) => !entry.isSystem)}
        onCaptureChange={onCaptureChange}
        onPoint={setPointedId}
        onSelect={select}
        selectedId={selectedId}
      />
    </SettingsPane>
  );
}
