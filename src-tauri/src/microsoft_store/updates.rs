// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether the Microsoft Store has a newer version of the package, and which.
//! `StoreContext` answers the first but not the second: the update it lists
//! names the installed package, the one that has an update waiting. The
//! version comes from the Store's public catalog, the product record its own
//! app reads. That record is undocumented, so without it the update is still
//! offered, unnamed.

use std::time::Duration;

use serde::Deserialize;
use windows::ApplicationModel::Package;
use windows::Services::Store::StoreContext;

/// Screenwide's page in the Store app, by the product ID
/// `.github/workflows/store.yml` publishes to.
pub const STORE_PAGE_URL: &str = "ms-windows-store://pdp/?productid=9MSN8CRH5W0F";

/// The product's record in every market at once, with the packages each SKU
/// installs.
const CATALOG_URL: &str =
  "https://displaycatalog.mp.microsoft.com/v7.0/products/9MSN8CRH5W0F?market=neutral&languages=neutral";

/// An update the Store has for the package.
#[derive(serde::Serialize)]
pub struct StoreUpdate {
  /// The new version as the app numbers itself (`1.0.3`), when the catalog
  /// said which.
  version: Option<String>,
}

/// The update the Store has for the package, if any. Only a copy installed
/// from the Store has one: a package registered from a folder finds none.
pub async fn find() -> Result<Option<StoreUpdate>, String> {
  let waiting = tauri::async_runtime::spawn_blocking(waiting_update)
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| format!("Could not ask the Microsoft Store for updates: {error}"))?;
  let Some(installed) = waiting else {
    return Ok(None);
  };
  let version = match published_version(&installed).await {
    Ok(version) => version,
    Err(error) => {
      eprintln!("Could not read the version the Microsoft Store offers: {error}");
      None
    }
  };
  Ok(Some(StoreUpdate { version }))
}

/// The installed package, when the Store has an update for it.
struct Installed {
  family: String,
  version: u64,
}

/// Blocks while the Store answers, which takes seconds.
fn waiting_update() -> windows::core::Result<Option<Installed>> {
  let updates = StoreContext::GetDefault()?
    .GetAppAndOptionalStorePackageUpdatesAsync()?
    .join()?;
  if updates.Size()? == 0 {
    return Ok(None);
  }
  let id = Package::Current()?.Id()?;
  let version = id.Version()?;
  Ok(Some(Installed {
    family: id.FamilyName()?.to_string(),
    version: pack([
      version.Major,
      version.Minor,
      version.Build,
      version.Revision,
    ]),
  }))
}

async fn published_version(installed: &Installed) -> Result<Option<String>, String> {
  let catalog = client()?
    .get(CATALOG_URL)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .map_err(|error| error.to_string())?
    .bytes()
    .await
    .map_err(|error| error.to_string())?;
  newest_version(&catalog, installed)
}

fn client() -> Result<reqwest::Client, String> {
  // reqwest is built without a TLS provider of its own; the updater and the
  // model downloads install the same one, whichever runs first.
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }
  reqwest::Client::builder()
    .user_agent(concat!("Screenwide/", env!("CARGO_PKG_VERSION")))
    .connect_timeout(Duration::from_secs(15))
    .timeout(Duration::from_secs(30))
    .build()
    .map_err(|error| error.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Catalog {
  product: Product,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Product {
  #[serde(default)]
  display_sku_availabilities: Vec<Availability>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Availability {
  sku: Sku,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Sku {
  properties: SkuProperties,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SkuProperties {
  packages: Option<Vec<CatalogPackage>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CatalogPackage {
  package_family_name: String,
  /// The four parts packed into one number, 16 bits each, as text.
  version: String,
}

/// A package version's four parts packed as the catalog writes them.
fn pack([major, minor, build, revision]: [u16; 4]) -> u64 {
  (u64::from(major) << 48)
    | (u64::from(minor) << 32)
    | (u64::from(build) << 16)
    | u64::from(revision)
}

/// The newest version the catalog lists for the installed package's family,
/// without the fourth part the Store's packages carry (`1.0.3.0` is
/// `1.0.3`). None when the catalog lists nothing newer than what is
/// installed, as it may for a while after a release while it catches up
/// with the Store.
fn newest_version(catalog: &[u8], installed: &Installed) -> Result<Option<String>, String> {
  let catalog: Catalog = serde_json::from_slice(catalog).map_err(|error| error.to_string())?;
  Ok(
    catalog
      .product
      .display_sku_availabilities
      .iter()
      .filter_map(|availability| availability.sku.properties.packages.as_ref())
      .flatten()
      .filter(|package| package.package_family_name == installed.family)
      .filter_map(|package| package.version.parse::<u64>().ok())
      .max()
      .filter(|&newest| newest > installed.version)
      .map(|newest| {
        format!(
          "{}.{}.{}",
          newest >> 48,
          (newest >> 32) & 0xFFFF,
          (newest >> 16) & 0xFFFF
        )
      }),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  const FAMILY: &str = "overpolish.Screenwide_y93s1f9758jx8";

  fn installed(version: [u16; 4]) -> Installed {
    Installed {
      family: FAMILY.to_owned(),
      version: pack(version),
    }
  }

  fn catalog(packages: &[(&str, [u16; 4])]) -> Vec<u8> {
    let packages: Vec<_> = packages
      .iter()
      .map(|(family, version)| {
        serde_json::json!({ "PackageFamilyName": family, "Version": pack(*version).to_string() })
      })
      .collect();
    serde_json::json!({ "Product": { "DisplaySkuAvailabilities": [
      { "Sku": { "Properties": { "Packages": packages } } },
      { "Sku": { "Properties": { "Packages": null } } },
    ] } })
    .to_string()
    .into_bytes()
  }

  #[test]
  fn reads_the_newest_version_of_the_family_without_its_fourth_part() {
    let catalog = catalog(&[
      (FAMILY, [1, 0, 2, 0]),
      (FAMILY, [1, 0, 3, 0]),
      ("someone.Else_abc", [9, 0, 0, 0]),
    ]);
    assert_eq!(
      newest_version(&catalog, &installed([1, 0, 1, 0])),
      Ok(Some("1.0.3".to_owned()))
    );
  }

  #[test]
  fn names_no_version_until_the_catalog_lists_a_newer_one() {
    let catalog = catalog(&[(FAMILY, [1, 0, 2, 0])]);
    assert_eq!(newest_version(&catalog, &installed([1, 0, 2, 0])), Ok(None));
    assert_eq!(newest_version(&catalog, &installed([1, 0, 3, 0])), Ok(None));
  }

  #[test]
  fn names_no_version_when_the_family_is_not_listed() {
    let catalog = catalog(&[("someone.Else_abc", [2, 0, 0, 0])]);
    assert_eq!(newest_version(&catalog, &installed([1, 0, 0, 0])), Ok(None));
  }

  #[test]
  fn reads_a_version_with_large_parts() {
    let catalog = catalog(&[(FAMILY, [2, 65535, 300, 0])]);
    assert_eq!(
      newest_version(&catalog, &installed([1, 0, 0, 0])),
      Ok(Some("2.65535.300".to_owned()))
    );
  }
}
