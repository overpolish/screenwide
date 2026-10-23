// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Switch } from "../../../../primitives/switch/switch";
import { swatchOf } from "../../ideas/look-pens-model";
import { LookPreview } from "../../ideas/look-picker";
import {
  type Look,
  LOOKS,
  type Pen,
  type Size,
  type Swatch,
} from "../../ideas/looks";
import { type Layout, LAYOUTS } from "../editor-data";
import { type Editor, type TimedMark } from "../model/use-editor";

// The controls the corner's panels are made of. A mark's colour comes from
// its look, so there are three, not a palette; its size is three steps, not
// a number; it can borrow another look. Everything else waits under More.

const choice = (isOn: boolean) =>
  cn(
    "flex items-center justify-center rounded-[10px] transition-colors",
    isOn ? "bg-fill-strong" : "hover:bg-fill",
  );

export function MarkChoices({
  editor,
  look,
  mark,
}: {
  editor: Editor;
  look: Look;
  mark: TimedMark;
}) {
  const swatch = swatchOf(mark, editor.lookId);
  const swatches: { background: string; id: Swatch; label: string }[] = [
    {
      background: "linear-gradient(135deg, #000 50%, #fff 50%)",
      id: "ink",
      label: "Ink",
    },
    { background: look.accent, id: "accent", label: "Accent" },
    { background: look.extra, id: "extra", label: "Second colour" },
  ];
  const sizes: Size[] = ["small", "medium", "large"];
  return (
    <div className="flex flex-col gap-2">
      <div className="grid grid-cols-6 gap-1">
        {swatches.map(({ background, id, label }) => (
          <button
            aria-label={label}
            aria-pressed={swatch === id}
            className={cn(choice(swatch === id), "h-9")}
            key={id}
            onClick={() => {
              editor.updateMark({ swatch: id });
            }}
            title={label}
            type="button"
          >
            <span
              className="size-5 rounded-full ring-1 ring-divider"
              style={{ background }}
            />
          </button>
        ))}
        {sizes.map((size, index) => (
          <button
            aria-label={size}
            aria-pressed={mark.size === size}
            className={cn(choice(mark.size === size), "h-9")}
            key={size}
            onClick={() => {
              editor.updateMark({ size });
            }}
            title={size}
            type="button"
          >
            <span
              className="rounded-full bg-fg"
              style={{ height: 5 + index * 3, width: 5 + index * 3 }}
            />
          </button>
        ))}
      </div>
      <div className="grid grid-cols-3 gap-1.5">
        {LOOKS.map(({ id, label }) => {
          const isOn = (mark.lookOverride ?? editor.lookId) === id;
          return (
            <button
              aria-label={label}
              aria-pressed={isOn}
              className={cn(
                "overflow-hidden rounded-[8px] ring-2 [&_svg]:h-11 [&_svg]:w-full [&_svg]:rounded-none",
                isOn ? "ring-fg" : "ring-transparent hover:ring-fill-strong",
              )}
              key={id}
              onClick={() => {
                editor.updateMark({
                  lookOverride: id === editor.lookId ? null : id,
                });
              }}
              title={label}
              type="button"
            >
              <LookPreview id={id} />
            </button>
          );
        })}
      </div>
    </div>
  );
}

/** When a mark comes and goes, said as events of the recording. */
export function MoreRows({ pen }: { pen: Pen }) {
  const [appears, setAppears] = useState("click");
  const [stays, setStays] = useState("step");
  const [comesIn, setComesIn] = useState(pen === "counter" ? "pop" : "draw");
  return (
    <div className="flex flex-col gap-inline border-t border-divider pt-3">
      <ControlRow title="Appears">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Appears"
            items={[
              { id: "click", label: "At its click" },
              { id: "here", label: "Here" },
            ]}
            onSelectionChange={setAppears}
            selected={appears}
          />
        )}
      </ControlRow>
      <ControlRow title="Stays">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Stays"
            items={[
              { id: "step", label: "This step" },
              { id: "short", label: "3 s" },
            ]}
            onSelectionChange={setStays}
            selected={stays}
          />
        )}
      </ControlRow>
      <ControlRow title="Comes in">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Comes in"
            items={[
              { id: "draw", label: "Draws" },
              { id: "pop", label: "Pops" },
              { id: "fade", label: "Fades" },
            ]}
            onSelectionChange={setComesIn}
            selected={comesIn}
          />
        )}
      </ControlRow>
      <ControlRow title="Hand-drawn">
        {(controlProps) => <Switch {...controlProps} />}
      </ControlRow>
      {pen === "arrow" ? (
        <div className="flex justify-end">
          <Button>Reverse</Button>
        </div>
      ) : null}
    </div>
  );
}

/** A scene drawn as what it does to the frame: the screen in grey, the
 * camera as a round face. */
function LayoutDrawing({ id }: { id: Layout }) {
  return (
    <svg aria-hidden className="h-10 w-16" viewBox="0 0 64 40">
      <rect fill="currentColor" height={40} opacity={0.12} rx={4} width={64} />
      {id === "side" ? (
        <>
          <rect
            fill="currentColor"
            height={28}
            opacity={0.5}
            rx={2}
            width={36}
            x={4}
            y={6}
          />
          <circle cx={52} cy={20} fill="currentColor" opacity={0.8} r={8} />
        </>
      ) : (
        <rect
          fill="currentColor"
          height={30}
          opacity={id === "follow" ? 0.25 : 0.5}
          rx={2}
          width={54}
          x={5}
          y={5}
        />
      )}
      {id === "follow" ? (
        <rect
          fill="none"
          height={14}
          rx={2}
          stroke="currentColor"
          strokeWidth={1.5}
          width={22}
          x={30}
          y={16}
        />
      ) : null}
      {id === "pip" ? (
        <circle cx={50} cy={28} fill="currentColor" r={6} />
      ) : null}
    </svg>
  );
}

export function LayoutPicker({
  onChange,
  value,
}: {
  onChange: (layout: Layout) => void;
  value: Layout;
}) {
  return (
    <div className="grid grid-cols-4 gap-1">
      {LAYOUTS.map(({ id, label }) => (
        <button
          aria-pressed={id === value}
          className={cn(
            choice(id === value),
            "flex-col gap-1 p-1.5 text-center text-caption",
            id === value ? "text-fg" : "text-fg-secondary",
          )}
          key={id}
          onClick={() => {
            onChange(id);
          }}
          type="button"
        >
          <LayoutDrawing id={id} />
          {label}
        </button>
      ))}
    </div>
  );
}
