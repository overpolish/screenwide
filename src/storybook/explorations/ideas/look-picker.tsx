// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";

import { CounterMark } from "./look-counter";
import { ArrowMark, MarkDefs } from "./look-marks";
import { type LookId, LOOKS } from "./looks";

// The project's look, chosen once: each choice shows itself, a counter and
// an arrow drawn the way that look draws them, so the choice is made by
// seeing rather than by reading a name.

export function LookPreview({ id }: { id: LookId }) {
  const look = LOOKS.find((item) => item.id === id) ?? LOOKS[0];
  return (
    <svg
      aria-hidden
      className="h-14 w-24 rounded-part bg-white"
      viewBox="0 0 96 56"
    >
      <MarkDefs />
      <rect fill="#f2f2f4" height={14} rx={4} width={34} x={52} y={30} />
      <ArrowMark
        color={look.id === "product" ? look.ink : look.accent}
        from={{ x: 14, y: 46 }}
        look={look}
        scale={0.6}
        to={{ x: 50, y: 36 }}
      />
      <CounterMark
        at={{ x: 36, y: 22 }}
        color={look.id === "product" ? look.ink : look.accent}
        lean={{ x: -0.5, y: -0.4 }}
        look={look}
        number={1}
        scale={0.75}
      />
    </svg>
  );
}

export function LookPicker({
  onChange,
  value,
}: {
  onChange: (id: LookId) => void;
  value: LookId;
}) {
  return (
    <div aria-label="Look" className="flex gap-separate" role="radiogroup">
      {LOOKS.map(({ id, label, like }) => {
        const isChosen = id === value;
        return (
          <button
            aria-checked={isChosen}
            className={cn(
              "flex items-center gap-separate rounded-panel p-inset-sm pe-inset text-start transition-colors",
              isChosen
                ? "bg-primary text-on-primary"
                : "bg-fill hover:bg-fill-strong",
            )}
            key={id}
            onClick={() => {
              onChange(id);
            }}
            role="radio"
            type="button"
          >
            <LookPreview id={id} />
            <span className="flex flex-col">
              <span className="text-label font-semibold">{label}</span>
              <span
                className={cn(
                  "max-w-36 text-caption",
                  isChosen ? "text-on-primary/70" : "text-fg-tertiary",
                )}
              >
                {like}
              </span>
            </span>
          </button>
        );
      })}
    </div>
  );
}
