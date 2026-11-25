// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { clearExportFailure } from "../../export/export-island-store";
import { proportionKey } from "../../recording/formats/format-scope";
import {
  activeFormat,
  formatInHand,
  formatsOfShape,
  isMainCanvas,
  mainDocument,
  withPhone,
  withPreview,
} from "../../recording/formats/recording-formats";
import {
  appOf,
  usePlatformOverlay,
} from "../../recording/formats/use-platform-overlay";
import {
  frameSizePreset,
  SCREENSHOT_SIZE_PRESETS,
} from "../../tools/frame/frame-sizes";

import { screenshotSynced } from "./screenshot-annotation-custom";
import { shapeSize, sizeScaleOf } from "./screenshot-format-resolve";
import {
  removedScreenshotPlatform,
  withScreenshotMainBackground,
  withScreenshotMainPlacement,
  withScreenshotPlatformShared,
  withScreenshotShapeFollowed,
} from "./screenshot-format-sharing";
import {
  addedScreenshotFormat,
  settledFormats,
  switchedScreenshotFormat,
} from "./screenshot-formats";
import {
  isLeavingOut,
  leftOutInHand,
  screenshotFormatSync,
} from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { FrameFormatsControl } from "../../recording/formats/use-recording-formats";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

type Workspace = ScreenshotWorkspaceOutputSettings;

/**
 * The screenshot's formats as the format bar, the Screen island and the
 * right-click menus show and change them, as a recording's are
 * (`useRecordingFormats`): a platform added, put in hand, taken away,
 * shared with others of its canvas, or given the main canvas's layout
 * back, or a layer or annotation shown in only some formats, swaps the
 * workspace the window shows and its formats as one edit, and the app and
 * phone of the platform in hand are laid over the picture. Null outside a
 * screenshot.
 */
export function useScreenshotFormats({
  blank,
  formats,
  isScreenshot,
  onFormatsChange,
  onWorkspaceChange,
  workspace,
}: {
  /** The layers drawn on rather than pictures, by id. */
  blank: ReadonlySet<number>;
  formats: ScreenshotFormats | null;
  isScreenshot: boolean;
  onFormatsChange: (formats: ScreenshotFormats | null) => void;
  onWorkspaceChange: (workspace: Workspace) => void;
  workspace: Workspace;
}): FrameFormatsControl | null {
  const shape = formats ? activeFormat(formats) : undefined;
  const inHandId = shape ? formatInHand(shape) : null;
  const preset = inHandId && inHandId !== shape?.id ? inHandId : null;
  const { height, width } = workspace;
  const isTall = height > width;
  // As on a recording, the phone and the app show only while the preview
  // is on, and only for a platform watched in a phone app.
  const isPreviewed = isScreenshot && (formats?.preview ?? false);
  const overlay = isPreviewed ? appOf(preset) : null;
  const phone = overlay && isTall ? (formats?.phone ?? null) : null;
  usePlatformOverlay({
    height,
    isEnabled: isScreenshot,
    overlay,
    phone: phone?.phone ?? null,
    width,
  });
  // Kept while the formats are, so the preview's drawn workspace is too.
  const leftOut = useMemo(() => leftOutInHand(formats), [formats]);
  const sizeScale = useMemo(
    () =>
      formats && !isMainCanvas(formats)
        ? sizeScaleOf(formats, {
            blank,
            main: mainDocument(formats, workspace),
            shape: activeFormat(formats),
          })
        : 1,
    [blank, formats, workspace],
  );
  // A layer moved, or the background changed, with a platform in hand sets
  // the platform its own way there at once, though that is written down
  // only as it is put down (`settledFormats`).
  const settled = useMemo(
    () => (formats ? settledFormats(formats, { blank, workspace }) : null),
    [blank, formats, workspace],
  );
  const ownOf = (id: string) => settled?.list.find((shape) => shape.id === id);
  if (!isScreenshot) return null;
  const apply = (
    next: { document: Workspace; formats: ScreenshotFormats | null } | null,
  ) => {
    if (!next) return;
    onWorkspaceChange(next.document);
    onFormatsChange(next.formats);
    clearExportFailure();
  };
  const inHand = { blank, workspace };
  return {
    onPatch: (patch) => {
      switch (patch.kind) {
        case "preview":
          onFormatsChange(withPreview(formats, patch.isShown));
          return;
        case "add": {
          if (!frameSizePreset(patch.preset)) return;
          // One format of each platform: asked for again, it is put in hand.
          const made = formats?.list.some((each) =>
            formatsOfShape(each).includes(patch.preset),
          );
          apply(
            made && formats
              ? switchedScreenshotFormat(formats, inHand, patch.preset)
              : addedScreenshotFormat(formats, inHand, patch.preset),
          );
          return;
        }
        case "switch":
          if (formats)
            apply(switchedScreenshotFormat(formats, inHand, patch.id));
          return;
        case "remove":
          if (formats)
            apply(removedScreenshotPlatform(formats, inHand, patch.id));
          return;
        case "phone":
          onFormatsChange(withPhone(formats, patch.phone));
          return;
        case "share":
          if (formats)
            apply(withScreenshotPlatformShared(formats, inHand, patch));
          return;
        case "follow":
          if (formats)
            apply(withScreenshotShapeFollowed(formats, inHand, patch.shape));
          return;
        case "main-placement":
          if (formats) apply(withScreenshotMainPlacement(formats, inHand));
          return;
        case "main-background":
          if (formats) apply(withScreenshotMainBackground(formats, inHand));
          return;
        case "sync":
          if (formats) apply(screenshotSynced(formats, inHand, patch));
          return;
        default:
          return;
      }
    },
    snapshot: {
      activeId: formats?.active ?? null,
      formatId: inHandId,
      height,
      leftOut,
      list: (formats?.list ?? []).map((each) => {
        const size = formats ? shapeSize(formats, each, workspace) : workspace;
        return {
          formats: formatsOfShape(each).map((id) => ({
            id,
            leavesOut: formats ? isLeavingOut(formats, id) : false,
            preset: id === each.id ? null : id,
          })),
          height: size.height,
          id: each.id,
          ownBackground: ownOf(each.id)?.ownBackground === true,
          ownWay: ownOf(each.id)?.placed === true,
          placed: ownOf(each.id)?.placed === true,
          width: size.width,
        };
      }),
      overlay: appOf(preset),
      phone: isTall ? (formats?.phone ?? null) : null,
      preset,
      preview: isPreviewed,
      scope: isMainCanvas(formats) ? "all" : "shape",
      sizeScale,
      syncOf: (id) =>
        // Settled, so an annotation just changed shows as custom at once.
        screenshotFormatSync(settled, {
          key: isMainCanvas(formats) ? null : proportionKey(workspace),
          syncId: id,
        }),
      templates: SCREENSHOT_SIZE_PRESETS,
      width,
    },
  };
}
