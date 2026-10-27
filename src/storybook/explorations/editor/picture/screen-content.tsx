// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";
import { MockApp } from "../../ideas/mock-app";
import { type Recording } from "../recording";

import { GameFrame } from "./game-frame";
import { RecordedThumbnail, RecordedVideo } from "./recorded-media";

/** What was recorded on screen at a moment: a real recording's own picture,
 * or, drawn, the app or a game. A thumbnail of a real recording comes from
 * its sheet of frames rather than its movie. */
export function ScreenContent({
  children,
  className,
  isThumbnail = false,
  recording,
  seconds,
}: {
  recording: Recording;
  seconds: number;
  children?: ReactNode;
  className?: string;
  isThumbnail?: boolean;
}) {
  const { media } = recording;
  if (media)
    return (
      <div className={cn("relative aspect-[16/10] overflow-hidden", className)}>
        {isThumbnail ? (
          <RecordedThumbnail
            duration={recording.duration}
            media={media}
            seconds={seconds}
          />
        ) : (
          <RecordedVideo media={media} seconds={seconds} />
        )}
        {children}
      </div>
    );
  return recording.look === "game" ? (
    <GameFrame className={className} recording={recording} seconds={seconds}>
      {children}
    </GameFrame>
  ) : (
    <MockApp className={className}>{children}</MockApp>
  );
}
