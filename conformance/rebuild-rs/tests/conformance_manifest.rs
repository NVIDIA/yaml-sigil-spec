// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

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

struct RebuiltFixtures(PathBuf);

impl RebuiltFixtures {
    fn new() -> Self {
        static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "yamlsigil-fixture-replay-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create temporary fixture root");
        let fixtures = Self(path);
        for suite in FIXTURE_SUITES {
            fs::create_dir(fixtures.0.join(suite)).expect("create fixture suite directory");
        }
        let output = Command::new(env!("CARGO_BIN_EXE_rebuild_all"))
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .env("CONFORMANCE_ROOT", &fixtures.0)
            .output()
            .expect("run fixture rebuilder");
        assert!(
            output.status.success(),
            "fixture rebuilder failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fixtures
    }

    fn read(&self, relative_path: &str) -> Vec<u8> {
        fs::read(
            self.0
                .join(relative_path.strip_prefix("conformance/").unwrap()),
        )
        .expect("read rebuilt fixture")
    }
}

impl Drop for RebuiltFixtures {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// Decode only the canonical fixture layouts used by the counterpart tests.
// Field numbers follow proto/yaml_sigil/v1alpha1/yaml_sigil.proto; the YAML
// marker and scalar rules follow artifact-decomposition.md and
// base64-requirements.md. These are test helpers, not general artifact parsers.
fn take_varint(input: &mut &[u8]) -> u64 {
    let mut value = 0;
    for shift in (0..70).step_by(7) {
        let (&byte, rest) = input.split_first().expect("complete fixture varint");
        *input = rest;
        assert!(shift < 63 || byte <= 1, "fixture varint fits u64");
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
    }
    panic!("unterminated fixture varint");
}

fn take_bytes<'a>(input: &mut &'a [u8], field: u64) -> &'a [u8] {
    assert_eq!(take_varint(input), (field << 3) | 2);
    let length = usize::try_from(take_varint(input)).expect("fixture length fits usize");
    let (bytes, rest) = input.split_at(length);
    *input = rest;
    bytes
}

fn protobuf_signature(input: &[u8]) -> (&[u8], u64, &[u8]) {
    let mut outer = input;
    let payload = take_bytes(&mut outer, 1);
    let mut inner = take_bytes(&mut outer, 2);
    assert!(outer.is_empty(), "no extra outer fixture fields");
    assert_eq!(take_varint(&mut inner), 1 << 3);
    let algorithm = take_varint(&mut inner);
    let signature = take_bytes(&mut inner, 3);
    assert!(inner.is_empty(), "no extra signature fixture fields");
    (payload, algorithm, signature)
}

fn decode_canonical_base64(scalar: &str) -> Vec<u8> {
    assert_ne!(scalar.len() % 4, 1, "valid unpadded base64 length");
    let mut decoded = Vec::new();
    let mut remainder = 0u16;
    let mut bits = 0;
    for byte in scalar.bytes() {
        let sextet = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => panic!("fixture signature uses only the URL-safe unpadded alphabet"),
        };
        remainder = (remainder << 6) | u16::from(sextet);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            decoded.push((remainder >> bits) as u8);
            remainder &= (1 << bits) - 1;
        }
    }
    assert_eq!(remainder, 0, "canonical zero trailing bits");
    decoded
}

fn yaml_signature(input: &[u8]) -> (&[u8], &str, Vec<u8>) {
    let marker = input
        .windows(4)
        .position(|bytes| bytes == b"---\n")
        .expect("fixture has a constrained marker");
    assert!(marker == 0 || input[marker - 1] == b'\n');
    let carrier = std::str::from_utf8(&input[marker + 4..]).expect("UTF-8 signature carrier");
    let fields = carrier
        .strip_prefix("schema: YamlSigilSignature.v1alpha1\nalg: ")
        .expect("canonical fixture schema and algorithm fields");
    let (algorithm, signature_field) = fields.split_once('\n').expect("algorithm line ending");
    let scalar = signature_field
        .strip_prefix("signature: ")
        .and_then(|line| line.strip_suffix('\n'))
        .expect("canonical fixture signature field");
    (&input[..marker], algorithm, decode_canonical_base64(scalar))
}

