// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { clearExportFailure } from "../../export/export-island-store";
import { frameSizePreset } from "../../tools/frame/frame-sizes";

import { withMainBackground } from "./format-look-carry";
import {
  isFramedOwnWay,
  removedPlatform,
  withMainPlacement,
  withPlatformShared,
  withShapeFollowed,
} from "./format-sharing";
import { madeForOf, withMainMadeFor } from "./main-made-for";
import {
  activeFormat,
  addedFormat,
  formatInHand,
  formatsOfShape,
  isMainCanvas,
  switchedFormat,
  withPhone,
  withPreview,
} from "./recording-formats";
import { appOf, usePlatformOverlay } from "./use-platform-overlay";

import type { FormatDocument } from "./format-document";
import type { EditScope } from "./format-scope";
import type { FormatSyncAction } from "./format-sync";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { PhoneView } from "../../../../bindings/PhoneView";
import type { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import type { ToolPanelFormats } from "../../tools/tool-panel-frame";
import type { CameraOverlaySettings } from "../../types";

/** What the format bar asks of the recording's formats. */
export type FrameFormatPatch =
  /** The phone and the app's interface shown over the picture, or not. */
  | { isShown: boolean; kind: "preview" }
  /** A platform added, into the shape of its proportions, and put in hand. */
  | { kind: "add"; preset: string }
  /** The format `id` put in hand. */
  | { id: string; kind: "switch" }
  /** The format `id` taken away. */
  | { id: string; kind: "remove" }
  /** The phone every tall format is previewed on, or none. */
  | { kind: "phone"; phone: PhoneView | null }
  /** A scene, annotation, song or sound effect, or the caption track,
   * brought in step across the formats (`format-sync.ts`). */
  | { action: FormatSyncAction; id: string; kind: "sync" }
  /** The platform `platform` made to share the shape in hand's framing,
   * or taken out of it to follow the main canvas (`format-sharing.ts`). */
  | { isShared: boolean; kind: "share"; platform: string }
  /** The shape in hand made to share the framing of the shape `shape`. */
  | { kind: "follow"; shape: string }
  /** The shape in hand's screen laid out from the main canvas again. */
  | { kind: "main-placement" }
  /** The shape in hand's background put back to the main canvas's, which
   * it follows again. */
  | { kind: "main-background" }
  /** The main canvas made for the template `preset`, or for none
   * (`main-made-for.ts`). */
  | { kind: "made-for"; preset: string | null };

export type FrameFormatsControl = {
  onPatch: (patch: FrameFormatPatch) => void;
  snapshot: ToolPanelFormats;
};

/**
 * The recording's formats as the format bar shows and changes them, the
 * app and phone of the format in hand laid over the editor's picture, and
 * where a layout changed in the shape in hand lands: on the main canvas,
 * the shared layout every platform follows; in a platform's shape, that
 * shape's own. Changing shape swaps the canvas and the
 * camera as one edit; the timeline is worked out for the format in hand
 * from the one shared edit (`use-format-edit.ts`).
 */
export function useRecordingFormats({
  cameraOverlay,
  formats,
  isRecording,
  onCameraOverlayChange,
  onFormatsChange,
  onRecordingOutputChange,
  recordingOutput,
}: {
  cameraOverlay: CameraOverlaySettings;
  formats: FrameFormats | null;
  isRecording: boolean;
  onCameraOverlayChange: (overlay: CameraOverlaySettings) => void;
  onFormatsChange: (formats: FrameFormats | null) => void;
  onRecordingOutputChange: (output: RecordingOutputSettings) => void;
  recordingOutput: RecordingOutputSettings;
}): FrameFormatsControl | null {
  const shape = formats ? activeFormat(formats) : undefined;
  const inHandId = shape ? formatInHand(shape) : null;
  const { height, width } = recordingOutput.primary;
  const isMain = isMainCanvas(formats);
  const madeFor = isMain ? madeForOf(formats, { height, width }) : null;
  // A shape made for no platform is named by its id, not a template; the
  // main canvas by the template it is made for, where it is.
  const preset =
    inHandId && inHandId !== shape?.id ? inHandId : (madeFor ?? null);
  const isStale = isMain && !madeFor && !!formats?.list[0]?.madeFor;
  const isTall = height > width;
  // The phone and the app are only laid over the picture while the preview
  // is on, and only for a platform watched in a phone app (Shorts, TikTok,
  // Reels): LinkedIn, X and the main canvas have no app to preview, so no
  // phone either. The phone is the project's, whichever format is in hand.
  const isPreviewed = isRecording && (formats?.preview ?? false);
  const overlay = isPreviewed ? appOf(preset) : null;
  const phone = overlay && isTall ? (formats?.phone ?? null) : null;
  const phoneId = phone?.phone ?? null;
  usePlatformOverlay({
    height,
    isEnabled: isRecording,
    overlay,
    phone: phoneId,
    width,
  });
  const document: FormatDocument = { cameraOverlay, recordingOutput };
  // A size typed in, or the frame dragged, leaves the template the main
  // canvas was made for: it is made for none, and keeps clear of nothing.
  useEffect(() => {
    if (!isStale || !isRecording) return;
    const next = withMainMadeFor(formats, document, null);
    if (!next) return;
    onRecordingOutputChange(next.document.recordingOutput);
    onFormatsChange(next.formats);
  });
  if (!isRecording) return null;
  // Edits on the main canvas are what every platform follows; a
  // platform's own stay its shape's.
  const scope: EditScope = isMain ? "all" : "shape";
  const apply = (
    next: { document: FormatDocument; formats: FrameFormats | null } | null,
  ) => {
    if (!next) return;
    onRecordingOutputChange(next.document.recordingOutput);
    onCameraOverlayChange(next.document.cameraOverlay);
    onFormatsChange(next.formats);
    clearExportFailure();
  };
  return {
    onPatch: (patch) => {
      switch (patch.kind) {
        case "preview":
          onFormatsChange(withPreview(formats, patch.isShown));
          return;
        case "add": {
          const template = frameSizePreset(patch.preset);
          if (!template) return;
          // One format of each platform: asked for again, it is put in hand,
          // the main canvas where that is made for it.
          const made = formats?.list.some(
            (each, index) =>
              formatsOfShape(each).includes(template.id) ||
              (index === 0 && each.madeFor === template.id),
          );
          if (made) {
            if (formats)
              apply(
                switchedFormat(
                  formats,
                  document,
                  formats.list[0]?.madeFor === template.id
                    ? formats.list[0].id
                    : template.id,
                ),
              );
            return;
          }
          apply(
            addedFormat(formats, document, {
              preset: template.id,
              size: template,
            }),
          );
          return;
        }
        case "switch":
          if (formats) apply(switchedFormat(formats, document, patch.id));
          return;
        case "remove":
          if (formats) apply(removedPlatform(formats, document, patch.id));
          return;
        case "phone":
          onFormatsChange(withPhone(formats, patch.phone));
          return;
        case "share":
          if (formats) apply(withPlatformShared(formats, document, patch));
          return;
        case "follow":
          if (formats) apply(withShapeFollowed(formats, document, patch.shape));
          return;
        case "main-placement":
          if (formats) apply(withMainPlacement(formats, document));
          return;
        case "main-background":
          if (formats) apply(withMainBackground(formats, document));
          return;
        case "made-for":
          apply(withMainMadeFor(formats, document, patch.preset));
          return;
        case "sync":
          // Handled with the timeline, which knows the format in hand.
          return;
      }
    },
    snapshot: {
      activeId: formats?.active ?? null,
      formatId: inHandId,
      height,
      list: (formats?.list ?? []).map((each, index) => {
        const size =
          each.id === formats?.active
            ? { height, width }
            : (each.look?.recordingOutput.primary ?? { height, width });
        // The main canvas is named for the template it is made for.
        const own = index === 0 ? madeForOf(formats, size) : null;
        return {
          formats: formatsOfShape(each).map((id) => ({
            id,
            preset: id === each.id ? own : id,
          })),
          height: size.height,
          id: each.id,
          ownBackground: each.ownBackground === true,
          ownWay: isFramedOwnWay(each, formats?.edits),
          placed: each.placed === true,
          width: size.width,
        };
      }),
      overlay: appOf(preset),
      phone: isTall ? (formats?.phone ?? null) : null,
      preset,
      preview: isPreviewed,
      scope,
      width,
    },
  };
}
