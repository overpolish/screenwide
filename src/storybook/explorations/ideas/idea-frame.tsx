// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { Text } from "../../../primitives/text/text";

// Frames one idea: what it is in a line, how to try it, and the idea itself
// on the editor's recess, the way the work sits in the editor.
export function IdeaFrame({
  children,
  hint,
  pitch,
  title,
}: {
  children: ReactNode;
  hint: ReactNode;
  pitch: string;
  title: string;
}) {
  return (
    <div className="flex size-full flex-col gap-separate p-group text-fg">
      <header className="flex flex-col gap-inline">
        <Text as="h1" variant="title">
          {title}
        </Text>
        <Text className="max-w-[72ch] text-fg-secondary">{pitch}</Text>
        <Text variant="caption">{hint}</Text>
      </header>
      <div className="relative flex min-h-0 grow flex-col items-center justify-center gap-group rounded-panel bg-recess p-group">
        {children}
      </div>
    </div>
  );
}
