// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";

import { getAnnotateSettings, setAnnotateSettings } from "../settings/api";
import { AnnotateSettings } from "../settings/types";
import { useGeneralSettings } from "../settings/use-general-settings";

import { AnnotateToolbar } from "./annotate-toolbar";
import {
  clearAnnotations,
  dismissAnnotate,
  listenToAnnotateSettings,
  persistAnnotateToolbarPosition,
  resizeAnnotateToolbar,
  undoAnnotation,
} from "./api";
import { useToolbarTyping } from "./use-toolbar-typing";

/** How long the drag has to stand still before the place is kept. A drag
 * reports every step, and each one would otherwise be a settings write. */
const MOVE_SETTLE_MS = 250;

/** How long a dragged edit has to stand still before it is written, for the
 * same reason. A slider reports every step of a drag, and each step would
 * otherwise be a validation, a file write, a tray refresh and a change event
 * the plate follows back - which is what made the controls flicker under the
 * pointer. */
const EDIT_SETTLE_MS = 150;

/** The dress a drag changes a step at a time, whose write can wait for the
 * pointer to settle. Everything else - the tool, the colour, the head - is a
 * single press, and has to reach the overlay before the next stroke does. */
const DRAGGED_EDITS = new Set<keyof AnnotateSettings>([
  "defaultCounterAngle",
  "defaultCounterSize",
  "defaultWidth",
]);

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action}`, cause);
};

/**
 * The toolbar as its own window.
 *
 * It holds no state of its own: what it shows is the annotate settings, and
 * every edit is a write of them. A write elsewhere - the Settings page -
 * arrives as the change event, so the two are never out of step. The colours
 * of your own come from the general settings the editor keeps them in.
 */
export function AnnotateToolbarWindow() {
  const [settings, setSettings] = useState<AnnotateSettings | null>(null);
  // An edit of our own is outstanding while `latest` runs ahead of `settled`.
  // Until it catches up the plate shows the choice under the pointer, and
  // neither a write's answer nor the change event it fires may move it.
  const editRef = useRef({ latest: 0, settled: 0 });
  // The last dress Rust took, which is what a refused edit falls back to.
  const acceptedRef = useRef<AnnotateSettings | null>(null);
  // The editor's kept colours, which the store broadcasts, so one saved while
  // the overlay is up appears here without the toolbar asking again.
  const general = useGeneralSettings();
  useToolbarTyping();

  useEffect(() => {
    let stopped = false;
    let stopListening: (() => void) | undefined;
    getAnnotateSettings()
      .then((current) => {
        if (stopped) return;
        acceptedRef.current = current;
        setSettings(current);
      })
      .catch(report("read the annotate settings"));
    listenToAnnotateSettings((changed) => {
      acceptedRef.current = changed;
      // Our own write echoes back through here too, so an edit still under
      // the pointer keeps what it is showing.
      if (editRef.current.latest === editRef.current.settled)
        setSettings(changed);
    })
      .then((unlisten) => {
        if (stopped) unlisten();
        else stopListening = unlisten;
      })
      .catch(report("follow the annotate settings"));
    return () => {
      stopped = true;
      stopListening?.();
    };
  }, []);

  // A drag moves the window natively, so the place is read back from it once
  // the pointer has settled.
  useEffect(() => {
    let timer = 0;
    let stopped = false;
    let stopListening: (() => void) | undefined;
    getCurrentWindow()
      .onMoved(() => {
        window.clearTimeout(timer);
        timer = window.setTimeout(() => {
          persistAnnotateToolbarPosition().catch(
            report("keep the toolbar's place"),
          );
        }, MOVE_SETTLE_MS);
      })
      .then((unlisten) => {
        if (stopped) unlisten();
        else stopListening = unlisten;
      })
      .catch(report("follow the toolbar's place"));
    return () => {
      stopped = true;
      window.clearTimeout(timer);
      stopListening?.();
    };
  }, []);

  const onSizeChange = useCallback((width: number, height: number) => {
    resizeAnnotateToolbar(width, height).catch(report("size the toolbar"));
  }, []);

  // Optimistic: the control shows the choice at once, and Rust's answer is
  // what the plate settles on, so a refused colour or size puts it back -
  // unless a later edit has arrived, which is the one being shown.
  //
  // A dragged edit's write waits for the pointer to settle, so a slider drag
  // costs one settings write rather than one per step. An edit that carries no
  // drag says so: a typed value has to be in the settings before the press that
  // draws with it, and that press is on the picture, a window away.
  const onChange = (patch: Partial<AnnotateSettings>, immediate = false) => {
    if (!settings) return;
    const next = { ...settings, ...patch };
    setSettings(next);
    editRef.current.latest += 1;
    const seq = editRef.current.latest;
    const write = () => {
      setAnnotateSettings(next)
        .then((saved) => {
          acceptedRef.current = saved;
          if (seq === editRef.current.latest) setSettings(saved);
        })
        .catch((cause: unknown) => {
          report("save the annotate settings")(cause);
          const last = acceptedRef.current;
          if (last && seq === editRef.current.latest) setSettings(last);
        })
        .finally(() => {
          editRef.current.settled = seq;
        });
    };
    const dragged = Object.keys(patch).every((key) =>
      DRAGGED_EDITS.has(key as keyof AnnotateSettings),
    );
    if (!dragged || immediate) {
      write();
      return;
    }
    window.setTimeout(() => {
      // The step a later one overtook has nothing left to write: the drag it
      // belonged to is still going, and its last step is the one that lands.
      if (seq === editRef.current.latest) write();
    }, EDIT_SETTLE_MS);
  };

  if (!settings) return null;

  return (
    <AnnotateToolbar
      onChange={onChange}
      onClear={() => {
        clearAnnotations().catch(report("clear the annotations"));
      }}
      onDone={() => {
        dismissAnnotate().catch(report("leave the overlay"));
      }}
      onSizeChange={onSizeChange}
      onUndo={() => {
        undoAnnotation().catch(report("undo the last annotation"));
      }}
      savedColors={general?.annotationColors}
      settings={settings}
    />
  );
}
