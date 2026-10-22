// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";
import { useEffect, useState } from "react";

import { cn } from "../../../lib/styling";

import { IdeaFrame } from "./idea-frame";

// Shortcuts on the keyboard they live on. Hold modifiers, on screen or on
// the real keyboard, and the keys they unlock light up; hover a key to read
// what it does with each. A free key is as plain as a key with nothing on it.

type Modifier = "cmd" | "ctrl" | "opt" | "shift";
const GLYPH: Record<Modifier, string> = {
  cmd: "⌘",
  ctrl: "⌃",
  opt: "⌥",
  shift: "⇧",
};

const BINDINGS: { key: string; label: string; modifiers: Modifier[] }[] = [
  {
    key: "6",
    label: "Show or hide recording controls",
    modifiers: ["shift", "cmd"],
  },
  { key: "9", label: "Save replay", modifiers: ["shift", "cmd"] },
  { key: "s", label: "Take a screenshot", modifiers: ["shift", "cmd"] },
  { key: "1", label: "Funny moment", modifiers: ["shift", "cmd"] },
  { key: "2", label: "Notable moment", modifiers: ["shift", "cmd"] },
  { key: "r", label: "Show ruler", modifiers: ["shift", "cmd"] },
  { key: "a", label: "Draw on the screen", modifiers: ["shift", "cmd"] },
  { key: "t", label: "Read text or a QR code", modifiers: ["shift", "cmd"] },
  { key: "x", label: "Clear annotations", modifiers: ["opt", "cmd"] },
  { key: "z", label: "Undo a measurement", modifiers: [] },
  { key: "c", label: "Copy measurement", modifiers: [] },
  { key: "v", label: "Vertical guide", modifiers: [] },
  { key: "h", label: "Horizontal guide", modifiers: [] },
  { key: "m", label: "Measure radius", modifiers: [] },
];

const ROWS: (string | [string, number])[][] = [
  [
    "`",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "0",
    "-",
    "=",
    ["delete", 1.5],
  ],
  [
    ["tab", 1.5],
    "q",
    "w",
    "e",
    "r",
    "t",
    "y",
    "u",
    "i",
    "o",
    "p",
    "[",
    "]",
    "\\",
  ],
  [
    ["caps", 1.75],
    "a",
    "s",
    "d",
    "f",
    "g",
    "h",
    "j",
    "k",
    "l",
    ";",
    "'",
    ["return", 1.75],
  ],
  [
    ["shift", 2.25],
    "z",
    "x",
    "c",
    "v",
    "b",
    "n",
    "m",
    ",",
    ".",
    "/",
    ["shift", 2.25],
  ],
  [
    ["ctrl", 1.25],
    ["opt", 1.25],
    ["cmd", 1.5],
    ["space", 6.25],
    ["cmd", 1.5],
    ["opt", 1.25],
  ],
];

const UNIT = 40;
const same = (first: Modifier[], second: Set<Modifier>) =>
  first.length === second.size &&
  first.every((modifier) => second.has(modifier));

export function KeyboardMap() {
  const [held, setHeld] = useState<Set<Modifier>>(
    () => new Set(["shift", "cmd"]),
  );
  const [hovered, setHovered] = useState<string | null>(null);
  const [pressed, setPressed] = useState<string | null>(null);

  useEffect(() => {
    const read = (event: KeyboardEvent) => {
      const next = new Set<Modifier>();
      if (event.metaKey) next.add("cmd");
      if (event.shiftKey) next.add("shift");
      if (event.altKey) next.add("opt");
      if (event.ctrlKey) next.add("ctrl");
      if (event.type === "keydown" || next.size > 0) setHeld(next);
      if (event.type === "keydown" && event.key.length === 1)
        setPressed(event.code.replace(/^Key|^Digit/, "").toLowerCase());
    };
    window.addEventListener("keydown", read);
    window.addEventListener("keyup", read);
    return () => {
      window.removeEventListener("keydown", read);
      window.removeEventListener("keyup", read);
    };
  }, []);

  const bindingsOn = (key: string) =>
    BINDINGS.filter((binding) => binding.key === key);
  const shown = hovered ?? pressed;

  return (
    <IdeaFrame
      hint="Click the modifiers, or hold them on your keyboard. Hover a key to read what it does."
      pitch="Shortcuts on the keyboard they live on. Hold modifiers and the keys they unlock light up; a key with a dot does something with other modifiers. Free keys are plain to see, so choosing a new shortcut starts from what is free."
      title="Keyboard map"
    >
      <div className="flex flex-col gap-group" dir="ltr">
        <div className="flex flex-col gap-1.5 rounded-panel bg-fill p-inset">
          {ROWS.map((row) => (
            <div className="flex gap-1.5" key={JSON.stringify(row[0])}>
              {row.map((entry, index) => {
                const [key, width] =
                  typeof entry === "string" ? [entry, 1] : entry;
                const modifier = (
                  ["cmd", "ctrl", "opt", "shift"] as const
                ).find((name) => name === key);
                const bound = bindingsOn(key);
                const isLit = bound.some((binding) =>
                  same(binding.modifiers, held),
                );
                const isUsed = bound.length > 0;
                return (
                  <motion.button
                    animate={{ scale: pressed === key ? [1, 0.92, 1] : 1 }}
                    className={cn(
                      "relative flex h-10 shrink-0 items-center justify-center rounded-control text-label transition-colors",
                      modifier && held.has(modifier)
                        ? "bg-primary text-on-primary"
                        : isLit
                          ? "bg-fill-strong text-fg ring-2 ring-primary ring-inset"
                          : "bg-fill-strong text-fg-secondary hover:text-fg",
                    )}
                    key={`${key}-${String(index)}`}
                    onClick={() => {
                      if (!modifier) return;
                      setHeld((current) => {
                        const next = new Set(current);
                        if (next.has(modifier)) next.delete(modifier);
                        else next.add(modifier);
                        return next;
                      });
                    }}
                    onPointerEnter={() => {
                      setHovered(isUsed ? key : null);
                    }}
                    onPointerLeave={() => {
                      setHovered(null);
                    }}
                    style={{ width: width * UNIT + (width - 1) * 6 }}
                    type="button"
                  >
                    {modifier
                      ? GLYPH[modifier]
                      : key.length === 1
                        ? key.toUpperCase()
                        : key}
                    {isUsed && !isLit ? (
                      <span className="absolute top-1 end-1 size-1 rounded-full bg-fg-tertiary" />
                    ) : null}
                  </motion.button>
                );
              })}
            </div>
          ))}
        </div>
        <div className="flex min-h-16 flex-col gap-1">
          {shown && bindingsOn(shown).length > 0 ? (
            bindingsOn(shown).map((binding) => (
              <span
                className="flex items-center gap-separate text-body"
                key={binding.label}
              >
                <span className="w-20 text-end font-medium tabular-nums">
                  {binding.modifiers
                    .map((modifier) => GLYPH[modifier])
                    .join("")}
                  {shown.toUpperCase()}
                </span>
                {binding.label}
              </span>
            ))
          ) : (
            <span className="text-caption text-fg-tertiary">
              {held.size === 0
                ? "Keys that work without a modifier, while a tool is open"
                : `Lit keys work with ${[...held].map((modifier) => GLYPH[modifier]).join("")}`}
            </span>
          )}
        </div>
      </div>
    </IdeaFrame>
  );
}
