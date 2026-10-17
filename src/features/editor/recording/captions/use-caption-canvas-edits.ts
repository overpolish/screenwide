// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { withChoice } from "../../tool-panels/captions/caption-word-choices";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { TranscriptTools } from "../transcript/use-transcript-tools";

import {
  CanvasCaptions,
  useCaptionCanvasMenu,
} from "./use-caption-canvas-menu";

import type { CaptionWordChoice } from "../../../../bindings/CaptionWordChoice";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import type {
  TranscriptCorrection,
  TranscriptRange,
} from "../../tool-panels/tool-panel-transcript";

/** A caption typed into on the picture, as Rust compared it with what it
 * showed: all the words spelled another way and taken out whole so far,
 * while the typing goes on and once it has ended. */
type CaptionEditCommit = {
  corrections: TranscriptCorrection[];
  durationMs: number;
  hidden: TranscriptRange[];
  phase: "change" | "end";
  sessionId: number;
  /** Which typing the changes are of. */
  typing: number;
};

/** What the edit held where typing on the picture changes it, as it was
 * when the typing began: each change is made from this again, standing in
 * for the change before. */
type TypingBase = {
  corrections: { value: RecordingTimelineEdit["transcriptCorrections"] } | null;
  typing: number;
  wordChoices: CaptionWordChoice[];
};

/**
 * The caption on the picture, edited where it shows. As it is typed into,
 * its respellings are corrected in the transcript and the words taken out
 * whole hidden from the captions, as the Captions panel would make them, so
 * the panels follow the typing; the whole typing is one step of the edit
 * history. A right press on one of its words opens the panel's word menu,
 * acting on that word; while it is typed into, Hide takes the word out of
 * the typing.
 */
export function useCaptionCanvasEdits(
  captions: CanvasCaptions,
  transcript: Pick<TranscriptTools, "edited" | "heard" | "snapshot">,
) {
  useCaptionCanvasMenu(captions, transcript);
  const editGesture = useEditorEditGesture();
  const commitRef = useRef<(commit: CaptionEditCommit) => void>(
    () => undefined,
  );
  const baseRef = useRef<TypingBase | null>(null);
  const choices = captions.snapshot?.wordChoices ?? [];

  commitRef.current = ({
    corrections,
    durationMs: length,
    hidden,
    phase,
    typing: id,
  }) => {
    let base = baseRef.current;
    // A typing whose end never came, as when the preview closed under it,
    // is over once another begins.
    if (base && base.typing !== id) {
      editGesture.endGesture();
      base = null;
    }
    if (!base) {
      editGesture.beginGesture();
      base = { corrections: null, typing: id, wordChoices: choices };
      baseRef.current = base;
    }
    const typing = base;
    captions.change(
      {
        wordChoices: hidden.reduce(
          (kept, word) =>
            withChoice(kept, { durationMs: length, word }, "hidden"),
          typing.wordChoices,
        ),
      },
      (edit) => {
        typing.corrections ??= { value: edit.transcriptCorrections };
        return transcript.edited(
          { ...edit, transcriptCorrections: typing.corrections.value },
          { corrections, type: "correct-many" },
        );
      },
    );
    if (phase === "end") {
      baseRef.current = null;
      editGesture.endGesture();
    }
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void listen<CaptionEditCommit>(
      "editor://recording-caption-edit",
      ({ payload }) => {
        if (!disposed) commitRef.current(payload);
      },
    )
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch((cause: unknown) => {
        console.error("Could not listen for caption edits", cause);
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
