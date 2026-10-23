// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion } from "motion/react";
import { type ReactNode, useLayoutEffect, useState } from "react";

import { type Editor } from "../model/use-editor";

import {
  CaptionsPanel,
  FramePanel,
  PointerPanel,
  ScenePanel,
} from "./mode-panels";
import { BeatPanel, MarkPanel, StepPanel } from "./selection-panels";
import { type Anchors, LookPanel, type Owner, PenHint } from "./tool-bar";

// The core options for whatever is in hand, beside the toolbar button that
// owns them: a pen's hint beside the pen, a selected annotation's choices
// beside the pen that draws it, a mode's settings beside the mode. A beat or
// a step comes from the timeline, so its panel sits at the foot of the
// workspace, over the timeline. It is one panel that moves and reshapes as
// the owner changes, and goes when nothing is in hand. In the app it is a
// native panel window, as everything floating is; here it borrows that
// window's material.

const reshape = { bounce: 0.15, duration: 0.4, type: "spring" } as const;

type Content = { key: string; node: ReactNode; owner: Owner | "timeline" };

function contentOf(editor: Editor): Content | null {
  if (editor.tool !== "select")
    return {
      key: `pen-${editor.tool}`,
      node: <PenHint editor={editor} pen={editor.tool} />,
      owner: editor.tool,
    };
  const { selection } = editor;
  if (selection?.kind === "mark") {
    const mark = editor.marks.find(({ id }) => id === selection.id);
    if (mark)
      return {
        key: `mark-${String(mark.id)}`,
        node: <MarkPanel editor={editor} mark={mark} />,
        owner: mark.pen,
      };
  }
  if (selection?.kind === "beat")
    return {
      key: `beat-${selection.id}`,
      node: <BeatPanel editor={editor} id={selection.id} />,
      owner: "timeline",
    };
  if (selection?.kind === "step")
    return {
      key: `step-${String(selection.index)}`,
      node: <StepPanel editor={editor} index={selection.index} />,
      owner: "timeline",
    };
  const panels = {
    captions: <CaptionsPanel editor={editor} />,
    cursor: <PointerPanel editor={editor} />,
    frame: <FramePanel editor={editor} />,
    look: <LookPanel editor={editor} />,
    scene: <ScenePanel editor={editor} />,
  };
  return editor.mode
    ? { key: editor.mode, node: panels[editor.mode], owner: editor.mode }
    : null;
}

/** Where each toolbar button is, as a distance from the workspace's top,
 * measured whenever the workspace changes size. The workspace comes as an
 * element, not a ref: this panel sits inside it, so its effects run before
 * the workspace's ref is attached. */
function useAnchorTops(anchors: Anchors, workspace: HTMLElement | null) {
  const [tops, setTops] = useState<Map<Owner, number>>(() => new Map());
  useLayoutEffect(() => {
    if (!workspace) return;
    const observer = new ResizeObserver(() => {
      const origin = workspace.getBoundingClientRect().top;
      setTops(
        new Map(
          [...anchors.current].map(([owner, button]) => [
            owner,
            Math.max(12, button.getBoundingClientRect().top - origin - 6),
          ]),
        ),
      );
    });
    observer.observe(workspace);
    return () => {
      observer.disconnect();
    };
  }, [anchors, workspace]);
  return tops;
}

export function ToolPanel({
  anchors,
  editor,
  workspace,
}: {
  anchors: Anchors;
  editor: Editor;
  workspace: HTMLElement | null;
}) {
  const content = contentOf(editor);
  const tops = useAnchorTops(anchors, workspace);
  const isAtFoot = content?.owner === "timeline";
  const top =
    content && content.owner !== "timeline"
      ? (tops.get(content.owner) ?? 12)
      : 12;
  return (
    <AnimatePresence>
      {content ? (
        <motion.div
          animate={{ opacity: 1, scale: 1, x: 0 }}
          className="absolute start-3 z-20 overflow-hidden rounded-panel bg-surface text-fg shadow-lg inset-ring inset-ring-divider"
          exit={{ opacity: 0, scale: 0.96, x: -8 }}
          initial={{ opacity: 0, scale: 0.96, x: -8 }}
          layout
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
          style={{
            borderRadius: 16,
            bottom: isAtFoot ? 12 : "auto",
            top: isAtFoot ? "auto" : top,
            transformOrigin: isAtFoot ? "left bottom" : "left top",
          }}
          transition={reshape}
        >
          <AnimatePresence initial={false} mode="popLayout">
            <motion.div
              animate={{ filter: "blur(0px)", opacity: 1 }}
              exit={{ filter: "blur(6px)", opacity: 0 }}
              initial={{ filter: "blur(6px)", opacity: 0 }}
              key={content.key}
              layout="position"
              transition={{ duration: 0.18 }}
            >
              {content.node}
            </motion.div>
          </AnimatePresence>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
