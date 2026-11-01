// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// How the parts and the minimap's ticks glide to new places as the strip
// makes room or is reshaped, or as what was lifted settles: while the cue
// drawn before them says so. Being before them, the cue is turned on before
// they move, even where the browser takes their new places mid-change, as a
// focused part removed makes it; a class on an element round them would
// only be turned on after.

/** What glides while the cue before it says so. */
export const GLIDE =
  "peer-data-glide:[&_*]:transition-[left,right,width,transform,translate] peer-data-glide:[&_*]:duration-(--glide) peer-data-glide:[&_*]:ease-out";

/** Drawn before what glides, as its sibling. */
export const GlideCue = ({ isGliding }: { isGliding: boolean }) => (
  <span className="peer hidden" data-glide={isGliding || undefined} />
);