#[test]
fn regeneration_matches_every_committed_fixture() {
    let root = repository_root();
    let rebuilt = RebuiltFixtures::new();
    let committed_assets = fixture_assets(&root).unwrap();
    for asset in &committed_assets {
        assert_eq!(
            rebuilt.read(asset),
            fs::read(root.join(asset)).expect("read committed fixture"),
            "regeneration differs for {asset}"
        );
    }
    let generated_assets = FIXTURE_SUITES
        .iter()
        .flat_map(|suite| {
            fs::read_dir(rebuilt.0.join(suite))
                .unwrap()
                .map(|entry| {
                    let name = entry.unwrap().file_name().into_string().unwrap();
                    format!("conformance/{suite}/{name}")
                })
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(generated_assets, committed_assets);
}

#[test]
fn malformed_signature_counterparts_match_in_both_forms() {
    // Independent inventory from issue #97: omitting any generator output,
    // manifest expectation, or sidecar invocation must fail this test.
    let ed25519 = (
        "alg-ed25519",
        "noncanonical-encodings",
        "noncanonical-encoding",
    );
    let ecdsa_ranges = (
        "alg-ecdsa",
        "invalid-component-ranges",
        "invalid-component-ranges",
    );
    let ecdsa_lengths = ("alg-ecdsa", "non-fixed-width", "non-fixed-width");
    let cases = [
        (ed25519, "noncanonical-R", 64),
        (ed25519, "noncanonical-S-equals-L", 64),
        (ed25519, "noncanonical-S-equals-L-plus-1", 64),
        (ecdsa_ranges, "invalid-r-zero", 64),
        (ecdsa_ranges, "invalid-s-zero", 64),
        (ecdsa_ranges, "invalid-r-equals-n", 64),
        (ecdsa_ranges, "invalid-s-equals-n", 64),
        (ecdsa_lengths, "signature-63-bytes", 63),
        (ecdsa_lengths, "signature-65-bytes", 65),
    ];
    let root = repository_root();
    let manifest = load_manifest(&root);
    for ((suite_id, case_id, sidecar_stem), stem, length) in cases {
        let (algorithm, algorithm_name, expected_payload, key_source, key_prefix) = match suite_id {
            "alg-ed25519" => (
                1,
                "ED25519_PUREEDDSA_RAW_RS64_CANONICAL",
                b"".as_slice(),
                "rfc8032-vec1-empty-message",
                "# public_key: ",
            ),
            "alg-ecdsa" => (
                2,
                "ECDSA_SECP256R1_SHA256_RAW_RS64",
                b"hello: world\n".as_slice(),
                "verify-happy-path",
                "# public key Q (uncompressed): ",
            ),
            _ => unreachable!(),
        };
        let suite_root = root.join("conformance").join(suite_id);
        let protobuf = fs::read(suite_root.join(format!("{stem}.binpb"))).unwrap();
        let yaml = fs::read(suite_root.join(format!("{stem}.yaml"))).unwrap();
        let (proto_payload, proto_algorithm, proto_signature) = protobuf_signature(&protobuf);
        let (yaml_payload, yaml_algorithm, yaml_signature) = yaml_signature(&yaml);
        assert_eq!(proto_payload, expected_payload, "{stem} payload");
        assert_eq!(yaml_payload, proto_payload, "{stem} cross-form payload");
        assert_eq!(proto_algorithm, algorithm, "{stem} protobuf algorithm");
        assert_eq!(yaml_algorithm, algorithm_name, "{stem} YAML algorithm");
        assert_eq!(yaml_signature, proto_signature, "{stem} decoded signature");
        assert_eq!(proto_signature.len(), length, "{stem} signature length");

        let key_sidecar =
            fs::read_to_string(suite_root.join(format!("{key_source}.expected.txt"))).unwrap();
        let public_key = key_sidecar
            .lines()
            .find_map(|line| line.strip_prefix(key_prefix))
            .unwrap();
        let sidecar =
            fs::read_to_string(suite_root.join(format!("{sidecar_stem}.expected.txt"))).unwrap();
        assert!(sidecar.contains(&format!("# public_key: {public_key}\n")));
        assert!(sidecar.contains(&format!("# Algorithm enabled: {algorithm_name}.\n")));
        let case = manifest
            .suites
            .iter()
            .find(|suite| suite.id == suite_id)
            .unwrap()
            .cases
            .iter()
            .find(|case| case.id == case_id)
            .unwrap();
        assert_eq!(case.context.forms, ["YAML", "PROTOBUF"]);
        for (form, extension) in [("PROTOBUF", "binpb"), ("YAML", "yaml")] {
            let asset = format!("{stem}.{extension}");
            assert!(case
                .assets
                .contains(&format!("conformance/{suite_id}/{asset}")));
            assert!(case.expectations.iter().any(|expectation| {
                expectation.kind == "verifier_state"
                    && expectation.details["value"] == "VERIFIER_STATE_MALFORMED_ATTEMPTED_SIGNED"
                    && expectation.details["assets"]
                        .as_array()
                        .unwrap()
                        .contains(&Value::String(asset.clone()))
            }));
            let invocation = format!("Verify(form={form}, input={asset},\n");
            let remainder = sidecar
                .split_once(&invocation)
                .expect("sidecar names form and fixture")
                .1;
            let mut lines = remainder.lines();
            assert!(lines.next().unwrap().contains("config.public_key_handle="));
            assert_eq!(lines.next(), Some("Expected: MalformedAttemptedSigned"));
        }
    }
}
