// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

const FIXTURE_SUITES: &[&str] = &[
    "yaml-decomposition",
    "protobuf-conformance",
    "yaml-signature-conformance",
    "verification-runtime",
    "transcoding",
    "schema-alignment",
    "key-id",
    "base64",
    "alg-ed25519",
    "alg-ecdsa",
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    normative: bool,
    fixture_asset_count: usize,
    authority: Authority,
    suites: Vec<Suite>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Authority {
    specification: String,
    fixture_index: String,
    conflict_rule: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    id: String,
    readme: String,
    cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    assets: Vec<String>,
    operations: Vec<String>,
    context: Context,
    expectations: Vec<Expectation>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Context {
    forms: Vec<String>,
    conformance_profiles: Vec<String>,
    outer_conformance: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Expectation {
    #[serde(rename = "type")]
    kind: String,
    #[serde(flatten)]
    details: Map<String, Value>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn manifest_path(root: &Path) -> PathBuf {
    root.join("implementation-kit/conformance-manifest.json")
}

fn load_manifest(root: &Path) -> Manifest {
    let path = manifest_path(root);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&source)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn fixture_assets(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut assets = BTreeSet::new();

    for suite_id in FIXTURE_SUITES {
        let directory = root.join("conformance").join(suite_id);
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;

        for entry in entries {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to read an entry in {}: {error}",
                    directory.display()
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                format!("failed to inspect {}: {error}", entry.path().display())
            })?;
            if !file_type.is_file() || entry.file_name() == "README.md" {
                continue;
            }

            let file_name = entry.file_name().into_string().map_err(|name| {
                format!(
                    "fixture name is not UTF-8 in {}: {}",
                    directory.display(),
                    name.to_string_lossy()
                )
            })?;
            assets.insert(format!("conformance/{suite_id}/{file_name}"));
        }
    }

    Ok(assets)
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn validate_manifest(manifest: &Manifest, root: &Path) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if manifest.schema_version != 1 {
        errors.push(format!(
            "unsupported manifest schema version {}",
            manifest.schema_version
        ));
    }
    if manifest.normative {
        errors.push("the implementation-kit manifest must be non-normative".to_string());
    }
    for (label, path) in [
        ("authority specification", &manifest.authority.specification),
        ("authority fixture index", &manifest.authority.fixture_index),
    ] {
        if path.trim().is_empty() || !root.join(path).is_file() {
            errors.push(format!(
                "{label} does not reference an existing file: {path}"
            ));
        }
    }
    if manifest.authority.conflict_rule.trim().is_empty() {
        errors.push("authority conflict_rule is empty".to_string());
    }

    let expected_suite_ids = FIXTURE_SUITES
        .iter()
        .map(|suite| (*suite).to_string())
        .collect::<BTreeSet<_>>();
    let mut suite_ids = BTreeSet::new();
    let mut referenced_assets = BTreeMap::<String, String>::new();

    for suite in &manifest.suites {
        if suite.id.trim().is_empty() {
            errors.push("suite id is empty".to_string());
        }
        if !suite_ids.insert(suite.id.clone()) {
            errors.push(format!("duplicate suite id: {}", suite.id));
        }

        let expected_readme = format!("conformance/{}/README.md", suite.id);
        if suite.readme != expected_readme || !root.join(&suite.readme).is_file() {
            errors.push(format!(
                "suite {} README must reference existing {expected_readme}",
                suite.id
            ));
        }
        if suite.cases.is_empty() {
            errors.push(format!("suite {} has no cases", suite.id));
        }

        let mut case_ids = BTreeSet::new();
        for case in &suite.cases {
            let scoped_case_id = format!("{}/{}", suite.id, case.id);
            if case.id.trim().is_empty() {
                errors.push(format!("suite {} contains an empty case id", suite.id));
            }
            if !case_ids.insert(case.id.clone()) {
                errors.push(format!("duplicate case id: {scoped_case_id}"));
            }
            if case.assets.is_empty() {
                errors.push(format!("case {scoped_case_id} has no assets"));
            }
            if case.operations.is_empty()
                || case
                    .operations
                    .iter()
                    .any(|operation| operation.trim().is_empty())
            {
                errors.push(format!("case {scoped_case_id} has no valid operations"));
            }
            if case.context.forms.is_empty()
                || case.context.forms.iter().any(|form| form.trim().is_empty())
                || case
                    .context
                    .conformance_profiles
                    .iter()
                    .any(|profile| profile.trim().is_empty())
                || case
                    .context
                    .outer_conformance
                    .iter()
                    .any(|profile| profile.trim().is_empty())
            {
                errors.push(format!("case {scoped_case_id} has invalid context"));
            }
            if case.expectations.is_empty() {
                errors.push(format!("case {scoped_case_id} has no expectations"));
            }
            for expectation in &case.expectations {
                if expectation.kind.trim().is_empty() || expectation.details.is_empty() {
                    errors.push(format!(
                        "case {scoped_case_id} has an empty typed expectation"
                    ));
                }
            }

            let required_prefix = format!("conformance/{}/", suite.id);
            for asset in &case.assets {
                let asset_path = Path::new(asset);
                if !is_safe_relative_path(asset_path) || !asset.starts_with(&required_prefix) {
                    errors.push(format!(
                        "case {scoped_case_id} has an out-of-suite asset path: {asset}"
                    ));
                }
                if !root.join(asset_path).is_file() {
                    errors.push(format!(
                        "case {scoped_case_id} references nonexistent asset: {asset}"
                    ));
                }
                if let Some(first_case) =
                    referenced_assets.insert(asset.clone(), scoped_case_id.clone())
                {
                    errors.push(format!(
                        "duplicate asset reference: {asset} in {first_case} and {scoped_case_id}"
                    ));
                }
            }
        }
    }

    if suite_ids != expected_suite_ids {
        let missing = expected_suite_ids
            .difference(&suite_ids)
            .cloned()
            .collect::<Vec<_>>();
        let unexpected = suite_ids
            .difference(&expected_suite_ids)
            .cloned()
            .collect::<Vec<_>>();
        errors.push(format!(
            "suite coverage differs; missing={missing:?}, unexpected={unexpected:?}"
        ));
    }

    match fixture_assets(root) {
        Ok(actual_assets) => {
            if manifest.fixture_asset_count != actual_assets.len() {
                errors.push(format!(
                    "fixture_asset_count is {}, but {} fixture assets exist",
                    manifest.fixture_asset_count,
                    actual_assets.len()
                ));
            }
            let manifest_assets = referenced_assets.keys().cloned().collect::<BTreeSet<_>>();
            let missing = actual_assets
                .difference(&manifest_assets)
                .cloned()
                .collect::<Vec<_>>();
            let unexpected = manifest_assets
                .difference(&actual_assets)
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!("fixture coverage is missing assets: {missing:?}"));
            }
            if !unexpected.is_empty() {
                errors.push(format!(
                    "fixture coverage has unexpected assets: {unexpected:?}"
                ));
            }
        }
        Err(error) => errors.push(error),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn assert_validation_error(manifest: &Manifest, expected: &str) {
    let errors = validate_manifest(manifest, &repository_root())
        .expect_err("mutated manifest unexpectedly passed validation");
    assert!(
        errors.iter().any(|error| error.contains(expected)),
        "expected an error containing {expected:?}, got {errors:#?}"
    );
}

#[test]
fn committed_manifest_is_valid_and_exhaustive() {
    let root = repository_root();
    let manifest = load_manifest(&root);
    if let Err(errors) = validate_manifest(&manifest, &root) {
        panic!("manifest validation failed:\n{}", errors.join("\n"));
    }
}

#[test]
fn missing_asset_coverage_is_rejected() {
    let root = repository_root();
    let mut manifest = load_manifest(&root);
    manifest.suites[0].cases[0].assets.clear();
    assert_validation_error(&manifest, "fixture coverage is missing assets");
}

#[test]
fn duplicate_asset_reference_is_rejected() {
    let root = repository_root();
    let mut manifest = load_manifest(&root);
    let duplicate = manifest.suites[0].cases[0].assets[0].clone();
    manifest.suites[0].cases[1].assets.push(duplicate);
    assert_validation_error(&manifest, "duplicate asset reference");
}

#[test]
fn nonexistent_asset_reference_is_rejected() {
    let root = repository_root();
    let mut manifest = load_manifest(&root);
    manifest.suites[0].cases[0]
        .assets
        .push("conformance/yaml-decomposition/does-not-exist.yaml".to_string());
    assert_validation_error(&manifest, "references nonexistent asset");
}
