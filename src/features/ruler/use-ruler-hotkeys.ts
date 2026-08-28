// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import type { Axis } from "./gradient-field";

export function useRulerHotkeys({
  cancelProbe,
  close,
  copyColor,
  copyLatestMeasurement,
  cycleTolerance,
  deleteHovered,
  deleteLatestMeasurement,
  finishProbe,
  redo,
  setNativeCursorRangeActive,
  startProbe,
  toggleCenterlines,
  toggleCrosshair,
  toggleDetectedBoxes,
  undo,
}: {
  cancelProbe: () => void;
  close: () => void;
  copyColor: () => void;
  copyLatestMeasurement: () => void;
  cycleTolerance: () => void;
  /** Deletes whatever a hovered label owns; falsy when nothing is hovered. */
  deleteHovered: () => boolean;
  deleteLatestMeasurement: () => void;
  finishProbe: () => void;
  /** Falsy when the redo stack is empty, so the key stays unhandled. */
  redo: () => boolean;
  setNativeCursorRangeActive: (active: boolean) => void;
  startProbe: (axis: Axis) => boolean;
  toggleCenterlines: () => void;
  toggleCrosshair: () => void;
  /** Debug view: outlines every box the detector found at this tolerance. */
  toggleDetectedBoxes: () => void;
  /** Falsy when the undo stack is empty, so the key stays unhandled. */
  undo: () => boolean;
}) {
  const [guideAxis, setGuideAxis] = useState<Axis>();
  const [probeAxis, setProbeAxis] = useState<Axis>();
  const [radiusActive, setRadiusActive] = useState(false);
  const heldProbeCodeRef = useRef<string | undefined>(undefined);

  useEffect(() => {
    const clearHeldTools = () => {
      setGuideAxis(undefined);
      setProbeAxis(undefined);
      setRadiusActive(false);
      heldProbeCodeRef.current = undefined;
      setNativeCursorRangeActive(false);
      cancelProbe();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      // History first: holding the combo repeats, and neither shortcut may fall
      // through to the plain-key handling below.
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
        if (event.shiftKey ? redo() : undo()) event.preventDefault();
        return;
      }
      // Ctrl+Y redoes too, for Windows habits.
      if (event.ctrlKey && event.code === "KeyY") {
        if (redo()) event.preventDefault();
        return;
      }
      if (
        event.repeat &&
        (event.code === "KeyX" ||
          event.code === "KeyB" ||
          event.code === "KeyC" ||
          event.code === "KeyM" ||
          event.code === "KeyR" ||
          event.code === "KeyT")
      )
        return;
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      } else if (event.key === "Tab") {
        event.preventDefault();
        copyColor();
      } else if (event.code === "KeyX") {
        toggleCrosshair();
      } else if (event.code === "KeyC" && (event.metaKey || event.ctrlKey)) {
        copyLatestMeasurement();
      } else if (event.code === "KeyB") {
        toggleDetectedBoxes();
      } else if (event.code === "KeyM") {
        toggleCenterlines();
      } else if (event.code === "KeyT") {
        cycleTolerance();
      } else if (
        event.code === "KeyR" &&
        !event.metaKey &&
        !event.ctrlKey &&
        !event.altKey
      ) {
        event.preventDefault();
        setRadiusActive(true);
      } else if (event.key === "Backspace" || event.key === "Delete") {
        if (!deleteHovered()) deleteLatestMeasurement();
      } else if (event.code === "KeyH" || event.code === "KeyV") {
        event.preventDefault();
        setGuideAxis(event.code === "KeyV" ? "x" : "y");
      } else if (event.code === "Digit1" || event.code === "Digit2") {
        event.preventDefault();
        if (event.repeat) return;
        const axis = event.code === "Digit1" ? "x" : "y";
        if (startProbe(axis)) {
          setNativeCursorRangeActive(true);
          heldProbeCodeRef.current = event.code;
          setProbeAxis(axis);
        }
      }
    };
    const onKeyUp = (event: KeyboardEvent) => {
      if (event.code === "KeyH") {
        setGuideAxis((current) => (current === "y" ? undefined : current));
      } else if (event.code === "KeyV") {
        setGuideAxis((current) => (current === "x" ? undefined : current));
      } else if (event.code === "KeyR") {
        setRadiusActive(false);
      } else if (
        (event.code === "Digit1" || event.code === "Digit2") &&
        heldProbeCodeRef.current === event.code
      ) {
        heldProbeCodeRef.current = undefined;
        setProbeAxis(undefined);
        setNativeCursorRangeActive(false);
        finishProbe();
      }
    };
    window.addEventListener("blur", clearHeldTools);
    window.addEventListener("keydown", onKeyDown, true);
    window.addEventListener("keyup", onKeyUp, true);
    return () => {
      window.removeEventListener("blur", clearHeldTools);
      window.removeEventListener("keydown", onKeyDown, true);
      window.removeEventListener("keyup", onKeyUp, true);
    };
  }, [
    cancelProbe,
    close,
    copyColor,
    copyLatestMeasurement,
    cycleTolerance,
    deleteHovered,
    deleteLatestMeasurement,
    finishProbe,
    redo,
    setNativeCursorRangeActive,
    startProbe,
    toggleCenterlines,
    toggleCrosshair,
    toggleDetectedBoxes,
    undo,
  ]);

  return { guideAxis, probeAxis, radiusActive };
}
