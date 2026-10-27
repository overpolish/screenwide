// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Captions,
  Focus,
  Keyboard,
  LayoutPanelLeft,
  type LucideIcon,
  MousePointer2,
  MousePointerClick,
  Proportions,
} from "lucide-react";

import {
  ArrowToolIcon,
  CounterToolIcon,
} from "../../../../features/annotations/annotation-tool-icons";
import { cn } from "../../../../lib/styling";
import { IconToggleButton } from "../../../../primitives/button/icon-button";
import { Toolbar } from "../../../../primitives/toolbar/toolbar";
import { type Pen } from "../../ideas/looks";
import { LEFT_COLUMN } from "../left-column";
import { type Editor, type Mode } from "../model/use-editor";

// The core tools in one slim column at the workspace's left edge: the pens,
// with the glyphs and in the order the annotate island gives them, then the
// modes. Whatever a tool needs opens on the island
// (`island/editor-island.tsx`), so the column never grows rows of its own.
// The transport sits under it, in line with it.

const PENS: { icon: LucideIcon; label: string; pen: Pen }[] = [
  { icon: ArrowToolIcon, label: "Arrow", pen: "arrow" },
  { icon: CounterToolIcon, label: "Counter", pen: "counter" },
  { icon: Focus, label: "Focus", pen: "focus" },
];

type ModeTool = { icon: LucideIcon; label: string; mode: Exclude<Mode, null> };

const MODES: Record<Editor["kind"], ModeTool[]> = {
  audio: [{ icon: Captions, label: "Captions", mode: "captions" }],
  recording: [
    { icon: Proportions, label: "Frame", mode: "frame" },
    { icon: LayoutPanelLeft, label: "Scene", mode: "scene" },
    { icon: MousePointerClick, label: "Pointer", mode: "cursor" },
    { icon: Captions, label: "Captions", mode: "captions" },
    { icon: Keyboard, label: "Shortcuts", mode: "keyboard" },
  ],
  screenshot: [{ icon: Proportions, label: "Frame", mode: "frame" }],
};

function Rule() {
  return <span aria-hidden className="mx-auto my-1 h-px w-4 bg-divider" />;
}

export function ToolBar({ editor }: { editor: Editor }) {
  const toggleMode = (mode: Exclude<Mode, null>) => {
    editor.setMode(editor.mode === mode ? null : mode);
  };
  const hasPicture = editor.kind !== "audio";
  return (
    <div className={cn(LEFT_COLUMN, "flex shrink-0 flex-col items-center")}>
      {hasPicture ? (
        <>
          <Toolbar aria-label="Pens" orientation="vertical">
            <IconToggleButton
              aria-label="Select"
              isSelected={editor.tool === "select" && !editor.mode}
              onChange={() => {
                editor.pickTool("select");
              }}
            >
              <MousePointer2 />
            </IconToggleButton>
            {PENS.map(({ icon: Icon, label, pen }) => (
              <IconToggleButton
                aria-label={label}
                data-island-tool={pen}
                isSelected={editor.tool === pen}
                key={pen}
                onChange={() => {
                  editor.pickTool(editor.tool === pen ? "select" : pen);
                }}
              >
                <Icon />
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
            data-island-tool={mode}
            isSelected={editor.mode === mode}
            key={mode}
            onChange={() => {
              toggleMode(mode);
            }}
          >
            <Icon />
          </IconToggleButton>
        ))}
      </Toolbar>
    </div>
  );
}
