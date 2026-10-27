// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Hourglass, Pin, Scissors, Trash2 } from "lucide-react";
import { type ReactNode } from "react";

import {
  IslandButton,
  IslandTextButton,
} from "../../../../primitives/island/island-controls";
import { type Pen } from "../../ideas/looks";
import { presetLabel } from "../editor-data";
import { type AnnotationStyle, PEN_NAMES } from "../model/annotation-style";
import { type TimedMark } from "../model/marks";
import { DEAD_AIR_S } from "../model/timeline-model";
import { type Editor } from "../model/use-editor";
import { beatTitle } from "../recording";

import { IslandConfirmButton } from "./island-confirm-button";
import { type IslandContent, RowTitle } from "./island-parts";
import { MarkActions, MarkSection } from "./mark-sections";
import { modeContent } from "./mode-content";
import { MomentPlay, MomentSection } from "./moment-section";
import { type Expanded } from "./overlay-parts";
import {
  BeatActions,
  PauseSection,
  SceneClipSection,
} from "./selection-sections";
import { ColorButton, ColorOptions, PenOptions } from "./style-controls";

// What the island holds for whatever is in hand: a row saying what it is,
// with its actions, and the choices hanging beneath.

const HINTS: Record<Pen, string> = {
  arrow: "Drag toward what it points at.",
  counter: "Click what it counts.",
  focus: "Click what to keep in focus.",
};

const markName = (mark: TimedMark) =>
  `${PEN_NAMES[mark.pen].one} ${mark.pen === "arrow" ? "to" : "on"} ${mark.element.label}`;

/** A style on the island as the annotate island lays it out: the colour in
 * hand in the row, and under it what the pen draws with, or, with the
 * colour pressed, the colours in its place under the colour's end. */
function styleParts({
  colors,
  editor,
  key,
  onChange,
  options,
  pen,
  style,
}: {
  colors: Expanded;
  editor: Editor;
  key: string;
  onChange: (change: Partial<AnnotationStyle>) => void;
  options: ReactNode;
  pen: TimedMark["pen"];
  style: AnnotationStyle;
}) {
  const isOpen = pen !== "focus" && colors.isOpen(key);
  return {
    colorButton:
      pen === "focus" ? null : (
        <ColorButton
          color={style.color}
          isOpen={isOpen}
          onPress={() => {
            colors.toggle(key);
          }}
        />
      ),
    section: isOpen
      ? {
          anchor: "end" as const,
          key: `${key}-colors`,
          node: (
            <ColorOptions
              editor={editor}
              onChange={(color) => {
                onChange({ color });
              }}
              value={style.color}
            />
          ),
        }
      : { anchor: "start" as const, key: `${key}-pen`, node: options },
  };
}

function selectionContent(
  editor: Editor,
  colors: Expanded,
): IslandContent | null {
  const { selection } = editor;
  if (selection?.kind === "mark") {
    const mark = editor.marks.find(({ id }) => id === selection.id);
    if (!mark) return null;
    const key = `mark-${String(mark.id)}`;
    const { colorButton, section } = styleParts({
      colors,
      editor,
      key,
      onChange: editor.updateMark,
      options: <MarkSection editor={editor} mark={mark} />,
      pen: mark.pen,
      style: mark.style,
    });
    return {
      actions: (
        <>
          {colorButton}
          <MarkActions editor={editor} />
        </>
      ),
      from: { kind: "strip" },
      key,
      section,
      title: <RowTitle title={markName(mark)} />,
    };
  }
  if (selection?.kind === "beat") {
    const beat = editor.beats.find(({ id }) => id === selection.id);
    if (!beat) return null;
    const isMoment = beat.kind === "cue" && beat.cue === "moment";
    return {
      actions: (
        <>
          {isMoment ? <MomentPlay beat={beat} editor={editor} /> : null}
          <BeatActions editor={editor} key={beat.id} />
        </>
      ),
      from: { kind: "strip" },
      key: `beat-${beat.id}`,
      section: isMoment
        ? {
            key: `beat-${beat.id}-words`,
            node: <MomentSection beat={beat} editor={editor} />,
          }
        : undefined,
      title: <RowTitle title={beatTitle(beat)} />,
    };
  }
  if (selection?.kind === "scene") {
    const clip = editor.scenes.find(({ id }) => id === selection.id);
    if (!clip) return null;
    return {
      actions: (
        <IslandConfirmButton
          confirmLabel="Confirm Delete"
          label="Delete the scene"
          onConfirm={editor.remove}
        >
          <Trash2 aria-hidden className="size-icon" />
        </IslandConfirmButton>
      ),
      from: { kind: "strip" },
      key: clip.id,
      section: {
        key: clip.id,
        node: <SceneClipSection clip={clip} editor={editor} />,
      },
      title: <RowTitle title={presetLabel(clip.preset)} />,
    };
  }
  if (selection?.kind === "pause") {
    const pause = editor.map.pauses.find(({ id }) => id === selection.id);
    if (!pause) return null;
    const isKept =
      (editor.settings[pause.id] as Editor["settings"][string] | undefined)
        ?.kind === "keep";
    return {
      actions: (
        <IslandConfirmButton
          confirmLabel="Confirm Cut"
          label="Cut the pause"
          onConfirm={editor.remove}
        >
          <Scissors aria-hidden className="size-icon" />
        </IslandConfirmButton>
      ),
      from: { kind: "strip" },
      key: pause.id,
      section: {
        key: pause.id,
        node: <PauseSection editor={editor} id={pause.id} />,
      },
      title: (
        <RowTitle
          glyph={
            isKept ? (
              <Pin aria-hidden className="size-icon" />
            ) : (
              <Hourglass aria-hidden className="size-icon" />
            )
          }
          title={pause.still > DEAD_AIR_S ? "Dead air" : "Pause"}
        />
      ),
    };
  }
  if (selection?.kind === "range") {
    // Drawn with ⌥ held, it was drawn to be cut: one press, which undo
    // takes back, rather than a confirmation.
    return {
      actions: (
        <IslandButton aria-label="Cut it and its time" onPress={editor.remove}>
          <Scissors aria-hidden className="size-icon" />
        </IslandButton>
      ),
      from: { kind: "strip" },
      key: `range-${String(selection.start)}`,
      title: <RowTitle title="Stretch" />,
    };
  }
  return null;
}

export function contentOf(
  editor: Editor,
  expanded: Expanded,
): IslandContent | null {
  const { tool } = editor;
  if (tool !== "select") {
    const key = `pen-${tool}`;
    const onChange = (change: Partial<AnnotationStyle>) => {
      editor.setPenStyle(tool, change);
    };
    // The pen's own style, which the annotations it places next take.
    const { colorButton, section } = styleParts({
      colors: expanded,
      editor,
      key,
      onChange,
      options: (
        <PenOptions
          aimed={225}
          onChange={onChange}
          pen={tool}
          style={editor.penStyles[tool]}
        />
      ),
      pen: tool,
      style: editor.penStyles[tool],
    });
    return {
      actions: (
        <>
          {colorButton}
          <IslandTextButton
            color="primary"
            onPress={() => {
              editor.pickTool("select");
            }}
          >
            Done
          </IslandTextButton>
        </>
      ),
      from: { kind: "tool", tool },
      key,
      section,
      title: <RowTitle note={HINTS[tool]} title={PEN_NAMES[tool].one} />,
    };
  }
  return selectionContent(editor, expanded) ?? modeContent(editor, expanded);
}
