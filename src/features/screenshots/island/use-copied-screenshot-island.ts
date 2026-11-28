// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import {
  dismissCopiedScreenshot,
  editCopiedScreenshot,
  getCopiedScreenshot,
  saveCopiedScreenshot,
} from "../api";

import type { CopiedScreenshot } from "../../../bindings/CopiedScreenshot";

/** How long the island stays out for a copied shot, and again once the
 * pointer leaves it, in ms. */
const SHOWN_MS = 5_000;
/** How long it stays to say the shot was saved, in ms. */
const SAVED_MS = 1_500;

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/**
 * A shot just copied as the island shows it: the shot as last sent, kept
 * after it goes so the island tucks away showing it; `null` until a shot
 * has first been copied. The time is counted down
 * here, held while the pointer is over the island.
 */
export function useCopiedScreenshotIsland() {
  const [shot, setShot] = useState<CopiedScreenshot | null>(null);
  const [last, setLast] = useState<CopiedScreenshot | null>(null);
  if (shot && shot !== last) setLast(shot);
  const [isHovered, setIsHovered] = useState(false);
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    getCopiedScreenshot()
      .then((current) => {
        if (!disposed && !isToldRef.current) setShot(current);
      })
      .catch(report("Could not read the copied screenshot"));
    return () => {
      disposed = true;
    };
  }, []);

  useTauriEvent("copied-screenshot://island", (payload) => {
    isToldRef.current = true;
    setShot(payload as CopiedScreenshot | null);
  });

  const id = shot?.id;
  const isSaved = shot?.saved ?? false;
  const dismiss = () => {
    if (id === undefined) return;
    dismissCopiedScreenshot(id).catch(
      report("Could not dismiss the copied screenshot"),
    );
  };

  useTauriEvent("copied-screenshot://dismiss-requested", dismiss);

  useEffect(() => {
    if (id === undefined || isHovered) return;
    const timer = window.setTimeout(
      () => {
        dismissCopiedScreenshot(id).catch(
          report("Could not dismiss the copied screenshot"),
        );
      },
      isSaved ? SAVED_MS : SHOWN_MS,
    );
    return () => {
      window.clearTimeout(timer);
    };
  }, [id, isHovered, isSaved]);

  const shown = shot ?? last;
  return {
    onDismiss: dismiss,
    onEdit: () => {
      if (id === undefined) return;
      editCopiedScreenshot(id).catch(
        report("Could not open the screenshot in the editor"),
      );
    },
    onHoverChange: setIsHovered,
    onSave: () => {
      if (id === undefined) return;
      saveCopiedScreenshot(id).catch(report("Could not save the screenshot"));
    },
    shot: shown && {
      height: shown.height,
      isSaved: shown.saved,
      src: convertFileSrc(shown.path),
      width: shown.width,
    },
  };
}
