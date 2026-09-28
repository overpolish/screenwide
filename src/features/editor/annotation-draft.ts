// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { usePublishAnnotationDraft } from "./annotation-channel";
import {
  firstAnnotationDress,
  useAnnotationAngleDefault,
  useAnnotationAnimatedDefault,
  useAnnotationDefaults,
} from "./annotation-defaults";
import { ANNOTATION_KINDS } from "./annotation-kinds";
import { drawingToolKind } from "./tool-panels/tool-registry";
import { EditorKind } from "./types";

/**
 * Publish the dress the drawing tool in hand draws its next annotation in, so
 * the panel can show and change it before anything is drawn: the dress last
 * settled on for that kind, or the tool's own first one. A counter carries the
 * aim its next one is dropped at, and whether the next one draws itself in
 * rides along - except for a kind that always arrives still.
 */
export function useAnnotationDraft(workspace: EditorKind, tool: string | null) {
  const kind = drawingToolKind(tool);
  const remembered = useAnnotationDefaults(kind ?? "arrow");
  const animated = useAnnotationAnimatedDefault();
  const angle = useAnnotationAngleDefault();
  usePublishAnnotationDraft(
    workspace,
    kind === null
      ? null
      : {
          animated: !ANNOTATION_KINDS[kind].startsStill && (animated ?? true),
          id: "",
          isDraft: true,
          kind,
          style: remembered ?? firstAnnotationDress(kind),
          ...(kind === "counter" ? { angle: angle ?? 0 } : {}),
        },
  );
}
