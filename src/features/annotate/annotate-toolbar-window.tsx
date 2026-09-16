// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";

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

/** How long the drag has to stand still before the place is kept. A drag
 * reports every step, and each one would otherwise be a settings write. */
const MOVE_SETTLE_MS = 250;

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
  const [isSaving, setIsSaving] = useState(false);
  // The editor's kept colours, which the store broadcasts, so one saved while
  // the overlay is up appears here without the toolbar asking again.
  const general = useGeneralSettings();

  useEffect(() => {
    let stopped = false;
    let stopListening: (() => void) | undefined;
    getAnnotateSettings()
      .then((current) => {
        if (!stopped) setSettings(current);
      })
      .catch(report("read the annotate settings"));
    listenToAnnotateSettings((changed) => {
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
  // what it settles on, so a refused colour or width puts it back rather than
  // leaving it showing a value the overlay never took.
  const onChange = (patch: Partial<AnnotateSettings>) => {
    if (!settings) return;
    const next = { ...settings, ...patch };
    setSettings(next);
    setIsSaving(true);
    setAnnotateSettings(next)
      .then(setSettings)
      .catch((cause: unknown) => {
        report("save the annotate settings")(cause);
        setSettings(settings);
      })
      .finally(() => {
        setIsSaving(false);
      });
  };

  if (!settings) return null;

  return (
    <AnnotateToolbar
      isDisabled={isSaving}
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
