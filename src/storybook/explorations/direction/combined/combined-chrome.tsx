// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowUpRight,
  Captions,
  ClipboardCopy,
  Crop,
  Keyboard,
  LayoutPanelLeft,
  MousePointer2,
  MousePointerClick,
  Proportions,
  Redo2,
  ScrollText,
  Share,
  SquareDashedMousePointer,
  Trash2,
  Undo2,
} from "lucide-react";
import { Fragment } from "react";

import { cn } from "../../../../lib/styling";
import { PROJECT_NAME } from "../shared/mock-data";

import { Btn, IconBtn } from "./combined-controls";

export type EditorKind = "audio" | "recording" | "screenshot";

/** The window's own bar: the name, then history and what leaves the
 * editor, only as tall as the window buttons need. */
export function TitleBar({ kind }: { kind: EditorKind }) {
  return (
    <header className="flex h-9 shrink-0 items-center gap-1 ps-traffic-lights pe-2">
      <span className="max-w-60 truncate text-[13px] font-semibold" dir="auto">
        {kind === "screenshot"
          ? "Screenwide 2026-08-08 at 14.32.05"
          : PROJECT_NAME}
      </span>
      <span className="ms-auto flex items-center gap-0.5">
        <IconBtn label="Undo">
          <Undo2 />
        </IconBtn>
        <IconBtn isDisabled label="Redo">
          <Redo2 />
        </IconBtn>
        <IconBtn label="Delete">
          <Trash2 />
        </IconBtn>
      </span>
      {kind === "screenshot" ? (
        <Btn variant="default">
          <ClipboardCopy />
          Copy
        </Btn>
      ) : null}
      <Btn className="ms-1" variant="primary">
        <Share />
        Export
      </Btn>
    </header>
  );
}

type Tool = { icon: typeof Crop; label: string };

const GROUPS: Record<EditorKind, Tool[][]> = {
  // A recording without a picture has nothing to put captions on; its words
  // are its transcript.
  audio: [[{ icon: ScrollText, label: "Transcript" }]],
  recording: [
    [
      { icon: MousePointer2, label: "Select" },
      { icon: SquareDashedMousePointer, label: "Marquee" },
    ],
    [
      // The frame's proportions are what the tool changes; scenes arrange
      // the screen and camera, so a layout rather than a clapperboard; and
      // cursor effects show a click, so they do not read as Select.
      { icon: Proportions, label: "Frame" },
      { icon: LayoutPanelLeft, label: "Scene" },
      { icon: Crop, label: "Crop" },
    ],
    [
      { icon: MousePointerClick, label: "Cursor" },
      { icon: Keyboard, label: "Keyboard" },
      { icon: ScrollText, label: "Transcript" },
      { icon: Captions, label: "Captions" },
    ],
  ],
  screenshot: [
    [
      { icon: MousePointer2, label: "Select" },
      { icon: SquareDashedMousePointer, label: "Marquee" },
    ],
    [
      { icon: Proportions, label: "Resize canvas" },
      { icon: Crop, label: "Crop" },
    ],
  ],
};

/**
 * Every picture tool that is not an annotation, in groups, beside the
 * picture. The annotations have one button at the end: it opens the ring,
 * and shows the last annotation tool taken up, so a second press draws
 * another. Holding on the picture opens the same ring where the pointer is.
 */
export function PictureTools({
  isRingOpen = false,
  kind,
}: {
  kind: EditorKind;
  isRingOpen?: boolean;
}) {
  return (
    <nav
      aria-label="Picture tools"
      className="flex w-9 shrink-0 flex-col items-center gap-1 pt-1"
    >
      {GROUPS[kind].map((group, index) => (
        <Fragment key={group[0]?.label}>
          {index > 0 ? <span className="my-1 h-px w-4 bg-(--divider)" /> : null}
          {group.map(({ icon: Icon, label }) => (
            <IconBtn
              className="size-8"
              isActive={label === "Select" && !isRingOpen}
              key={label}
              label={label}
            >
              <Icon />
            </IconBtn>
          ))}
        </Fragment>
      ))}
      {kind === "audio" ? null : (
        <>
          <span className="my-1 h-px w-4 bg-(--divider)" />
          <IconBtn
            className={cn("size-8", isRingOpen && "combined-accent")}
            label="Annotations: Arrow"
          >
            <ArrowUpRight />
          </IconBtn>
        </>
      )}
    </nav>
  );
}
