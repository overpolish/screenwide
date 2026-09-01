// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

import { type GlideAction, type GlideDetection } from "./glide-detection";
import { type GlideFit, GlidePreview } from "./glide-preview";
import { type GlideRegion, sameRegion } from "./glide-regions";

type GlideInputEvent =
  | { detection: GlideDetection; type: "detection" }
  | { anchorX: number; anchorY: number; cancelled: boolean; type: "end" }
  | { sessionId: number; type: "start" };

/** The glided app's icon, resolved off the gesture's path and sent once. */
type GlideIconEvent = { iconPath: null | string; sessionId: number };

/** What the window's move actually achieved, reported once it settles. */
type GlideFitEvent = GlideFit & { sessionId: number };

export function GlideWindow() {
  const [fit, setFit] = useState<GlideFit | null>(null);
  const [iconSrc, setIconSrc] = useState<null | string>(null);
  const [pending, setPending] = useState<GlideAction | null>(null);
  const [readyPulse, setReadyPulse] = useState(0);
  const [region, setRegion] = useState<GlideRegion | null>(null);

  useEffect(() => {
    let disposed = false;
    let stopListening: (() => void) | undefined;
    let stopIconListening: (() => void) | undefined;
    let stopFitListening: (() => void) | undefined;
    // The session id keeps asynchronous icon and fit reports from landing on a
    // preview already showing a different app.
    let sessionId: null | number = null;
    // Where the window was last sent, so a transition that lands back on the
    // region it already occupies - a disarm restoring the region the arm was
    // taken over - asks for no move at all.
    let moved: GlideRegion | null = null;
    // The haptic and the preview's ready breath are one "next gesture is
    // armed" moment - the breath carries it where no trackpad is under hand,
    // and it stays when the haptic is switched off.
    const tickReady = () => {
      setReadyPulse((count) => count + 1);
    };
    const clear = () => {
      moved = null;
      setFit(null);
      setIconSrc(null);
      setPending(null);
      setRegion(null);
    };
    void listen<GlideInputEvent>("glide://input", ({ payload }) => {
      if (payload.type === "start") {
        sessionId = payload.sessionId;
        clear();
        return;
      }
      if (payload.type === "end") {
        sessionId = null;
        clear();
        return;
      }

      const detection = payload.detection;

      // The transitions themselves are silent - the preview is their feedback.
      // The tick says the fingers have rested and the next gesture will land.
      if (detection.becameReady) tickReady();
      if (!detection.changed) return;

      setPending(detection.pending);
      setRegion(detection.region);
      // A destination the window has not reached yet knows nothing about how
      // it will fit: the last settle's report is stale from the moment the
      // region changes, and the preview goes back to the plain fill until the
      // new one lands.
      if (!sameRegion(detection.region, moved)) setFit(null);
      // Keep presentation knowledge of the last applied region so arming an
      // action over it does not discard a still-current fit report.
      if (
        detection.region !== null &&
        detection.pending === null &&
        !sameRegion(detection.region, moved)
      ) {
        moved = detection.region;
      }
    }).then((unlisten) => {
      if (disposed) unlisten();
      else stopListening = unlisten;
    });
    // Extraction runs off the gesture, so the icon arrives on its own beat -
    // possibly after the reveal. The session id is what keeps it from landing
    // on a preview already showing a different app.
    void listen<GlideIconEvent>("glide://icon", ({ payload }) => {
      if (payload.sessionId !== sessionId) return;
      setIconSrc(
        payload.iconPath === null ? null : convertFileSrc(payload.iconPath),
      );
    }).then((unlisten) => {
      if (disposed) unlisten();
      else stopIconListening = unlisten;
    });
    // The move's own answer, once the tween settles: whether the window could
    // take the region it was sent to, and the frame it reached instead.
    void listen<GlideFitEvent>("glide://fit", ({ payload }) => {
      if (payload.sessionId !== sessionId) return;
      setFit({ actual: payload.actual, fits: payload.fits });
    }).then((unlisten) => {
      if (disposed) unlisten();
      else stopFitListening = unlisten;
    });

    return () => {
      disposed = true;
      stopListening?.();
      stopIconListening?.();
      stopFitListening?.();
    };
  }, []);

  return (
    <main className="flex h-screen w-screen items-center justify-center">
      <GlidePreview
        fit={fit}
        iconSrc={iconSrc}
        pending={pending}
        pulse={readyPulse}
        region={region}
      />
    </main>
  );
}
