// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Captions,
  LayoutPanelLeft,
  type LucideIcon,
  MousePointer2,
  MousePointerClick,
  Palette,
  Proportions,
} from "lucide-react";
import { type RefObject } from "react";

import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { Text } from "../../../../primitives/text/text";
import { Toolbar } from "../../../../primitives/toolbar/toolbar";
import { LookPreview } from "../../ideas/look-picker";
import { LOOKS, type Pen } from "../../ideas/looks";
import { PenIcon } from "../../ideas/pen-toolbar";
import { type Editor, type Mode, type Tool } from "../model/use-editor";

// The core tools in one slim column at the workspace's left edge: the pens,
// drawn in the project's look so each is its own preview, then the modes,
// then the look itself. Whatever a tool needs opens beside its button
// (`tool-panel.tsx`), so the column never grows rows of its own.

export type Owner = Exclude<Mode, null> | Tool;
export type Anchors = RefObject<Map<Owner, HTMLElement>>;

const PENS: { label: string; pen: Pen }[] = [
  { label: "Counter", pen: "counter" },
  { label: "Arrow", pen: "arrow" },
  { label: "Focus", pen: "focus" },
];

type ModeTool = { icon: LucideIcon; label: string; mode: Exclude<Mode, null> };

const MODES: Record<Editor["kind"], ModeTool[]> = {
  audio: [{ icon: Captions, label: "Captions", mode: "captions" }],
  recording: [
    { icon: Proportions, label: "Frame", mode: "frame" },
    { icon: LayoutPanelLeft, label: "Scene", mode: "scene" },
    { icon: MousePointerClick, label: "Pointer", mode: "cursor" },
    { icon: Captions, label: "Captions", mode: "captions" },
  ],
  screenshot: [{ icon: Proportions, label: "Frame", mode: "frame" }],
};

function Rule() {
  return <span aria-hidden className="mx-auto my-1 h-px w-4 bg-divider" />;
}

export function ToolBar({
  anchors,
  editor,
}: {
  anchors: Anchors;
  editor: Editor;
}) {
  const anchor = (owner: Owner) => (element: HTMLElement | null) => {
    if (element) anchors.current.set(owner, element);
  };
  const toggleMode = (mode: Exclude<Mode, null>) => {
    editor.setMode(editor.mode === mode ? null : mode);
  };
  const hasPicture = editor.kind !== "audio";
  return (
    <div className="flex shrink-0 flex-col">
      {hasPicture ? (
        <>
          <Toolbar aria-label="Pens" orientation="vertical">
            <IconToggleButton
              aria-label="Select"
              isSelected={editor.tool === "select" && !editor.mode}
              onChange={() => {
                editor.pickTool("select");
              }}
              ref={anchor("select")}
            >
              <MousePointer2 />
            </IconToggleButton>
            {PENS.map(({ label, pen }) => (
              <IconToggleButton
                aria-label={label}
                isSelected={editor.tool === pen}
                key={pen}
                onChange={() => {
                  editor.pickTool(editor.tool === pen ? "select" : pen);
                }}
                ref={anchor(pen)}
              >
                <span className="scale-[0.8]">
                  <PenIcon lookId={editor.lookId} pen={pen} />
                </span>
              </IconToggleButton>
            ))}
          </Toolbar>
          <Rule />
        </>
      ) : null}
      <Toolbar aria-label="Modes" orientation="vertical">
        {MODES[editor.kind].map(({ icon: Icon, label, mode }) => (
          <IconToggleButton
            aria-label={label}
            isSelected={editor.mode === mode}
            key={mode}
            onChange={() => {
              toggleMode(mode);
            }}
            ref={anchor(mode)}
          >
            <Icon />
          </IconToggleButton>
        ))}
      </Toolbar>
      {hasPicture ? (
        <>
          <Rule />
          <Toolbar aria-label="Look" orientation="vertical">
            <IconToggleButton
              aria-label="Project look"
              isSelected={editor.mode === "look"}
              onChange={() => {
                toggleMode("look");
              }}
              ref={anchor("look")}
            >
              <Palette />
            </IconToggleButton>
          </Toolbar>
        </>
      ) : null}
    </div>
  );
}

const HINTS: Record<Pen, string> = {
  arrow: "Drag toward what it points at.",
  counter: "Click what it counts.",
  focus: "Click what to keep in focus.",
};

export function PenHint({ editor, pen }: { editor: Editor; pen: Pen }) {
  return (
    <div className="flex w-64 items-center gap-separate p-inset-sm ps-inset">
      <Text className="grow" variant="caption">
        {HINTS[pen]}
      </Text>
      <Button
        onPress={() => {
          editor.pickTool("select");
        }}
      >
        Done
      </Button>
    </div>
  );
}

/** The project's look, chosen once, shown as itself. */
export function LookPanel({ editor }: { editor: Editor }) {
  return (
    <div className="flex w-80 flex-col gap-2 p-3">
      <Text variant="caption">
        Every annotation is drawn in the project look
      </Text>
      <div className="grid grid-cols-3 gap-1.5">
        {LOOKS.map(({ id, label }) => (
          <button
            aria-pressed={editor.lookId === id}
            className={cn(
              "flex flex-col gap-1 rounded-[10px] p-1 text-caption [&_svg]:h-12 [&_svg]:w-full",
              editor.lookId === id
                ? "bg-selected text-fg"
                : "text-fg-secondary hover:bg-fill",
            )}
            key={id}
            onClick={() => {
              editor.setLookId(id);
            }}
            type="button"
          >
            <LookPreview id={id} />
            {label}
          </button>
        ))}
      </div>
    </div>
  );
}
