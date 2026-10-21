// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CopyPlus, Trash2 } from "lucide-react";
import { useState } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { ControlRow } from "../../../composites/control-row/control-row";
import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { Slider } from "../../../primitives/slider/slider";
import { Switch } from "../../../primitives/switch/switch";
import { ChangedDot } from "../shared/changed-dot";
import { ANNOTATION_COLORS, ARROW_COLOR, PRESETS } from "../shared/mock-data";
import { PanelWindow } from "../shared/panel-window";
import { ArrowGlyph } from "../shared/picture";

import { ANNOTATIONS, at, clock, span } from "./editor-data";

const ARROW = ANNOTATIONS.find(({ isSelected }) => isSelected);

// The dark the presets are previewed on: content, so a white arrow shows.
const PREVIEW_BACKDROP = "#1c1c1e";

// When the arrow shows, on a strip that is the whole recording: the span in
// the primary colour, a knob at each end to drag.
function Span({ end, start }: { end: number; start: number }) {
  return (
    <span className="relative mx-1 h-4" dir="ltr">
      <span className="absolute inset-x-0 top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-fill" />
      <span
        className="absolute top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-primary"
        style={span(start, end)}
      />
      {[start, end].map((second) => (
        <span
          className="absolute top-1/2 h-mark w-2 -translate-1/2 rounded-full control-knob"
          key={second}
          style={{ insetInlineStart: at(second) }}
        />
      ))}
    </span>
  );
}

// Styles kept for arrows, one of a set chosen in place, so the chosen one
// takes the quieter selected grey with its name unchanged.
function Presets() {
  const [chosen, setChosen] = useState(PRESETS[0]?.name ?? "");
  return (
    <RadioGroup
      aria-label="Style"
      className="grid grid-cols-4 gap-0.5"
      onChange={setChosen}
      orientation="horizontal"
      value={chosen}
    >
      {PRESETS.map(({ color, isHandDrawn, name, width }) => (
        <RadioField key={name} value={name}>
          <RadioButton
            className={cn(
              "flex w-full cursor-default flex-col items-center gap-1 rounded-control p-1 text-caption text-fg-secondary transition-colors",
              "data-[hovered]:bg-fill data-[selected]:bg-selected data-[selected]:text-fg",
              focusStyles,
              elementFocusVisible,
            )}
          >
            <span
              className="flex size-10 items-center justify-center rounded-part"
              style={{ background: PREVIEW_BACKDROP }}
            >
              <ArrowGlyph
                className="size-7"
                color={color}
                isHandDrawn={isHandDrawn}
                width={width}
              />
            </span>
            {name}
          </RadioButton>
        </RadioField>
      ))}
    </RadioGroup>
  );
}

// The colour is checked, so a ring in the primary colour marks it.
function Colors() {
  const [chosen, setChosen] = useState(ARROW_COLOR);
  return (
    <RadioGroup
      aria-label="Colour"
      className="flex justify-between"
      onChange={setChosen}
      orientation="horizontal"
      value={chosen}
    >
      {ANNOTATION_COLORS.map((color) => (
        <RadioField key={color} value={color}>
          <RadioButton
            aria-label={color}
            className={cn(
              "block size-6 cursor-default rounded-part inset-ring inset-ring-divider outline-offset-2",
              "data-[selected]:outline-2 data-[selected]:outline-primary data-[selected]:outline-solid",
              focusStyles,
              elementFocusVisible,
            )}
            style={{ background: color }}
          />
        </RadioField>
      ))}
    </RadioGroup>
  );
}

// The tool panel for the selected arrow: a window of its own, hung from the
// workspace's corner whenever something is selected, so it is always in the
// same place. Its head says what it edits and when; then the house row
// order: styles, size, options, animation, colour, actions.
export function ArrowPanel() {
  const [head, setHead] = useState("end");
  const [isHandDrawn, setIsHandDrawn] = useState(false);
  const [isAnimated, setIsAnimated] = useState(true);
  return (
    <PanelWindow className="w-72 gap-separate p-inset">
      <div className="flex items-center gap-inline">
        <span className="text-heading">Arrow 1</span>
        <span className="text-caption text-fg-tertiary tabular-nums">
          {ARROW ? `${clock(ARROW.start)} to ${clock(ARROW.end)}` : null}
        </span>
        <span className="ms-auto flex">
          <IconButton aria-label="Duplicate" variant="ghost">
            <CopyPlus />
          </IconButton>
          <IconButton aria-label="Delete" variant="ghost">
            <Trash2 />
          </IconButton>
        </span>
      </div>
      {ARROW ? <Span end={ARROW.end} start={ARROW.start} /> : null}
      <Presets />
      <div className="flex flex-col gap-inline">
        <ControlRow
          title="Width"
          titleAccessory={<ChangedDot setting="Width" />}
        >
          {(controlProps) => (
            <Slider {...controlProps} className="w-28" defaultValue={45} />
          )}
        </ControlRow>
        <ControlRow title="Head">
          {(controlProps) => (
            <span {...controlProps} role="group">
              <PillGroup
                aria-label="Head"
                items={[
                  { id: "none", label: "None" },
                  { id: "end", label: "End" },
                  { id: "both", label: "Both" },
                ]}
                onSelectionChange={setHead}
                selected={head}
              />
            </span>
          )}
        </ControlRow>
        <ControlRow title="Hand-drawn">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={isHandDrawn}
              onChange={setIsHandDrawn}
            />
          )}
        </ControlRow>
        <ControlRow title="Animate">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={isAnimated}
              onChange={setIsAnimated}
            />
          )}
        </ControlRow>
      </div>
      <Colors />
      <div className="flex justify-end">
        <Button>Reverse</Button>
      </div>
    </PanelWindow>
  );
}
