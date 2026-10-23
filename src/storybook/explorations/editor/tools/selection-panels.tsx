// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Scissors, Trash2 } from "lucide-react";
import { type ReactNode } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { Button } from "../../../../primitives/button/button";
import { IconButton } from "../../../../primitives/button/icon-button";
import { Switch } from "../../../../primitives/switch/switch";
import { Text } from "../../../../primitives/text/text";
import { describe } from "../../ideas/action-script-lines";
import { clock } from "../../ideas/ideas-data";
import { LOOKS } from "../../ideas/looks";
import { appearsAt } from "../../ideas/point-geometry";
import { STEPS } from "../editor-data";
import { type Editor, type TimedMark } from "../model/use-editor";

import { LayoutPicker, MarkChoices, MoreRows } from "./choices";

// The panel for a selection: what it is and when, then
// the few things to change. Times are said as events where they come from
// one, since that is what times them.

const PEN_NAMES = { arrow: "Arrow", counter: "Counter", focus: "Focus" };

const markName = (mark: TimedMark) =>
  `${PEN_NAMES[mark.pen]} ${mark.pen === "arrow" ? "to" : "on"} ${mark.element.label}`;

function Head({
  action,
  note,
  title,
}: {
  action: ReactNode;
  note: string;
  title: string;
}) {
  return (
    <div className="flex items-start gap-inline">
      <div className="flex min-w-0 grow flex-col">
        <Text className="truncate" variant="heading">
          {title}
        </Text>
        <Text variant="caption">{note}</Text>
      </div>
      {action}
    </div>
  );
}

export function MarkPanel({
  editor,
  mark,
}: {
  editor: Editor;
  mark: TimedMark;
}) {
  const look = LOOKS.find(
    ({ id }) => id === (mark.lookOverride ?? editor.lookId),
  );
  const isTimedByAction = appearsAt(mark.element)?.at === mark.start;
  return (
    <div className="flex w-72 flex-col gap-3 p-3">
      <Head
        action={
          <IconButton aria-label="Delete" onPress={editor.remove}>
            <Trash2 />
          </IconButton>
        }
        note={
          isTimedByAction
            ? `From ${clock(mark.start)}, when ${mark.element.label} was used, to the end of the step`
            : `From ${clock(mark.start)} to the end of the step`
        }
        title={markName(mark)}
      />
      <MarkChoices editor={editor} look={look ?? LOOKS[0]} mark={mark} />
      {editor.isMoreOpen ? <MoreRows pen={mark.pen} /> : null}
      <div className="flex justify-end">
        <Button
          onPress={() => {
            editor.setIsMoreOpen(!editor.isMoreOpen);
          }}
        >
          {editor.isMoreOpen ? "Less" : "More"}
        </Button>
      </div>
    </div>
  );
}

export function BeatPanel({ editor, id }: { editor: Editor; id: string }) {
  const beat = editor.beats.find((item) => item.id === id);
  if (!beat) return null;
  const seconds = (beat.end - beat.start).toFixed(1);
  return (
    <div className="flex w-72 flex-col gap-3 p-3">
      <Head
        action={
          <IconButton aria-label="Cut it and its time" onPress={editor.remove}>
            <Scissors />
          </IconButton>
        }
        note={`${seconds} s at ${clock(beat.start)}`}
        title={beat.kind === "action" ? describe(beat.action) : beat.text}
      />
      {beat.kind === "action" ? (
        // The picture already shows the beat: the playhead moved to it.
        <ControlRow title="Zoom in on it">
          {(controlProps) => <Switch {...controlProps} defaultSelected />}
        </ControlRow>
      ) : null}
    </div>
  );
}

export function StepPanel({
  editor,
  index,
}: {
  editor: Editor;
  index: number;
}) {
  const step = STEPS[index];
  return (
    <div className="flex w-80 flex-col gap-3 p-3">
      <Head
        action={
          <IconButton aria-label="Cut the step" onPress={editor.remove}>
            <Scissors />
          </IconButton>
        }
        note={`${(step.end - step.start).toFixed(1)} s, from ${clock(step.start)}`}
        title={step.title}
      />
      <LayoutPicker
        onChange={(layout) => {
          editor.setScene(index, layout);
        }}
        value={editor.scenes[index]}
      />
    </div>
  );
}
