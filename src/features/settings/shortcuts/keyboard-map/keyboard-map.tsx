// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CSSProperties } from "react";

import { t } from "../../../../i18n/i18n";
import {
  type HotkeyModifier,
  hotkeyValue,
  parseHotkey,
} from "../../../../lib/hotkey";
import { reservedUse } from "../reserved-shortcuts";

import { KeyCap, type KeyState } from "./key-cap";
import { type KeyDef, keyboardRows } from "./keyboard-layout";
import { useSettledHover } from "./use-settled-hover";

import type { ShortcutScope } from "../shortcut-conflicts";
import type { ShortcutEntry } from "../shortcut-entries";

// The keyboard, with the shortcuts of one scope on it, as in Final Cut
// Pro's Command Editor: latch ⌘⇧ and the keys that do something with ⌘⇧
// come up. It is a lens on the list beneath it: everything it does can be
// done there too. With a shortcut chosen in the list, a click on a key
// gives it that key (`map-assignment.ts`). Each key's button reaches
// into the gaps round its face, so the buttons tile the map whole.

// A key's width in quarter keys, the grid's track; every row is 14.5 keys.
const TRACKS = 58;

// How far each key's button reaches past its face: halfway across a gap
// between keys, so neighbours meet with no gap between them, and to the
// map's edge at its sides. The gap above the bottom row is the bottom
// row's alone: its up and down arrows are half a key tall, and with the
// whole gap the up arrow's button is as tall as the down arrow's, which
// reaches the map's edge, both about the least a pointer target should be.
const HALF_GAP = "calc(var(--spacing-inline) / 2)";
const GAP = "var(--spacing-inline)";
const EDGE = "var(--spacing-inset-sm)";
const NONE = "0px";
/** Half the gap between the stacked arrows' faces (`gap-0.5`). */
const HALF_STACK_GAP = "calc(var(--spacing) * 0.25)";

/** How far the keys of row `index` of `count` reach up and down. */
function rowReach(index: number, count: number) {
  const last = count - 1;
  return {
    bottom: index === last ? EDGE : index === last - 1 ? NONE : HALF_GAP,
    top: index === 0 ? EDGE : index === last ? GAP : HALF_GAP,
  };
}

const sameModifiers = (
  modifiers: readonly HotkeyModifier[],
  latched: ReadonlySet<HotkeyModifier>,
) =>
  modifiers.length === latched.size &&
  modifiers.every((modifier) => latched.has(modifier));

export function KeyboardMap({
  assigned,
  clashing,
  entries,
  isMac,
  latched,
  onHoverKey,
  onPressKey,
  onToggleModifier,
  pointedId,
  scope,
  selectedId,
}: {
  /** The key a click just gave a shortcut, and how many times one has. */
  assigned: { code: string; count: number } | null;
  /** Ids of shortcuts sharing their combination with another. */
  clashing: ReadonlySet<string>;
  /** The shortcuts of the scope shown. */
  entries: readonly ShortcutEntry[];
  isMac: boolean;
  latched: ReadonlySet<HotkeyModifier>;
  onHoverKey: (code: string | null) => void;
  onPressKey: (key: KeyDef) => void;
  onToggleModifier: (modifier: HotkeyModifier) => void;
  pointedId: string | null;
  scope: ShortcutScope;
  selectedId: string | null;
}) {
  const hover = useSettledHover(onHoverKey);
  const latchedList = [...latched];
  // A key held (Glide's) takes a modifier key itself rather than latching it.
  const holdChosen = entries.some(
    (entry) => entry.id === selectedId && entry.kind === "hold",
  );
  const stateOf = (def: KeyDef): KeyState => {
    const bound = def.modifier
      ? []
      : entries.filter((entry) => {
          const parsed =
            entry.kind === "combo" ? parseHotkey(entry.value, isMac) : null;
          return (
            parsed?.code === def.code &&
            sameModifiers(parsed.modifiers, latched)
          );
        });
    const held = entries.filter(
      (entry) => entry.kind === "hold" && entry.value === def.code,
    );
    const uses = [...bound, ...held];
    return {
      does: uses.map((entry) => entry.label),
      icon: uses[0]?.icon,
      isClash: bound.some((entry) => clashing.has(entry.id)),
      isOff: uses.length > 0 && uses.every((entry) => entry.isOff),
      isPointed: uses.some((entry) => entry.id === pointedId),
      isReserved:
        scope === "anywhere" &&
        !def.modifier &&
        latched.size > 0 &&
        reservedUse(hotkeyValue(def.code, latchedList), isMac) !== null,
      isSelected: uses.some((entry) => entry.id === selectedId),
    };
  };
  /** A key, reaching as far as `reach` says round its face. */
  const cap = (def: KeyDef, reach: CSSProperties, isStacked = false) => (
    <KeyCap
      assigned={assigned?.code === def.code ? assigned.count : null}
      // A half-height arrow has no room for the inset, and takes its
      // height from its half of the row.
      className={
        isStacked ? "items-center justify-center p-0" : "h-control-height"
      }
      def={def}
      isLatched={def.modifier !== undefined && latched.has(def.modifier)}
      key={def.code}
      onHoverChange={(isHovered) => {
        hover(def.code, isHovered);
      }}
      onPress={() => {
        if (def.modifier && !holdChosen) onToggleModifier(def.modifier);
        else onPressKey(def);
      }}
      state={stateOf(def)}
      style={
        isStacked
          ? { ...reach, flex: 1, minHeight: 0 }
          : { ...reach, gridColumn: `span ${String((def.width ?? 1) * 4)}` }
      }
    />
  );
  const rows = keyboardRows(isMac);

  return (
    <div
      aria-label={t("settings-shortcuts-map")}
      className="flex flex-col rounded-panel bg-fill"
      // A keyboard is laid out left to right in every language.
      dir="ltr"
      role="group"
    >
      {rows.map((row, rowIndex) => {
        const { bottom, top } = rowReach(rowIndex, rows.length);
        return (
          <div
            className="grid"
            key={("column" in row[0] ? row[0].column[0] : row[0]).code}
            style={{
              gridTemplateColumns: `repeat(${String(TRACKS)}, minmax(0, 1fr))`,
            }}
          >
            {row.map((item, index) => {
              const sides = {
                paddingLeft: index === 0 ? EDGE : HALF_GAP,
                paddingRight: index === row.length - 1 ? EDGE : HALF_GAP,
              };
              if (!("column" in item))
                return cap(item, {
                  ...sides,
                  paddingBottom: bottom,
                  paddingTop: top,
                });
              const [up, down] = item.column;
              return (
                <div
                  className="flex flex-col"
                  key={up.code}
                  style={{ gridColumn: "span 4" }}
                >
                  {cap(
                    up,
                    {
                      ...sides,
                      paddingBottom: HALF_STACK_GAP,
                      paddingTop: top,
                    },
                    true,
                  )}
                  {cap(
                    down,
                    {
                      ...sides,
                      paddingBottom: bottom,
                      paddingTop: HALF_STACK_GAP,
                    },
                    true,
                  )}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}
