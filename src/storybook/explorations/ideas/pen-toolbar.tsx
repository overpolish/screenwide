// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Focus, MousePointer2 } from "lucide-react";

import { cn } from "../../../lib/styling";

import { CounterMark } from "./look-counter";
import { ArrowMark, MarkDefs } from "./look-marks";
import { lookById, type LookId, type Pen } from "./looks";

// The pens, each showing what it draws in the project's look, so the tool is
// its own preview: pick it and point.

// Every pen on a white tile, the picture's own ground, so the preview reads
// the same on a toolbar's fill and on the island's black.
export function PenIcon({ lookId, pen }: { lookId: LookId; pen: Pen }) {
  const look = lookById(lookId);
  const color = look.id === "product" ? look.ink : look.accent;
  if (pen === "focus")
    return (
      <span className="flex size-7 items-center justify-center rounded-[6px] bg-white">
        <Focus className="size-5" style={{ color }} />
      </span>
    );
  return (
    <svg
      aria-hidden
      className="size-7 rounded-[6px] bg-white"
      viewBox="0 0 28 28"
    >
      <MarkDefs />
      {pen === "counter" ? (
        <CounterMark
          at={{ x: 14, y: 14 }}
          color={color}
          lean={{ x: 0, y: 0 }}
          look={look}
          number={1}
          scale={0.85}
        />
      ) : (
        <ArrowMark
          color={color}
          from={{ x: 6, y: 22 }}
          look={look}
          scale={0.55}
          to={{ x: 22, y: 7 }}
        />
      )}
    </svg>
  );
}

export function PenToolbar({
  lookId,
  onChange,
  value,
}: {
  lookId: LookId;
  onChange: (pen: Pen | "select") => void;
  value: Pen | "select";
}) {
  return (
    <div
      aria-label="Pens"
      className="flex flex-col gap-1 rounded-panel bg-fill p-1"
      role="toolbar"
    >
      {(["select", "counter", "arrow", "focus"] as const).map((item) => (
        <button
          aria-label={item}
          aria-pressed={value === item}
          className={cn(
            "flex size-10 items-center justify-center rounded-control transition-colors",
            value === item
              ? "bg-primary text-on-primary"
              : "hover:bg-fill-strong",
          )}
          key={item}
          onClick={() => {
            onChange(item);
          }}
          type="button"
        >
          {item === "select" ? (
            <MousePointer2 className="size-5" />
          ) : (
            <PenIcon lookId={lookId} pen={item} />
          )}
        </button>
      ))}
    </div>
  );
}
