// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleAlert, CircleCheck, Info } from "lucide-react";
import { type ReactNode } from "react";

import { Keyboard } from "../../../../primitives/keyboard/keyboard";
import { Text } from "../../../../primitives/text/text";

import {
  comboText,
  GLYPH,
  heldText,
  holdKey,
  keyText,
  type Modifier,
  parseCombo,
  RESERVED,
  sameMods,
  type Scope,
  type ShortcutEntry,
} from "./shortcut-model";

// The line under the map, saying in words what the map shows: everything a
// key under the pointer does, under any modifiers; how to give the chosen
// shortcut new keys; or why a click on a key did not take. It stays one
// line tall whatever it says, so the list under it never moves.

export type Notice = { text: string; tone: "done" | "refused" };

function Line({ children, icon }: { children: ReactNode; icon: ReactNode }) {
  return (
    <div
      aria-live="polite"
      className="flex h-control-height min-w-0 items-center gap-inline px-inset-sm"
    >
      <span className="flex shrink-0 text-fg-tertiary [&_svg]:size-icon-sm">
        {icon}
      </span>
      <div className="flex min-w-0 items-center gap-separate overflow-hidden whitespace-nowrap">
        {children}
      </div>
    </div>
  );
}

function Uses({ keys, label }: { keys: string; label: string }) {
  return (
    <span className="flex shrink-0 items-center gap-inline">
      <Keyboard>{keys}</Keyboard>
      <Text as="span" variant="caption">
        {label}
      </Text>
    </span>
  );
}

export function KeyDetail({
  entries,
  held,
  hoveredKey,
  notice,
  scope,
  selected,
}: {
  entries: ShortcutEntry[];
  held: ReadonlySet<Modifier>;
  hoveredKey: string | null;
  notice: Notice | null;
  scope: Scope;
  selected: ShortcutEntry | null;
}) {
  // A modifier key is named by its side ("opt-left"); only a held key can
  // be on one, since a modifier pressed alone is no shortcut.
  const isModifier = hoveredKey?.includes("-") ?? false;
  const holds = entries.filter(
    (entry) => entry.kind === "hold" && holdKey(entry.value) === hoveredKey,
  );
  if (hoveredKey && (!isModifier || holds.length > 0)) {
    const name = isModifier
      ? GLYPH[hoveredKey.split("-")[0] as Modifier]
      : keyText(hoveredKey);
    const uses = entries.flatMap((entry) => {
      const combo = entry.kind === "combo" ? parseCombo(entry.value) : null;
      return combo?.key === hoveredKey
        ? [{ keys: comboText(combo), label: entry.label }]
        : [];
    });
    const reserved =
      scope === "anywhere"
        ? RESERVED.filter(
            (candidate) =>
              candidate.key === hoveredKey && sameMods(candidate.mods, held),
          )
        : [];
    if (uses.length + holds.length + reserved.length === 0)
      return (
        <Line icon={<Info />}>
          <Text as="span" variant="caption">
            {`${heldText(held)}${name} is free.`}
          </Text>
        </Line>
      );
    return (
      <Line icon={<Info />}>
        {uses.map((use) => (
          <Uses key={use.label} {...use} />
        ))}
        {holds.map((entry) => (
          <Uses key={entry.id} keys={name} label={`Hold: ${entry.label}`} />
        ))}
        {reserved.map((candidate) => (
          <Uses
            key={candidate.label}
            keys={comboText(candidate)}
            label={`${candidate.label}, in ${candidate.by === "macOS" ? "macOS" : "most apps"}`}
          />
        ))}
      </Line>
    );
  }
  if (notice)
    return (
      <Line
        icon={
          notice.tone === "done" ? (
            <CircleCheck className="text-success" />
          ) : (
            <CircleAlert className="text-warning" />
          )
        }
      >
        <Text as="span" variant="caption">
          {notice.text}
        </Text>
      </Line>
    );
  if (selected)
    return (
      <Line icon={<Info />}>
        <Text as="span" variant="caption">
          {selected.kind === "hold"
            ? `Press the key to hold for ${selected.label} in its field.`
            : `Click a key to give ${selected.label} new keys, holding the modifiers you want.`}
        </Text>
      </Line>
    );
  return (
    <Line icon={<Info />}>
      <Text as="span" variant="caption">
        {held.size > 0
          ? `Raised keys do something with ${heldText(held)}. Point at one to see what.`
          : scope === "anywhere"
            ? "Hold ⌘, ⌥ or ⌃, here or on your keyboard, to see what they unlock."
            : "These keys work while the tool is open. Point at one to see what."}
      </Text>
    </Line>
  );
}
