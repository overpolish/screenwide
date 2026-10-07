// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { hidePopupPanel, showPopupPanel } from "../../../popup-panel/api";
import {
  activePopupPanel,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "../../../popup-panel/store";
import { boundsAnchor } from "../../../popup-panel/use-popup-menu";

import { RecordingMoment } from "./recording-moments";
import { MOMENT_NOTE_PLAYING_EVENT } from "./use-note-playback";

const NOTE_PANEL_WIDTH = 260;
/** A guess the panel corrects once it has measured itself. */
const NOTE_PANEL_HEIGHT = 44;
const ANCHOR_GAP = 4;

/**
 * Opens `moment`'s voice note in the shared panel window, centred under the
 * pin at `pin`, or puts it away if it is the one already showing. The panel
 * is a window of its own so the native preview cannot cover it.
 */
export async function toggleMomentNote(
  artifactId: number,
  moment: RecordingMoment,
  pin: DOMRect,
) {
  if (!moment.note || !isTauri()) return;
  const id = `moment-note:${artifactId.toString()}:${moment.index.toString()}`;
  const state = usePopupPanelStore.getState();
  const current = activePopupPanel(state, SHARED_POPUP_PANEL);
  if (current?.id === id) {
    state.close(SHARED_POPUP_PANEL);
    await hidePopupPanel(false, SHARED_POPUP_PANEL);
    return;
  }
  state.open(SHARED_POPUP_PANEL, {
    content: {
      artifactId,
      color: moment.color,
      durationMs: moment.note.durationMs,
      kind: "note",
      moment: moment.index,
      name: moment.name,
      waveform: moment.note.waveform,
    },
    focusContents: false,
    id,
    label: `${moment.name} voice note`,
  });
  await showPopupPanel({
    anchor: boundsAnchor(pin),
    fitted: true,
    focusContents: false,
    offset: new LogicalPosition(
      pin.left + pin.width / 2 - NOTE_PANEL_WIDTH / 2,
      pin.bottom + ANCHOR_GAP,
    ),
    parentWindowLabel: getCurrentWindow().label,
    size: new LogicalSize(NOTE_PANEL_WIDTH, NOTE_PANEL_HEIGHT),
    triggerId: id,
  });
}

/** Pauses the timeline whenever a voice note starts playing, so the two are
 * never heard over each other. */
export function usePauseForMomentNotes(pause: () => void) {
  const pauseRef = useRef(pause);
  pauseRef.current = pause;
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen(MOMENT_NOTE_PLAYING_EVENT, () => {
      pauseRef.current();
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}
