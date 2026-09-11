// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useMemo, useRef } from "react";

import { PreviewFit, PreviewFitContext } from "./preview-fit-context";

/** Holds the editor's one preview fit, for the tools above it to call. */
export function PreviewFitProvider({ children }: { children: ReactNode }) {
  const fitRef = useRef<((width?: number) => void) | null>(null);
  const basisRef = useRef<((width?: number) => void) | null>(null);
  const value = useMemo<PreviewFit>(
    () => ({
      fitPreview: (width) => {
        fitRef.current?.(width);
      },
      registerFitPreview: (fit, setBasis) => {
        fitRef.current = fit;
        basisRef.current = setBasis;
        return () => {
          if (fitRef.current === fit) fitRef.current = null;
          if (basisRef.current === setBasis) basisRef.current = null;
        };
      },
      setFitBasis: (width) => {
        basisRef.current?.(width);
      },
    }),
    [],
  );

  return <PreviewFitContext value={value}>{children}</PreviewFitContext>;
}
