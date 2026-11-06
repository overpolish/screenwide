// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { cn } from "../lib/styling";

type SiteClipProps = {
  /** What the clip shows, for assistive technology. */
  label: string;
  /** The clip and its poster in `public/recordings`, without an extension. */
  name: string;
  className?: string;
};

/**
 * A short recording of the real app, playing muted on a loop while it is in
 * view. Reduced motion leaves it on its first frame.
 */
export function SiteClip({ className, label, name }: SiteClipProps) {
  const ref = useRef<HTMLVideoElement>(null);
  useEffect(() => {
    const video = ref.current;
    if (!video) return;
    const still = matchMedia("(prefers-reduced-motion: reduce)");
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !still.matches) {
          // A play cut short by scrolling away rejects; the next view retries.
          video.play().catch(() => undefined);
        } else {
          video.pause();
        }
      },
      { threshold: 0.3 },
    );
    observer.observe(video);
    return () => {
      observer.disconnect();
    };
  }, []);
  return (
    <video
      aria-label={label}
      className={cn("block h-auto w-full", className)}
      loop
      muted
      playsInline
      poster={`/recordings/${name}.jpg`}
      preload="metadata"
      ref={ref}
      src={`/recordings/${name}.mp4`}
    />
  );
}
