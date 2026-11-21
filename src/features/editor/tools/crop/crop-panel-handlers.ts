// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useLayoutEffect, useRef } from "react";
import { create } from "zustand";

import { ToolPanelHandlers } from "../tool-panel-handlers";

import type { EditorCropTarget } from "./crop-target";

// The crop is the chosen layer's, taken up and put away by the editor's
// preview, which owns what is in hand; the islands ask for it from their
// own window. The preview publishes how it takes the crop up while it is
// mounted, and the islands' asks reach it through here.

const useCropToolControl = create<{
  onCropChange: ((isCropping: boolean) => void) | null;
}>(() => ({ onCropChange: null }));

/** Publishes how the preview takes the crop up and puts it away while it is
 * mounted. */
export function usePublishCropTool(
  onCropChange: (isCropping: boolean) => void,
) {
  const changeRef = useRef(onCropChange);
  useLayoutEffect(() => {
    changeRef.current = onCropChange;
  });
  useEffect(() => {
    useCropToolControl.setState({
      onCropChange: (isCropping) => {
        changeRef.current(isCropping);
      },
    });
    return () => {
      useCropToolControl.setState({ onCropChange: null });
    };
  }, []);
}

/** The crop island's asks, and the layer islands' crop, answered against the
 * workspace's own source and the preview's tools. */
export const cropPanelHandlers = (
  target: EditorCropTarget | null,
): Pick<
  ToolPanelHandlers,
  "onCropAspectChange" | "onCropReset" | "onCropToolChange"
> => ({
  onCropAspectChange: (aspect) => {
    target?.chooseAspect(aspect);
  },
  onCropReset: () => {
    target?.reset();
  },
  onCropToolChange: (isCropping) => {
    useCropToolControl.getState().onCropChange?.(isCropping);
  },
});
