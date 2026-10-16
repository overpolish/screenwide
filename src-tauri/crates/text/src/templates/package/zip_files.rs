// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A package's files read from a zip and written to one. A zip may hold its
//! manifest at the top or in one folder there, and whatever the archiver
//! added beside it; a name that would climb out of the package is turned
//! away.

use std::io::{Read, Write};
use std::path::{Component, Path};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use super::super::MANIFEST_FILE;

/// The most entries a zip may list, whatever is kept of them: enough for
/// any package and what an archiver adds, short of a file that would take
/// long to look through.
const MAX_ENTRIES: usize = 1_024;

/// A zip entry's name as the folders and file it is, or none for one that
/// climbs out of the package or is not a plain path.
fn parts(name: &Path) -> Option<Vec<String>> {
  name
    .components()
    .map(|component| match component {
      Component::Normal(part) => part.to_str().map(str::to_owned),
      _ => None,
    })
    .collect()
}

/// Each file of the package zipped at `path`, as `add` takes them: its
/// place in the package, how large it says it is, and how to read it.
pub(super) fn read(
  path: &Path,
  mut add: impl FnMut(String, u64, &mut dyn FnMut() -> std::io::Result<Vec<u8>>) -> Result<(), String>,
) -> Result<(), String> {
  let file = std::fs::File::open(path).map_err(|error| format!("it cannot be opened: {error}"))?;
  let mut archive =
    ZipArchive::new(file).map_err(|error| format!("it is not a zip that can be read: {error}"))?;
  if archive.len() > MAX_ENTRIES {
    return Err(format!("it lists more than {MAX_ENTRIES} entries"));
  }
  let mut names = Vec::with_capacity(archive.len());
  for index in 0..archive.len() {
    let entry = archive
      .by_index_raw(index)
      .map_err(|error| error.to_string())?;
    let name = entry
      .enclosed_name()
      .and_then(|name| parts(&name))
      .ok_or_else(|| format!("{} would be written outside the template", entry.name()))?;
    names.push((name, entry.is_file()));
  }
  // The manifest at the top, or in the one folder that holds it there.
  let manifests: Vec<&Vec<String>> = names
    .iter()
    .filter(|(name, is_file)| {
      *is_file
        && name.last().is_some_and(|last| last == MANIFEST_FILE)
        && name.len() <= 2
        && name.first().is_some_and(|first| first != "__MACOSX")
    })
    .map(|(name, _)| name)
    .collect();
  let prefix = match manifests.as_slice() {
    [only] => only[..only.len() - 1].to_vec(),
    [] => return Err(format!("it has no {MANIFEST_FILE}")),
    _ => return Err(format!("it holds more than one {MANIFEST_FILE}")),
  };
  for (index, (name, is_file)) in names.iter().enumerate() {
    let Some(within) = name.strip_prefix(prefix.as_slice()).filter(|_| *is_file) else {
      continue;
    };
    let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
    let size = entry.size();
    add(within.join("/"), size, &mut || {
      let mut bytes = Vec::new();
      // Read no more than the entry says, and a byte past it, so one that
      // holds more than it claims shows itself without being read whole.
      (&mut entry).take(size + 1).read_to_end(&mut bytes)?;
      Ok(bytes)
    })?;
  }
  Ok(())
}

/// Writes `files` zipped at `destination`, in a folder named `folder`.
pub(super) fn write(
  destination: &Path,
  folder: &str,
  files: &[(String, Vec<u8>)],
) -> Result<(), String> {
  let written = (|| -> zip::result::ZipResult<()> {
    let mut zip = ZipWriter::new(std::fs::File::create(destination)?);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.add_directory(format!("{folder}/"), options)?;
    for (path, bytes) in files {
      zip.start_file(format!("{folder}/{path}"), options)?;
      zip.write_all(bytes)?;
    }
    zip.finish()?;
    Ok(())
  })();
  written.map_err(|error| format!("it cannot be written to {}: {error}", destination.display()))
}
