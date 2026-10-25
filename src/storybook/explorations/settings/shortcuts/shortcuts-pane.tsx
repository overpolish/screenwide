// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { useSettingsApi } from "../../../../features/settings/settings-api-context";
import { useShortcutCapture } from "../../../../features/settings/use-shortcut-capture";
import { SearchField } from "../../../../primitives/input-fields/search-field";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Text } from "../../../../primitives/text/text";
import { Pane } from "../panes/pane";

import { KeyDetail, type Notice } from "./key-detail";
import { KeyboardMap } from "./keyboard-map";
import { shortcutEntries } from "./shortcut-entries";
import { ShortcutList } from "./shortcut-list";
import {
  clashes,
  comboText,
  comboValue,
  heldText,
  type Modifier,
  MODIFIERS,
  parseCombo,
  RESERVED,
  sameMods,
  type Scope,
} from "./shortcut-model";
import { useHeldKeys } from "./use-held-keys";

import type { ExploredSettings } from "../settings-state";

// Shortcuts: the keyboard above, the list below, one scope at a time. Keys
// of a tool only work while it is open, so they get a scope of their own
// rather than crowding the keys that work anywhere. Both halves show the
// same shortcuts: pointing at a row lights its key, clicking a key picks
// its row, and a chosen row takes new keys from a click on the map.

const SCOPES: { id: Scope; label: string }[] = [
  { id: "anywhere", label: "Anywhere" },
  { id: "ruler", label: "In Ruler" },
  { id: "ocr", label: "In OCR" },
];

export function ShortcutsPane({ settings }: { settings: ExploredSettings }) {
  const api = useSettingsApi();
  const onCaptureChange = useShortcutCapture(
    api.beginShortcutCapture,
    api.endShortcutCapture,
  );
  const [scope, setScope] = useState<Scope>("anywhere");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [previewId, setPreviewId] = useState<string | null>(null);
  const [hoveredKey, setHoveredKey] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [query, setQuery] = useState("");
  const [held, setHeld] = useState<ReadonlySet<Modifier>>(
    () => new Set(["shift", "cmd"]),
  );

  const entries = shortcutEntries(settings).filter(
    (entry) => entry.scope === scope,
  );
  const clashMap = clashes(entries);
  const needle = query.trim().toLowerCase();
  const shown = needle
    ? entries.filter((entry) =>
        `${entry.label} ${entry.group}`.toLowerCase().includes(needle),
      )
    : entries;
  const selected = entries.find((entry) => entry.id === selectedId) ?? null;
  const reserved = scope === "anywhere" ? RESERVED : [];
  const boundTo = (key: string, mods: ReadonlySet<Modifier>) =>
    entries.find((entry) => {
      const combo = parseCombo(entry.value);
      return (
        entry.kind === "combo" &&
        combo?.key === key &&
        sameMods(combo.mods, mods)
      );
    });

  // Choosing a shortcut holds its modifiers, so its key is lit to be seen.
  const select = (id: string | null) => {
    setSelectedId(id);
    setNotice(null);
    const combo = parseCombo(
      entries.find((entry) => entry.id === id && entry.kind === "combo")
        ?.value ?? null,
    );
    if (combo) setHeld(new Set(combo.mods));
  };
  const pressedKey = useHeldKeys({
    onEscape: () => {
      select(null);
    },
    onHeldChange: setHeld,
    onLookUp: (key, mods) => {
      const found = boundTo(key, mods);
      if (!found) return false;
      select(found.id);
      document
        .querySelector(`[data-entry-id="${found.id}"]`)
        ?.scrollIntoView({ behavior: "smooth", block: "nearest" });
      return true;
    },
  });

  const pressKey = (key: string) => {
    const bound = boundTo(key, held);
    if (!selected || bound?.id === selected.id) {
      select(bound && bound.id !== selected?.id ? bound.id : null);
      return;
    }
    if (selected.kind === "hold") return;
    const combo = {
      key,
      mods: MODIFIERS.filter((modifier) => held.has(modifier)),
    };
    if (
      scope === "anywhere" &&
      !combo.mods.some((modifier) => modifier !== "shift")
    ) {
      setNotice({
        text: `Hold ⌘, ⌥ or ⌃ as well, so ${heldText(held)}${comboText({ key, mods: [] })} still types as usual.`,
        tone: "refused",
      });
      return;
    }
    const taken = reserved.find(
      (candidate) => candidate.key === key && sameMods(candidate.mods, held),
    );
    if (taken) {
      setNotice({
        text: `${comboText(taken)} is ${taken.label} in ${taken.by === "macOS" ? "macOS" : "most apps"}. Choose another key.`,
        tone: "refused",
      });
      return;
    }
    selected.onChange(comboValue(combo));
    setNotice({
      text: `${selected.label} is now ${comboText(combo)}.`,
      tone: "done",
    });
  };

  const toggleModifier = (modifier: Modifier) => {
    const next = new Set(held);
    if (next.has(modifier)) next.delete(modifier);
    else next.add(modifier);
    setHeld(next);
  };

  return (
    <Pane
      label="Shortcuts"
      stage={
        <div className="flex flex-col gap-inline">
          <div className="flex items-center justify-between gap-separate">
            <PillGroup
              aria-label="Where the shortcuts work"
              items={SCOPES}
              onSelectionChange={(next) => {
                setScope(next as Scope);
                select(null);
                setHeld(new Set(next === "anywhere" ? ["shift", "cmd"] : []));
              }}
              selected={scope}
            />
            <SearchField
              aria-label="Find a shortcut"
              className="w-44"
              onChange={setQuery}
              placeholder="Find a shortcut"
              value={query}
            />
          </div>
          <KeyboardMap
            clashing={new Set(clashMap.keys())}
            entries={entries}
            held={held}
            matches={needle ? new Set(shown.map((entry) => entry.id)) : null}
            onHoverKey={setHoveredKey}
            onPressKey={pressKey}
            onToggleModifier={toggleModifier}
            pressedKey={pressedKey}
            previewId={previewId}
            reserved={reserved}
            selectedId={selectedId}
          />
          <KeyDetail
            entries={entries}
            held={held}
            hoveredKey={hoveredKey}
            notice={notice}
            scope={scope}
            selected={selected}
          />
        </div>
      }
    >
      {shown.length === 0 ? (
        <Text variant="caption">{`No shortcut matches “${query}”.`}</Text>
      ) : (
        <ShortcutList
          clashes={clashMap}
          entries={shown}
          onCaptureChange={onCaptureChange}
          onPreview={setPreviewId}
          onSelect={select}
          selectedId={selectedId}
        />
      )}
    </Pane>
  );
}
