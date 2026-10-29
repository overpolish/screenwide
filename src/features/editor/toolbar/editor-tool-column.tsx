// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Ellipsis, MousePointer2 } from "lucide-react";
import { Fragment, useEffect, useRef, useState } from "react";

import { t } from "../../../i18n/i18n";
import { cn } from "../../../lib/styling";
import { IconButton } from "../../../primitives/button/icon-button";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import { TooltipTrigger } from "../../../primitives/tooltip/tooltip-trigger";
import {
  annotationToolLabel,
  annotationToolName,
} from "../../annotations/annotation-text";
import { EDITOR_LEFT_COLUMN } from "../editor-left-column";
import { ANNOTATION_TOOLS } from "../tool-panels/tool-registry";

import { annotationToolLayout } from "./annotation-tool-layout";
import {
  editorModeGroups,
  type EditorKind,
  type EditorMode,
} from "./editor-modes";
import {
  closeAnnotationToolMenu,
  useAnnotationToolMenu,
} from "./use-annotation-tool-menu";
import { usePenCapacity } from "./use-pen-capacity";

import type { AnnotationKind } from "../../annotations/types";

// The editor's tools in one slim column at the workspace's start edge:
// Select and the pens, then the modes. Whatever a tool needs opens on the
// island, so the column never grows rows of its own. Spotlight is drawn by
// the focus pen, as on the annotate island, so it has no button.
//
// A short window cannot hold every pen. The pens are in the order most used
// first, so the tail goes into a menu at the end of the pens, and the pen
// last taken from it takes the place before the menu's button; the modes
// always show.

const PENS = ANNOTATION_TOOLS.filter(({ id }) => id !== "spotlight");

export type EditorTool = AnnotationKind | "select";

export type EditorToolColumnProps = {
  /** Whether the recording has a camera, which offers the Camera mode. */
  hasCamera: boolean;
  kind: EditorKind;
  /** The mode open, or null. A mode open puts the tool in hand down. */
  mode: EditorMode | null;
  /** Told the mode chosen, or null when the open one is pressed again. */
  onModeChange: (mode: EditorMode | null) => void;
  /** Told the tool taken up; a pen pressed again goes back to Select. */
  onToolChange: (tool: EditorTool) => void;
  /** The tool in hand. */
  tool: EditorTool;
};

export function EditorToolColumn({
  hasCamera,
  kind,
  mode,
  onModeChange,
  onToolChange,
  tool,
}: EditorToolColumnProps) {
  const hasPicture = kind !== "audio";
  const { capacity, columnRef, modesRef, toolsRef } = usePenCapacity(
    PENS.length,
  );
  const [slotted, setSlotted] = useState<(typeof PENS)[number] | null>(null);
  const moreRef = useRef<HTMLButtonElement>(null);
  const openMenu = useAnnotationToolMenu(({ id }) => {
    onToolChange(id);
  });
  // Taking up a tool any other way, by its shortcut or its button, answers
  // the menu as surely as picking from it would.
  useEffect(() => {
    void closeAnnotationToolMenu();
  }, [tool]);

  const layout = annotationToolLayout(PENS, capacity, slotted);
  const inHand = PENS.find(({ id }) => id === tool);
  if (inHand && layout.hidden.includes(inHand)) setSlotted(inHand);
  const shown = layout.slot ? [...layout.visible, layout.slot] : layout.visible;

  return (
    <div
      className={cn(
        "flex h-full shrink-0 flex-col items-center overflow-hidden",
        EDITOR_LEFT_COLUMN,
      )}
      ref={columnRef}
    >
      {hasPicture ? (
        <div ref={toolsRef}>
          <Toolbar
            aria-label={t("editor-toolbar-tools")}
            orientation="vertical"
          >
            <ToolbarTool
              aria-label={
                kind === "screenshot"
                  ? t("editor-tool-select-name")
                  : t("editor-tool-select-clip-name")
              }
              isSelected={tool === "select" && mode === null}
              label={t("editor-tool-select")}
              onChange={() => {
                onToolChange("select");
              }}
              shortcut="V"
            >
              <MousePointer2 />
            </ToolbarTool>
            {shown.map(({ icon: Icon, id, shortcut }) => (
              <span className="inline-flex" data-island-tool={id} key={id}>
                <ToolbarTool
                  aria-label={
                    id === "focus"
                      ? t("annotate-island-focus-name")
                      : annotationToolName(id)
                  }
                  isSelected={tool === id && mode === null}
                  label={annotationToolLabel(id)}
                  onChange={(isSelected) => {
                    onToolChange(isSelected ? id : "select");
                  }}
                  shortcut={shortcut}
                >
                  <Icon />
                </ToolbarTool>
              </span>
            ))}
            {layout.hidden.length > 0 ? (
              <TooltipTrigger tooltip={t("editor-toolbar-more-tools")}>
                <IconButton
                  aria-label={t("editor-toolbar-more-annotation-tools")}
                  onPress={() => {
                    const bounds = moreRef.current?.getBoundingClientRect();
                    if (bounds) void openMenu(layout.hidden, bounds);
                  }}
                  ref={moreRef}
                  variant="ghost"
                >
                  <Ellipsis />
                </IconButton>
              </TooltipTrigger>
            ) : null}
          </Toolbar>
        </div>
      ) : null}
      <div className="flex flex-col items-center" ref={modesRef}>
        {hasPicture ? <Rule /> : null}
        <Toolbar aria-label={t("editor-toolbar-modes")} orientation="vertical">
          {editorModeGroups({ hasCamera, kind }).map((group, index) => (
            <Fragment key={group[0]?.id}>
              {index > 0 ? <Rule /> : null}
              {group.map(({ icon: Icon, id, label, shortcut }) => (
                <span className="inline-flex" data-island-tool={id} key={id}>
                  <ToolbarTool
                    aria-label={label}
                    isSelected={mode === id}
                    label={label}
                    onChange={(isSelected) => {
                      onModeChange(isSelected ? id : null);
                    }}
                    shortcut={shortcut}
                  >
                    <Icon />
                  </ToolbarTool>
                </span>
              ))}
            </Fragment>
          ))}
        </Toolbar>
      </div>
    </div>
  );
}

function Rule() {
  return <span aria-hidden className="mx-auto my-inline h-px w-4 bg-divider" />;
}
