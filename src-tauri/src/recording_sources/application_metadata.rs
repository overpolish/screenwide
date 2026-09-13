// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The directory the application pickers cache extracted app icons in. Glide
/// shares it, so an icon one surface has already paid for is instant for the
/// next, and the asset protocol only has this one path to allow.
pub(crate) fn application_icon_cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
  let cache_dir = app
    .path()
    .temp_dir()
    .map_err(|error| error.to_string())?
    .join("Screenwide")
    .join("application-sources");
  std::fs::create_dir_all(&cache_dir).map_err(|error| error.to_string())?;
  Ok(cache_dir)
}

pub(super) fn application_details(
  applications: HashMap<String, (String, Option<PathBuf>, HashSet<u32>)>,
) -> Result<Vec<ApplicationDetails>, String> {
  let mut result = applications
    .into_iter()
    .map(|(id, (label, icon_path, process_ids))| {
      let mut process_ids = process_ids.into_iter().collect::<Vec<_>>();
      process_ids.sort_unstable();
      ApplicationDetails {
        id,
        label,
        icon_path,
        process_ids,
      }
    })
    .collect::<Vec<_>>();
  result.sort_by_cached_key(|application| application.label.to_lowercase());
  Ok(result)
}
