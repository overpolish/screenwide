// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

type FeatureStoryStageProps = {
  children: ReactNode;
  height: number;
  viewMode: string;
  width: number;
};

/**
 * Preserves a feature window's production dimensions. The Storybook canvas
 * scrolls around the window when space is tight instead of flex-shrinking it.
 */
export function FeatureStoryStage({
  children,
  height,
  viewMode,
  width,
}: FeatureStoryStageProps) {
  const feature = (
    <div className="shrink-0" style={{ height, width }}>
      {children}
    </div>
  );

  if (viewMode === "docs") {
    return <div className="max-w-full overflow-auto p-6">{feature}</div>;
  }

  return (
    <div className="fixed inset-0 overflow-auto">
      <div className="box-border flex h-max min-h-full w-max min-w-full items-center justify-center p-6">
        {feature}
      </div>
    </div>
  );
}
