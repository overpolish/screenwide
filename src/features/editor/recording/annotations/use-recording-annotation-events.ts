// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { Annotation, AnnotationTextEdit } from "../../annotations/annotations";

import {
  RecordingAnnotationPinCommit,
  RecordingAnnotationSceneShapesCommit,
} from "./recording-annotations";

/** An edit on the picture, as the native preview reports it once it lands. */
export type RecordingAnnotationEvent = {
  annotations: Annotation[];
  paneIndex: number;
  /** The pins of the pinned annotations among `annotations`. */
  pins: RecordingAnnotationPinCommit[];
  /** The scene places of those an edit inside a scene reached. */
  sceneShapes: RecordingAnnotationSceneShapesCommit[];
  /** Every annotation chosen once the commit lands. */
  selectedAnnotationIds: string[];
  sessionId: number;
  sourcePositionMs: number;
  /** Where in a text box's typing this commit falls; the typing's commits
   * are grouped into one edit. */
  textEdit: AnnotationTextEdit | null;
};
type AnnotationHoverEvent = { annotationId: string | null; sessionId: number };

/** Hears the native preview of session `sessionId`: each edit it makes on
 * the picture, and the annotation under the pointer, handed to the latest
 * `onCommit` and `onHover`. */
export function useRecordingAnnotationEvents(
  sessionId: number | null,
  handlers: {
    onCommit: (event: RecordingAnnotationEvent) => void;
    onHover: (annotationId: string | null) => void;
  },
) {
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;
  useEffect(() => {
    if (sessionId === null) return;
    let disposed = false;
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => {
      if (disposed) stop();
      else stops.push(stop);
    };
    void listen<RecordingAnnotationEvent>(
      "editor://recording-annotations",
      ({ payload }) => {
        if (!disposed && payload.sessionId === sessionId)
          handlersRef.current.onCommit(payload);
      },
    )
      .then(retain)
      .catch((cause: unknown) => {
        console.error("Could not listen for recording annotations", cause);
      });
    void listen<AnnotationHoverEvent>(
      "editor://recording-annotation-hover",
      ({ payload }) => {
        if (!disposed && payload.sessionId === sessionId)
          handlersRef.current.onHover(payload.annotationId);
      },
    )
      .then(retain)
      .catch((cause: unknown) => {
        console.error("Could not listen for annotation hover", cause);
      });
    return () => {
      disposed = true;
      for (const stop of stops) stop();
    };
  }, [sessionId]);
}
