// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../lib/styling";

/** The glided app's icon. Decorative: the app is not named to assistive
 * technology, so it has no alt text. */
export function GlideAppIcon({
  className,
  src,
}: {
  className: string;
  src: string;
}) {
  return (
    <>
      {/* Some Windows executables expose unusually low-alpha icon artwork. A
          second identical layer, shown only there, restores its visual weight
          without changing opaque icons or baking in an app-specific
          backdrop. */}
      <img
        alt=""
        className={cn("glide-app-icon glide-app-icon-windows-boost", className)}
        src={src}
      />
      <img alt="" className={cn("glide-app-icon", className)} src={src} />
    </>
  );
}
