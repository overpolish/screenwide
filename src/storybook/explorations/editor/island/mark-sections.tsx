// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Trash2 } from "lucide-react";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { Button } from "../../../../primitives/button/button";
import { aimedAngle, PEN_NAMES } from "../model/annotation-style";
import { type TimedMark } from "../model/marks";
import { type Editor } from "../model/use-editor";

import { PenOptions } from "./style-controls";

// What the island holds for a selected annotation: the live overlay's own
// choices for its kind, which change it alone, and a press to draw every
// annotation of its kind the same way. When it shows is its bar on the
// timeline.

export function MarkActions({ editor }: { editor: Editor }) {
  return (
    <ConfirmActionButton
      confirmLabel="Confirm Delete"
      icon={<Trash2 aria-hidden />}
      label="Delete"
      onConfirm={editor.remove}
      variant="ghost"
    />
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
        <Button onPress={editor.matchAll}>
          {`Apply to All ${PEN_NAMES[mark.pen].many}`}
        </Button>
      </div>
    </div>
  );
}
