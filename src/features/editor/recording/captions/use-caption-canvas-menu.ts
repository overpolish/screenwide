// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { t } from "../../../../i18n/i18n";
import {
  pointerAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import {
  breakIn,
  canBreakIn,
  withBreak,
  withChoice,
} from "../../tool-panels/captions/caption-word-choices";
import {
  CaptionMenuAction,
  captionMenuItems,
} from "../../tool-panels/captions/use-caption-menu";
import { middleOf } from "../../tool-panels/transcript/transcript-words";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { restoreRecordingTranscriptSpelling } from "../transcript/recording-timeline-transcript";
import { TranscriptTools } from "../transcript/use-transcript-tools";

import { CaptionRetype, captionTypedRestore } from "./caption-typed-restore";
import { CaptionsPatch } from "./recording-captions";

import type { RecordingCaptions } from "../../../../bindings/RecordingCaptions";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import type { TranscriptRange } from "../../tool-panels/tool-panel-transcript";

const MENU_WIDTH = 200;

/** The Captions tool, as far as editing on the picture reaches it: the
 * captions the edit holds, and a change to them, after an edit made to the
 * rest of the edit where given. */
export type CanvasCaptions = {
  change: (
    patch: CaptionsPatch,
    before?: (edit: RecordingTimelineEdit) => RecordingTimelineEdit,
  ) => void;
  snapshot: RecordingCaptions | null;
};

/** A right press on a word of the caption showing on the picture; on one
 * being typed into, `typed` is the word's bytes in the typed text. */
type CaptionWordMenu = {
  before: TranscriptRange | null;
  isKeyWord: boolean | null;
  sessionId: number;
  startsCaption: boolean;
  typed: {
    end: number;
    runStart: number;
    runText: string;
    start: number;
  } | null;
  word: TranscriptRange;
  x: number;
  y: number;
};

/**
 * A right press on a word of the caption on the picture opens the Captions
 * panel's word menu, acting on that word. While the caption is typed into,
 * the word is one as typed, and the menu acts on the typing: Hide takes the
 * word out, Restore spelling types back what was heard for it, and Edit is
 * left out, since the caption is already open.
 */
export function useCaptionCanvasMenu(
  captions: CanvasCaptions,
  transcript: Pick<TranscriptTools, "heard" | "snapshot">,
) {
  const editGesture = useEditorEditGesture();
  const menuRef = useRef<CaptionWordMenu | null>(null);
  /** What Restore spelling types for the word the menu is open on, while
   * its caption is typed into. */
  const restoreRef = useRef<CaptionRetype | null>(null);
  const actRef = useRef<(action: CaptionMenuAction) => void>(() => undefined);
  const openRef = useRef<(menu: CaptionWordMenu) => void>(() => undefined);
  const openMenu = usePopupMenu({
    idPrefix: "caption-canvas:",
    label: t("editor-panels-captions-actions"),
    mode: "menu",
    onSelect: (itemId) => {
      actRef.current(itemId as CaptionMenuAction);
    },
    showsAllItems: true,
    width: MENU_WIDTH,
  });

  const shown = captions.snapshot;
  const durationMs = transcript.snapshot?.durationMs ?? 0;
  const breaks = shown?.breaks ?? [];
  const choices = shown?.wordChoices ?? [];

  openRef.current = (menu) => {
    if (!shown) return;
    menuRef.current = menu;
    const gapAt = menu.before
      ? { after: menu.word, before: menu.before, durationMs }
      : null;
    const gap =
      gapAt && canBreakIn(gapAt)
        ? (breakIn(breaks, gapAt)?.kind ??
          (menu.startsCaption ? "break" : "none"))
        : null;
    // A word being typed is already open, and reads as the typing spells
    // it: corrected wherever that is not what was heard.
    const typing = menu.typed !== null;
    restoreRef.current = menu.typed
      ? captionTypedRestore(
          { start: menu.typed.runStart, text: menu.typed.runText },
          menu.typed.start,
          transcript.heard(menu.word),
        )
      : null;
    const items = captionMenuItems({
      gap,
      isCorrected: typing
        ? restoreRef.current !== null
        : (transcript.snapshot?.corrections ?? []).some(
            (correction) =>
              correction.startMs < menu.word.endMs &&
              menu.word.startMs < correction.endMs,
          ),
      isKeyWord: menu.isKeyWord,
      isShown: true,
    })
      .filter((item) => !typing || item.id !== "edit")
      // The panel's keys act on its text, not on the picture.
      .map(({ shortcut: _shortcut, ...item }) => item);
    void openMenu({
      anchor: pointerAnchor(menu.x, menu.y),
      context: "word",
      items,
    });
  };

  /** Types `text` over bytes `start` to `end` of the caption being typed
   * into, as typing would. */
  const retype = (
    { sessionId }: CaptionWordMenu,
    { end, start, text }: CaptionRetype,
  ) => {
    void invoke("retype_caption_text", { end, sessionId, start, text }).catch(
      (cause: unknown) => {
        console.error("Could not change the caption being typed", cause);
      },
    );
  };

  actRef.current = (action) => {
    const menu = menuRef.current;
    if (!menu || !shown) return;
    const word = menu.word;
    switch (action) {
      case "edit":
        void invoke("edit_recording_caption", {
          sessionId: menu.sessionId,
          wordMs: middleOf(word),
        }).catch((cause: unknown) => {
          console.error("Could not open the caption for typing", cause);
        });
        return;
      case "restore-spelling":
        // In the typing, the word puts back what was heard for it, which
        // the typing then corrects back to.
        if (menu.typed) {
          if (restoreRef.current) retype(menu, restoreRef.current);
          return;
        }
        editGesture.beginGesture();
        captions.change({}, (edit) =>
          restoreRecordingTranscriptSpelling(edit, word, durationMs),
        );
        editGesture.endGesture();
        return;
      case "hide": {
        // A word being typed goes from the typing, which hides it.
        if (menu.typed) {
          retype(menu, {
            end: menu.typed.end,
            start: menu.typed.start,
            text: "",
          });
          return;
        }
        // A word the rules leave out shows only by a choice to show it;
        // taking that choice away hides it again, and keeps no choice
        // that changes nothing.
        const shownByHand = choices.some(
          (choice) =>
            choice.kind === "shown" &&
            choice.sourceAt >= word.startMs / durationMs &&
            choice.sourceAt <= word.endMs / durationMs,
        );
        captions.change({
          wordChoices: withChoice(
            choices,
            { durationMs, word },
            shownByHand ? null : "hidden",
          ),
        });
        return;
      }
      case "show":
        return;
      case "make-key-word":
      case "not-key-word":
        captions.change({
          emphasisChoices: withChoice(
            shown.emphasisChoices ?? [],
            { durationMs, word },
            action === "make-key-word" ? "emphasized" : "plain",
          ),
        });
        return;
      case "clear-break":
      case "join":
      case "split":
        if (!menu.before) return;
        captions.change({
          breaks: withBreak(
            breaks,
            { after: word, before: menu.before, durationMs },
            action === "clear-break" ? null : action,
          ),
        });
    }
  };

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void listen<CaptionWordMenu>(
      "preview://caption-context-menu",
      ({ payload }) => {
        if (!disposed) openRef.current(payload);
      },
    )
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch((cause: unknown) => {
        console.error("Could not listen for the caption word menu", cause);
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
