// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Captions,
  Keyboard,
  LayoutPanelLeft,
  MousePointerClick,
  Proportions,
} from "lucide-react";
import { type ReactNode } from "react";

import { BACKDROPS } from "../editor-data";
import { type Editor, type Mode } from "../model/use-editor";

import { type IslandContent, RowTitle } from "./island-parts";
import { FrameSection, PointerSection, SceneSection } from "./mode-sections";
import { captionParts, keysParts } from "./overlay-options";
import { type Expanded, overlayParts } from "./overlay-parts";

// What the island holds for a mode picked from the toolbar, beside the
// button that opened it, or over the strip where it was opened from there.

const backdropLabel = (editor: Editor) =>
  BACKDROPS.find(({ id }) => id === editor.backdrop)?.label ?? "";

type ModeParts = Pick<IslandContent, "actions" | "section"> & {
  glyph: ReactNode;
  title: string;
  note?: string;
};

function partsOf(
  editor: Editor,
  expanded: Expanded,
  mode: Exclude<Mode, null>,
): ModeParts {
  const icon = "size-icon";
  const toggle = (effect: "captions" | "keys") => () => {
    editor.setEffects({ ...editor.effects, [effect]: !editor.effects[effect] });
  };
  switch (mode) {
    case "captions":
      return editor.isVoiced
        ? {
            glyph: <Captions aria-hidden className={icon} />,
            title: "Captions",
            ...overlayParts({
              expanded,
              isShown: editor.effects.captions,
              key: "captions",
              onToggleShown: toggle("captions"),
              parts: captionParts(editor),
              showLabel: "Show captions",
            }),
          }
        : {
            glyph: <Captions aria-hidden className={icon} />,
            note: "Nothing was said, so there is nothing to caption.",
            title: "No voice",
          };
    case "keyboard":
      return {
        glyph: <Keyboard aria-hidden className={icon} />,
        title: "Shortcuts",
        ...overlayParts({
          expanded,
          isShown: editor.effects.keys,
          key: "keys",
          onToggleShown: toggle("keys"),
          parts: keysParts(editor),
          showLabel: "Show shortcuts",
        }),
      };
    case "cursor":
      return {
        glyph: <MousePointerClick aria-hidden className={icon} />,
        section: { key: mode, node: <PointerSection editor={editor} /> },
        title: "Pointer",
      };
    case "frame":
      return {
        glyph: <Proportions aria-hidden className={icon} />,
        note: backdropLabel(editor),
        section: { key: mode, node: <FrameSection editor={editor} /> },
        title: "Frame",
      };
    case "scene":
      return {
        glyph: <LayoutPanelLeft aria-hidden className={icon} />,
        note: "Add a scene at the playhead.",
        section: { key: mode, node: <SceneSection editor={editor} /> },
        title: "Scene",
      };
  }
}

export function modeContent(
  editor: Editor,
  expanded: Expanded,
): IslandContent | null {
  const { mode } = editor;
  if (!mode) return null;
  const { actions, glyph, note, section, title } = partsOf(
    editor,
    expanded,
    mode,
  );
  return {
    actions,
    from: editor.modeFrom ? { kind: "strip" } : { kind: "tool", tool: mode },
    key: mode,
    section,
    title: <RowTitle glyph={glyph} note={note} title={title} />,
  };
}
