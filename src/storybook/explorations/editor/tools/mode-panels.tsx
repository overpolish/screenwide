// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { cn } from "../../../../lib/styling";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Switch } from "../../../../primitives/switch/switch";
import { Text } from "../../../../primitives/text/text";
import { BACKDROPS, STEPS, stepAt } from "../editor-data";
import { type Editor } from "../model/use-editor";

import { LayoutPicker } from "./choices";

// The panel for a mode picked from the toolbar. Each acts on
// the picture as it is changed, so there is nothing to apply.

export function ScenePanel({ editor }: { editor: Editor }) {
  const index = stepAt(editor.playhead);
  return (
    <div className="flex w-80 flex-col gap-2 p-3">
      <Text variant="caption">{STEPS[index].title} is shown as</Text>
      <LayoutPicker
        onChange={(layout) => {
          editor.setScene(index, layout);
        }}
        value={editor.scenes[index]}
      />
    </div>
  );
}

export function FramePanel({ editor }: { editor: Editor }) {
  return (
    <div className="flex w-72 flex-col gap-2 p-3">
      <Text variant="caption">Background</Text>
      <div className="grid grid-cols-3 gap-1.5">
        {BACKDROPS.map(({ background, id, label }) => (
          <button
            aria-pressed={editor.backdrop === id}
            className={cn(
              "flex h-14 items-end rounded-[10px] p-1.5 text-caption ring-2",
              id === "none" && "border border-dashed border-fg-tertiary",
              editor.backdrop === id ? "ring-fg" : "ring-transparent",
            )}
            key={id}
            onClick={() => {
              editor.setBackdrop(id);
            }}
            style={{ background }}
            type="button"
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
}

export function PointerPanel({ editor }: { editor: Editor }) {
  return (
    <div className="flex w-72 flex-col gap-inline p-3">
      <ControlRow title="Show clicks">
        {(controlProps) => (
          <Switch
            {...controlProps}
            isSelected={editor.effects.ripple}
            onChange={(ripple) => {
              editor.setEffects({ ...editor.effects, ripple });
            }}
          />
        )}
      </ControlRow>
      <ControlRow title="Smooth movement">
        {(controlProps) => <Switch {...controlProps} defaultSelected />}
      </ControlRow>
    </div>
  );
}

export function CaptionsPanel({ editor }: { editor: Editor }) {
  const [style, setStyle] = useState("bar");
  if (!editor.isVoiced) {
    return (
      <div className="flex w-64 flex-col gap-1 p-3">
        <Text variant="heading">No voice</Text>
        <Text variant="caption">
          Nothing was said in this recording, so there is nothing to caption.
        </Text>
      </div>
    );
  }
  return (
    <div className="flex w-72 flex-col gap-inline p-3">
      <ControlRow title="Show captions">
        {(controlProps) => (
          <Switch
            {...controlProps}
            isSelected={editor.effects.captions}
            onChange={(captions) => {
              editor.setEffects({ ...editor.effects, captions });
            }}
          />
        )}
      </ControlRow>
      <ControlRow title="Style">
        {(controlProps) => (
          <PillGroup
            {...controlProps}
            aria-label="Style"
            items={[
              { id: "bar", label: "Bar" },
              { id: "words", label: "Word by word" },
            ]}
            onSelectionChange={setStyle}
            selected={style}
          />
        )}
      </ControlRow>
    </div>
  );
}
