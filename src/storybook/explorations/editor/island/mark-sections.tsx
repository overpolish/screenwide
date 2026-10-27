// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";

import { IslandTextButton } from "../../../../primitives/island/island-controls";
import { aimedAngle, PEN_NAMES } from "../model/annotation-style";
import { type TimedMark } from "../model/marks";
import { type Editor } from "../model/use-editor";

import { IslandConfirmButton } from "./island-confirm-button";
import { PenOptions } from "./style-controls";

// What the island holds for a selected annotation: the live overlay's own
// choices for its kind, which change it alone, and a press to draw every
// annotation of its kind the same way. When it shows is its bar on the
// timeline.

export function MarkActions({ editor }: { editor: Editor }) {
  return (
    <IslandConfirmButton
      confirmLabel="Confirm Delete"
      label="Delete"
      onConfirm={editor.remove}
    >
      <Trash2 aria-hidden className="size-icon" />
    </IslandConfirmButton>
  );
}

export function MarkSection({
  editor,
  mark,
}: {
  editor: Editor;
  mark: TimedMark;
}) {
  return (
    <div className="flex flex-col gap-2">
      <PenOptions
        aimed={aimedAngle(mark.element)}
        onChange={editor.updateMark}
        pen={mark.pen}
        style={mark.style}
      />
      <div className="flex justify-end">
        <IslandTextButton onPress={editor.matchAll}>
          {`Apply to All ${PEN_NAMES[mark.pen].many}`}
        </IslandTextButton>
      </div>
    </div>
  );
}
