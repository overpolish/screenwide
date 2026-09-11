// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useMemo, useRef } from "react";

import { PreviewFit, PreviewFitContext } from "./preview-fit-context";

/** Holds the editor's one preview fit, for the tools above it to call. */
export function PreviewFitProvider({ children }: { children: ReactNode }) {
  const fitRef = useRef<((width?: number) => void) | null>(null);
  const value = useMemo<PreviewFit>(
    () => ({
      fitPreview: (width) => {
        fitRef.current?.(width);
      },
      registerFitPreview: (fit) => {
        fitRef.current = fit;
        return () => {
          if (fitRef.current === fit) fitRef.current = null;
        };
      },
    }),
    [],
  );

  return <PreviewFitContext value={value}>{children}</PreviewFitContext>;
}
