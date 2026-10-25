// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import { getRulerSettings } from "../../settings/api";
import {
  closeRuler,
  getRulerIslandState,
  returnRulerKeyboard,
  runRulerAction,
  setRulerTolerance,
  setRulerTool,
} from "../api";

import type { RulerIslandParts } from "./ruler-island-parts";
import type { RulerStampKeys } from "./ruler-island-tools";
import type { RulerIslandState } from "../../../bindings/RulerIslandState";

const report = (action: string) => (cause: unknown) => {
  console.error(`Could not ${action}`, cause);
};

/**
 * The Ruler as the island shows it, with what its tools do: `isUp` while
 * the Ruler is up, and `parts` from the last state it sent, kept after it
 * goes so the island tucks away as it was rather than emptied. `parts` is
 * `null` until the Ruler has first been up.
 *
 * The island keeps no state of its own: what it shows is the session, sent
 * whenever it changes, so a held key or a press on the island shows the same
 * way. While the Ruler is up, every press on the island gives the keyboard
 * back to it once released, so its keys keep working.
 */
export function useRulerIsland(): {
  isUp: boolean;
  parts: RulerIslandParts | null;
} {
  const [state, setState] = useState<RulerIslandState | null>(null);
  const [last, setLast] = useState<RulerIslandState | null>(null);
  if (state && state !== last) setLast(state);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    void getRulerIslandState().then((current) => {
      if (!disposed && !isToldRef.current) setState(current);
    });
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("ruler://island", (payload) => {
    isToldRef.current = true;
    setState(payload as RulerIslandState | null);
  });
  // Escape is a global shortcut while the Ruler is up, handed to this
  // window, which is up whatever the island shows.
  useTauriEvent("ruler://dismiss-requested", () => {
    closeRuler().catch(report("close the Ruler"));
  });

  const isUp = state !== null;
  useEffect(() => {
    if (!isUp) return;
    const giveBack = () => {
      returnRulerKeyboard().catch(() => undefined);
    };
    document.addEventListener("pointerup", giveBack);
    return () => {
      document.removeEventListener("pointerup", giveBack);
    };
  }, [isUp]);

  // The stamp keys the measure note names, read as the Ruler comes up, so a
  // binding changed in Settings since is the one shown.
  const [stampKeys, setStampKeys] = useState<RulerStampKeys | null>(null);
  useEffect(() => {
    if (!isUp) return;
    let disposed = false;
    getRulerSettings()
      .then(({ bindings }) => {
        if (disposed) return;
        setStampKeys({
          horizontal: bindings.stampHorizontal ?? null,
          vertical: bindings.stampVertical ?? null,
        });
      })
      .catch(report("read the Ruler's stamp keys"));
    return () => {
      disposed = true;
    };
  }, [isUp]);

  const shown = state ?? last;
  if (!shown) return { isUp, parts: null };
  return {
    isUp,
    parts: {
      onAction: (action) => {
        runRulerAction(action).catch(report(`run the Ruler's ${action}`));
      },
      onClose: () => {
        closeRuler().catch(report("close the Ruler"));
      },
      onTolerance: (tolerance) => {
        setRulerTolerance(tolerance).catch(report("set the edge tolerance"));
      },
      onTool: (tool) => {
        setRulerTool(tool).catch(report("pick a Ruler tool"));
      },
      stampKeys,
      state: shown,
    },
  };
}
