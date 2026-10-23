// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MessageSquareText, Undo2 } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";

import { cn } from "../../../../lib/styling";
import { Text } from "../../../../primitives/text/text";
import { ICONS } from "../../ideas/action-script-lines";
import { type Beat, SPEECH, type Step, STEPS } from "../editor-data";
import { type Editor } from "../model/use-editor";
import { Still } from "../picture/still";
import { BeatBlock, Stills } from "../strip/strip-beats";
import { Playhead } from "../strip/strip-rails";
import { stripScale } from "../strip/strip-scale";

// The recording as the steps it walks through, a card each: what the step is
// about, outlined on its still, how long it now runs, and what was done in
// it. Whole steps are what people cut from a walkthrough, so a card is the
// thing to select, rescene and cut. Double-click a card to open it: its
// beats, for that step only, in the card's place, while the other cards make
// room. Board zoomed out, beats zoomed in; no zoom control.

const STILL_WIDTH = 148;
const COMPACT_STILL_WIDTH = 88;

function StepBeats({ editor, step }: { editor: Editor; step: Step }) {
  const ref = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      setWidth(entry.contentRect.width);
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
    };
  }, []);
  const scale = stripScale({ map: editor.map, range: [step.start, step.end] });
  return (
    <div className="relative h-11 grow overflow-hidden" dir="ltr" ref={ref}>
      <Stills pieces={editor.map.pieces} scale={scale} width={width} />
      {editor.beats.map((beat) => (
        <BeatBlock
          beat={beat}
          isSelected={
            editor.selection?.kind === "beat" && editor.selection.id === beat.id
          }
          key={beat.id}
          onPress={() => {
            editor.select({ id: beat.id, kind: "beat" });
            editor.seek(beat.kind === "action" ? beat.action.at : beat.start);
          }}
          scale={scale}
          width={width}
        />
      ))}
      {editor.playhead >= step.start && editor.playhead < step.end ? (
        <Playhead scale={scale} seconds={editor.playhead} />
      ) : null}
    </div>
  );
}

function ActionIcon({ beat }: { beat: Beat }) {
  if (beat.kind !== "action") return null;
  const Icon = ICONS[beat.action.kind];
  return <Icon aria-hidden className="size-icon-sm" />;
}

function Facts({ editor, step }: { editor: Editor; step: Step }) {
  const inStep = ({ start }: { start: number }) =>
    start >= step.start && start < step.end;
  const said = editor.isVoiced ? SPEECH.filter(inStep).length : 0;
  const marks = editor.marks.filter(inStep).length;
  return (
    <span className="flex items-center gap-1.5 text-caption text-fg-tertiary">
      {editor.beats.filter(inStep).map((beat) => (
        <ActionIcon beat={beat} key={beat.id} />
      ))}
      {said ? (
        <span className="flex items-center gap-0.5">
          <MessageSquareText aria-hidden className="size-icon-sm" />
          {said}
        </span>
      ) : null}
      {marks ? <span className="ms-auto">{marks} annotation</span> : null}
    </span>
  );
}

export function Storyboard({ editor }: { editor: Editor }) {
  const { map, openStep, selection } = editor;
  return (
    <div className="flex min-w-0 grow items-stretch gap-inline">
      {STEPS.map((step, index) => {
        const cutId = `step-${String(index)}`;
        if (editor.cuts.some(({ id }) => id === cutId)) {
          return (
            <button
              className="flex w-20 shrink-0 flex-col items-center justify-center gap-1 rounded-panel border border-dashed border-fg-tertiary text-caption text-fg-tertiary hover:text-fg"
              key={step.start}
              onClick={() => {
                editor.restore(cutId);
              }}
              type="button"
            >
              <Undo2 aria-hidden className="size-icon-sm" />
              {step.title}
            </button>
          );
        }
        const isOpen = openStep === index;
        const isCompact = openStep !== null && !isOpen;
        const isSelected =
          selection?.kind === "step" && selection.index === index;
        const seconds = map.toTime(step.end) - map.toTime(step.start);
        return (
          <div
            className={cn(
              "flex min-w-0 cursor-default flex-col gap-inline rounded-panel p-inset-sm text-start transition-colors",
              isOpen
                ? "grow basis-0 bg-fill"
                : isCompact
                  ? "w-28 shrink-0"
                  : "grow basis-0",
              isSelected && "ring-2 ring-primary",
              !isOpen && !isSelected && "hover:bg-fill",
            )}
            key={step.start}
            onClick={() => {
              editor.select({ index, kind: "step" });
              if (editor.playhead < step.start || editor.playhead >= step.end)
                editor.seek(step.start);
            }}
            onDoubleClick={() => {
              editor.setOpenStep(isOpen ? null : index);
            }}
            role="button"
            tabIndex={0}
          >
            <span className="flex items-baseline gap-inline">
              <Text as="span" className="truncate" variant="label">
                {step.title}
              </Text>
              <span className="ms-auto text-caption whitespace-nowrap text-fg-tertiary tabular-nums">
                {seconds.toFixed(1)} s
              </span>
            </span>
            {isOpen ? (
              <div className="flex min-w-0 items-center gap-separate">
                <Still
                  className="rounded-sm"
                  element={step.element}
                  width={COMPACT_STILL_WIDTH}
                />
                <StepBeats editor={editor} step={step} />
              </div>
            ) : (
              <>
                <Still
                  className="rounded-sm"
                  element={step.element}
                  width={isCompact ? COMPACT_STILL_WIDTH : STILL_WIDTH}
                />
                {isCompact ? null : <Facts editor={editor} step={step} />}
              </>
            )}
          </div>
        );
      })}
    </div>
  );
}
