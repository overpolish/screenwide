// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  getAnnotateSettings,
  listenToAnnotateSettings,
  setAnnotateSettings,
} from "../../settings/api";
import { useGeneralSettings } from "../../settings/use-general-settings";
import { clearAnnotations, dismissAnnotate, undoAnnotation } from "../api";

import type { AnnotatePartsProps } from "./use-annotate-parts";
import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

/** How long a dragged edit has to stand still before it is written. A drag
 * reports every step, and each step would otherwise be a validation, a file
 * write, a tray refresh and a change event the tools follow back, which
 * makes the controls flicker under the pointer. */
const EDIT_SETTLE_MS = 150;

/** The settings a drag changes a step at a time, whose write can wait for
 * the pointer to settle. Everything else (the tool, the colour, the head) is
 * a single press, and has to reach the overlay before the next stroke does. */
const DRAGGED_EDITS = new Set<keyof AnnotateSettings>([
  "defaultCounterAngle",
  "defaultCounterSize",
  "defaultWidth",
  "shapeRadius",
  "spotlightRadius",
]);

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action}`, cause);
};

/** What the annotate tools edit and do, wherever the island shows them. */
export type AnnotateEditing = Omit<AnnotatePartsProps, "initialSection">;

/**
 * The annotate settings as the island's tools edit them, or `null` until
 * they are read.
 *
 * The tools hold no state of their own: what they show is the annotate
 * settings, and every edit is a write of them. A write elsewhere (the
 * Settings page, a tool's letter over the overlay) arrives as the change
 * event, so they are never out of step. The colours of your own come from
 * the general settings the editor keeps them in.
 */
export function useAnnotateEditing(): AnnotateEditing | null {
  const [settings, setSettings] = useState<AnnotateSettings | null>(null);
  // An edit of our own is outstanding while `latest` runs ahead of `settled`.
  // Until it catches up the tools show the choice under the pointer, and
  // neither a write's answer nor the change event it fires may move it.
  const editRef = useRef({ latest: 0, settled: 0 });
  // The settings Rust last took, which a refused edit falls back to and
  // every write is built on.
  const acceptedRef = useRef<AnnotateSettings | null>(null);
  // Our own edits not yet answered. A write carries them over what Rust last
  // took rather than over the shown copy, which stops following the change
  // event while an edit is outstanding: a drag's write landing after a tool
  // key would otherwise put the old tool back.
  const outstandingRef = useRef<Partial<AnnotateSettings>>({});
  // The editor's kept colours, which the store broadcasts, so one saved
  // while the overlay is up appears without asking again.
  const general = useGeneralSettings();

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

  if (!settings) return null;

  // Optimistic: the control shows the choice at once, and Rust's answer is
  // what the tools settle on, so a refused colour or size puts it back,
  // unless a later edit has arrived, which is the one being shown.
  //
  // A dragged edit's write waits for the pointer to settle, so a drag costs
  // one settings write rather than one per step. An edit that carries no
  // drag says so with `immediate`.
  const onChange = (patch: Partial<AnnotateSettings>, immediate = false) => {
    setSettings({ ...settings, ...patch });
    outstandingRef.current = { ...outstandingRef.current, ...patch };
    editRef.current.latest += 1;
    const seq = editRef.current.latest;
    const write = () => {
      setAnnotateSettings({
        ...(acceptedRef.current ?? settings),
        ...outstandingRef.current,
      })
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
          // The latest write carried every edit before it, answered or not.
          if (seq === editRef.current.latest) outstandingRef.current = {};
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

  return {
    onChange,
    onClear: () => {
      clearAnnotations().catch(report("clear the annotations"));
    },
    onDone: () => {
      dismissAnnotate().catch(report("leave the overlay"));
    },
    onUndo: () => {
      undoAnnotation().catch(report("undo the last annotation"));
    },
    savedColors: general?.annotationColors,
    settings,
  };
}
