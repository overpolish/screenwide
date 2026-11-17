// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// How the folder CoreML keeps a model compiled in, beside the model's first
/// file, starts: that file's name. The helper adds what tells one compile
/// from another, and the app removes every folder starting so with the file.
pub fn compiled_model_prefix(first_file_name: &str) -> String {
  format!("{first_file_name}.coreml-")
}
