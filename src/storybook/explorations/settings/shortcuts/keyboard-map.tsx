// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useLayoutEffect, useRef, useState } from "react";

import { KeyCap, type KeyState } from "./key-cap";
import { KEY_ROWS, ROW_UNITS } from "./keyboard-layout";
import {
  holdKey,
  type Modifier,
  parseCombo,
  type Reserved,
  sameMods,
  type ShortcutEntry,
} from "./shortcut-model";

// The keyboard, with the shortcuts of one scope on it. The keys light for
// the modifiers held, on screen or on the real keyboard, as in Final Cut
// Pro's Command Editor: hold ⇧⌘ and the keys that do something with ⇧⌘ come
// up. It is a lens on the list beneath it, not a second place to set things
// up: everything it does can be done from the list.

const GAP = 4;

export function KeyboardMap({
  clashing,
  entries,
  held,
  matches,
  onHoverKey,
  onPressKey,
  onToggleModifier,
  pressedKey,
  previewId,
  reserved,
  selectedId,
}: {
  /** Ids of entries sharing a combination with another. */
  clashing: ReadonlySet<string>;
  entries: ShortcutEntry[];
  held: ReadonlySet<Modifier>;
  /** Ids of entries the search finds; null with no search. */
  matches: ReadonlySet<string> | null;
  onHoverKey: (key: string | null) => void;
  onPressKey: (key: string) => void;
  onToggleModifier: (modifier: Modifier) => void;
  pressedKey: string | null;
  previewId: string | null;
  reserved: Reserved[];
  selectedId: string | null;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [unit, setUnit] = useState(32);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      const width = entry.contentRect.width;
      setUnit(Math.min((width - (ROW_UNITS - 1) * GAP) / ROW_UNITS, 44));
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  const height = Math.round(unit * 0.92);
  const widthOf = (keys = 1) => keys * unit + (keys - 1) * GAP;

  const stateOf = (id: string, modifier?: Modifier): KeyState => {
    const combos = entries
      .filter((entry) => entry.kind === "combo")
      .map((entry) => ({ combo: parseCombo(entry.value), entry }))
      .filter(({ combo }) => combo?.key === id);
    const bound = combos
      .filter(({ combo }) => combo && sameMods(combo.mods, held))
      .map(({ entry }) => entry);
    return {
      bound,
      elsewhere: combos.length > bound.length,
      hasClash: bound.some((entry) => clashing.has(entry.id)),
      holds: entries.filter(
        (entry) => entry.kind === "hold" && holdKey(entry.value) === id,
      ),
      isDimmed:
        matches !== null &&
        bound.length > 0 &&
        !bound.some((entry) => matches.has(entry.id)),
      isHeld: modifier !== undefined && held.has(modifier),
      isPressed: pressedKey === id,
      isPreviewed: bound.some((entry) => entry.id === previewId),
      isSelected: bound.some((entry) => entry.id === selectedId),
      reserved: reserved.find(
        (candidate) => candidate.key === id && sameMods(candidate.mods, held),
      ),
    };
  };

  return (
    <div
      aria-label="Keyboard"
      className="flex flex-col rounded-panel bg-fill p-inset-sm"
      dir="ltr"
      role="group"
      style={{ gap: GAP }}
    >
      <div className="flex flex-col" ref={ref} style={{ gap: GAP }}>
        {KEY_ROWS.map((row) => (
          <div className="flex" key={row[0].id} style={{ gap: GAP }}>
            {row.map((def) =>
              def.id === "updown" ? (
                <span
                  aria-hidden
                  className="flex shrink-0 flex-col"
                  key={def.id}
                  style={{ gap: GAP / 2, height, width: widthOf() }}
                >
                  {["↑", "↓"].map((arrow) => (
                    <span
                      className="flex grow items-center justify-center rounded-sm bg-fill text-[9px] text-fg-tertiary"
                      key={arrow}
                    >
                      {arrow}
                    </span>
                  ))}
                </span>
              ) : (
                <KeyCap
                  def={def}
                  height={height}
                  key={def.id}
                  onHover={onHoverKey}
                  onPress={() => {
                    if (def.modifier) onToggleModifier(def.modifier);
                    else onPressKey(def.id);
                  }}
                  state={stateOf(def.id, def.modifier)}
                  width={widthOf(def.width)}
                />
              ),
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
