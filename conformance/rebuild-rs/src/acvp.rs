// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Vendored NIST ACVP-Server ECDSA SigGen FIPS 186-5 test vectors.
//!
//! The data file is opened at runtime through pinned, no-follow directory
//! handles. A limit-plus-one read captures one bounded byte snapshot; a
//! storage-free structural pass validates its work limits before the same
//! bytes are deserialized and made available for cryptographic replay. The
//! exact upstream pin and how to verify the bytes by hand are documented in
//! [`../vendor/acvp/README.md`](../vendor/acvp/README.md).
//!
//! ## File shape (relevant subset)
//!
//! The pinned ACVP-Server `internalProjection.json` exposes test groups,
//! each with a curve/hash combination and a `tests` array. In the vendored
//! file, the selected AFT groups carry `d`, `qx`, and `qy`, and each case
//! carries `tcId`, `k`, `message`, `r`, and `s`. This is an NVIDIA-authored
//! summary of the pinned JSON shape, not a quotation from the evolving
//! [ACVP ECDSA draft](https://pages.nist.gov/ACVP/draft-fussell-acvp-ecdsa.html).
//! The exact vendored source and NIST terms are recorded in the repository
//! `THIRD_PARTY_NOTICES.md`.
//!
//! For the `verify-happy-path` conformance artifact we use the first
//! AFT test case of the first `curve = "P-256" / hashAlg = "SHA2-256"`
//! group. The [`test_replay_all_p256_sha256`] (in `alg_ecdsa.rs`)
//! walks **all** such cases and asserts our hand-rolled signer
//! reproduces the published `(r, s)` byte-for-byte.

use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::io;
use std::path::Path;
use yamlsigil_pinned_dir::PinnedDir;

const VENDORED_FILE: &str = "ECDSA-SigGen-FIPS186-5.json";

/// Maximum encoded size of the vendored ACVP snapshot.
pub const MAX_CORPUS_BYTES: usize = 3 * 1024 * 1024;

const MAX_GROUPS: usize = 512;
const MAX_CASES_PER_GROUP: usize = 64;
const MAX_TOTAL_CASES: usize = 4_096;
const MAX_REPLAY_GROUPS: usize = 8;
const MAX_REPLAY_CASES: usize = 256;
const MAX_TOKEN_BYTES: usize = 64;
const MAX_SCALAR_HEX_CHARS: usize = 160;
const MAX_MESSAGE_HEX_CHARS: usize = 4_096;
const MAX_RANDOM_VALUE_HEX_CHARS: usize = 256;

/// The upstream commit hash the vendored file was pulled from.
/// Kept here in addition to `vendor/acvp/README.md` so the generator
/// can stamp it into the per-fixture `.expected.txt` sidecar.
pub const VENDORED_COMMIT: &str = "15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0";

/// The upstream path within the ACVP-Server repo.
pub const VENDORED_PATH: &str = "gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcvpFile {
    algorithm: String,
    #[serde(rename = "isSample")]
    is_sample: bool,
    mode: String,
    revision: String,
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<AcvpGroup>,
    #[serde(rename = "vsId")]
    vs_id: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcvpGroup {
    #[serde(rename = "componentTest")]
    _component_test: bool,
    #[serde(rename = "tgId")]
    pub tg_id: u64,
    #[serde(rename = "testType")]
    pub test_type: String,
    pub curve: String,
    #[serde(rename = "hashAlg")]
    pub hash_alg: String,
    /// Some groups carry a `conformance` tag (e.g. `"SP800-106"` for
    /// the NIST randomized-hashing extension, which prepends a
    /// `randomValue` to the message before hashing). Our slot does
    /// plain `SHA-256(message)` — see [`p256_sha256_aft_groups`] for
    /// the filter that rejects those non-baseline groups.
    #[serde(default)]
    pub conformance: Option<String>,
    pub d: String,
    pub qx: String,
    pub qy: String,
    pub tests: Vec<AcvpCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcvpCase {
    #[serde(rename = "deferred")]
    _deferred: bool,
    #[serde(rename = "tcId")]
    pub tc_id: u64,
    pub k: String,
    pub message: String,
    pub r: String,
    #[serde(rename = "randomValue")]
    random_value: Option<String>,
    #[serde(rename = "randomValueLen")]
    random_value_len: Option<u64>,
    pub s: String,
}

/// Read and parse the vendored JSON through one bounded, pinned snapshot.
///
/// Schema or bound drift is an `InvalidData` error. Callers surface it before
/// writing fixtures or performing any P-256 replay.
pub fn load() -> io::Result<AcvpFile> {
    let manifest = PinnedDir::open(Path::new(env!("CARGO_MANIFEST_DIR")))?;
    let vendor = manifest.open_child("vendor")?;
    let acvp = vendor.open_child("acvp")?;
    let bytes = acvp.read_regular_file_bounded(VENDORED_FILE, MAX_CORPUS_BYTES)?;
    parse_bounded(&bytes)
}

fn parse_bounded(bytes: &[u8]) -> io::Result<AcvpFile> {
    if bytes.len() > MAX_CORPUS_BYTES {
        return Err(invalid_data(format!(
            "ACVP corpus exceeds {MAX_CORPUS_BYTES}-byte limit"
        )));
    }

    // The first pass retains no corpus collection or field string. It proves
    // allocation and replay limits for this exact immutable byte slice.
    let mut budget = PreflightBudget::default();
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    FileSeed {
        budget: &mut budget,
    }
    .deserialize(&mut deserializer)
    .map_err(json_error)?;
    deserializer.end().map_err(json_error)?;

    // Deserialization reparses the already-preflighted byte snapshot, never a
    // pathname that could resolve to different bytes between the two passes.
    let file: AcvpFile = serde_json::from_slice(bytes).map_err(json_error)?;
    validate_loaded_file(&file)?;
    Ok(file)
}

fn validate_loaded_file(file: &AcvpFile) -> io::Result<()> {
    if file.algorithm != "ECDSA"
        || !file.is_sample
        || file.mode != "sigGen"
        || file.revision != "FIPS186-5"
        || file.vs_id != 0
    {
        return Err(invalid_data(
            "ACVP top-level identity does not match the pinned snapshot",
        ));
    }

    let mut group_ids = HashSet::with_capacity(file.test_groups.len());
    let mut case_ids = HashSet::new();
    let mut total_cases = 0usize;
    let mut replay_groups = 0usize;
    let mut replay_cases = 0usize;

    for group in &file.test_groups {
        if !group_ids.insert(group.tg_id) {
            return Err(invalid_data(format!(
                "duplicate ACVP test-group ID {}",
                group.tg_id
            )));
        }
        total_cases = total_cases
            .checked_add(group.tests.len())
            .ok_or_else(|| invalid_data("ACVP total case count overflow"))?;
        if total_cases > MAX_TOTAL_CASES {
            return Err(invalid_data(format!(
                "ACVP total cases exceed {MAX_TOTAL_CASES}"
            )));
        }
        for case in &group.tests {
            if !case_ids.insert(case.tc_id) {
                return Err(invalid_data(format!(
                    "duplicate ACVP test-case ID {}",
                    case.tc_id
                )));
            }
            if case.random_value.is_some() != case.random_value_len.is_some() {
                return Err(invalid_data(format!(
                    "ACVP tcId {} has incomplete randomized-hashing metadata",
                    case.tc_id
                )));
            }
        }

        if is_p256_sha256_aft(group) {
            replay_groups += 1;
            if replay_groups > MAX_REPLAY_GROUPS {
                return Err(invalid_data(format!(
                    "ACVP replay groups exceed {MAX_REPLAY_GROUPS}"
                )));
            }
            validate_exact_p256_scalar("d", group.tg_id, None, &group.d)?;
            validate_exact_p256_scalar("qx", group.tg_id, None, &group.qx)?;
            validate_exact_p256_scalar("qy", group.tg_id, None, &group.qy)?;

            for case in &group.tests {
                replay_cases += 1;
                if replay_cases > MAX_REPLAY_CASES {
                    return Err(invalid_data(format!(
                        "ACVP replay cases exceed {MAX_REPLAY_CASES}"
                    )));
                }
                validate_exact_p256_scalar("k", group.tg_id, Some(case.tc_id), &case.k)?;
                validate_exact_p256_scalar("r", group.tg_id, Some(case.tc_id), &case.r)?;
                validate_exact_p256_scalar("s", group.tg_id, Some(case.tc_id), &case.s)?;
            }
        }
    }
    Ok(())
}

fn validate_exact_p256_scalar(
    field: &str,
    group_id: u64,
    case_id: Option<u64>,
    value: &str,
) -> io::Result<()> {
    if value.len() != 64 {
        let location = case_id.map_or_else(
            || format!("tgId {group_id}"),
            |id| format!("tgId {group_id}, tcId {id}"),
        );
        return Err(invalid_data(format!(
            "ACVP {location} field {field} must contain exactly 64 hex characters"
        )));
    }
    Ok(())
}

fn is_p256_sha256_aft(group: &AcvpGroup) -> bool {
    group.curve == "P-256"
        && group.hash_alg == "SHA2-256"
        && group.test_type == "AFT"
        && group.conformance.is_none()
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn json_error(error: serde_json::Error) -> io::Error {
    invalid_data(format!("invalid bounded ACVP JSON: {error}"))
}

#[derive(Default)]
struct PreflightBudget {
    groups: usize,
    cases: usize,
}

#[derive(Clone, Copy)]
enum TextRule {
    Token,
    Hex { allow_empty: bool },
}

#[derive(Clone, Copy)]
struct TextSeed {
    field: &'static str,
    max_bytes: usize,
    rule: TextRule,
    expected: Option<&'static str>,
}

impl TextSeed {
    const fn token(field: &'static str) -> Self {
        Self {
            field,
            max_bytes: MAX_TOKEN_BYTES,
            rule: TextRule::Token,
            expected: None,
        }
    }

    const fn expected(field: &'static str, expected: &'static str) -> Self {
        Self {
            field,
            max_bytes: MAX_TOKEN_BYTES,
            rule: TextRule::Token,
            expected: Some(expected),
        }
    }

    const fn scalar(field: &'static str) -> Self {
        Self {
            field,
            max_bytes: MAX_SCALAR_HEX_CHARS,
            rule: TextRule::Hex { allow_empty: false },
            expected: None,
        }
    }

    const fn message() -> Self {
        Self {
            field: "message",
            max_bytes: MAX_MESSAGE_HEX_CHARS,
            rule: TextRule::Hex { allow_empty: true },
            expected: None,
        }
    }

    const fn random_value() -> Self {
        Self {
            field: "randomValue",
            max_bytes: MAX_RANDOM_VALUE_HEX_CHARS,
            rule: TextRule::Hex { allow_empty: true },
            expected: None,
        }
    }
}

impl<'de> DeserializeSeed<'de> for TextSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(TextVisitor(self))
    }
}

struct TextVisitor(TextSeed);

impl Visitor<'_> for TextVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "bounded ACVP {} string", self.0.field)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        validate_text(value, self.0).map_err(E::custom)
    }

    fn visit_borrowed_str<E>(self, value: &'_ str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(value)
    }
}

fn validate_text(value: &str, seed: TextSeed) -> Result<(), String> {
    if value.len() > seed.max_bytes {
        return Err(format!(
            "ACVP field {} exceeds {} bytes",
            seed.field, seed.max_bytes
        ));
    }
    if let Some(expected) = seed.expected {
        if value != expected {
            return Err(format!("ACVP field {} must equal {expected:?}", seed.field));
        }
    }
    match seed.rule {
        TextRule::Token => {
            if value.is_empty() || !value.bytes().all(|byte| (0x21..=0x7e).contains(&byte)) {
                return Err(format!(
                    "ACVP field {} must be nonempty printable ASCII",
                    seed.field
                ));
            }
        }
        TextRule::Hex { allow_empty } => {
            if (!allow_empty && value.is_empty())
                || !value.len().is_multiple_of(2)
                || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(format!(
                    "ACVP field {} must be even-length ASCII hex",
                    seed.field
                ));
            }
        }
    }
    Ok(())
}

fn mark_once<E>(seen: &mut bool, field: &'static str) -> Result<(), E>
where
    E: de::Error,
{
    if std::mem::replace(seen, true) {
        Err(E::duplicate_field(field))
    } else {
        Ok(())
    }
}

fn require_field<E>(seen: bool, field: &'static str) -> Result<(), E>
where
    E: de::Error,
{
    if seen {
        Ok(())
    } else {
        Err(E::missing_field(field))
    }
}

struct FileSeed<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> DeserializeSeed<'de> for FileSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(FileVisitor {
            budget: self.budget,
        })
    }
}

struct FileVisitor<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> Visitor<'de> for FileVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the bounded ACVP top-level object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut algorithm = false;
        let mut is_sample = false;
        let mut mode = false;
        let mut revision = false;
        let mut test_groups = false;
        let mut vs_id = false;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "algorithm" => {
                    mark_once(&mut algorithm, "algorithm")?;
                    map.next_value_seed(TextSeed::expected("algorithm", "ECDSA"))?;
                }
                "isSample" => {
                    mark_once(&mut is_sample, "isSample")?;
                    let value: bool = map.next_value()?;
                    if !value {
                        return Err(de::Error::custom("ACVP isSample must be true"));
                    }
                }
                "mode" => {
                    mark_once(&mut mode, "mode")?;
                    map.next_value_seed(TextSeed::expected("mode", "sigGen"))?;
                }
                "revision" => {
                    mark_once(&mut revision, "revision")?;
                    map.next_value_seed(TextSeed::expected("revision", "FIPS186-5"))?;
                }
                "testGroups" => {
                    mark_once(&mut test_groups, "testGroups")?;
                    map.next_value_seed(GroupsSeed {
                        budget: self.budget,
                    })?;
                }
                "vsId" => {
                    mark_once(&mut vs_id, "vsId")?;
                    let value: u64 = map.next_value()?;
                    if value != 0 {
                        return Err(de::Error::custom("ACVP vsId must be zero"));
                    }
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        key,
                        &[
                            "algorithm",
                            "isSample",
                            "mode",
                            "revision",
                            "testGroups",
                            "vsId",
                        ],
                    ));
                }
            }
        }

        require_field(algorithm, "algorithm")?;
        require_field(is_sample, "isSample")?;
        require_field(mode, "mode")?;
        require_field(revision, "revision")?;
        require_field(test_groups, "testGroups")?;
        require_field(vs_id, "vsId")
    }
}

struct GroupsSeed<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> DeserializeSeed<'de> for GroupsSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(GroupsVisitor {
            budget: self.budget,
        })
    }
}

struct GroupsVisitor<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> Visitor<'de> for GroupsVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded ACVP testGroups array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        loop {
            if self.budget.groups == MAX_GROUPS {
                if sequence.next_element::<IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom(format!(
                        "ACVP testGroups exceed {MAX_GROUPS}"
                    )));
                }
                return Ok(());
            }
            match sequence.next_element_seed(GroupSeed {
                budget: self.budget,
            })? {
                Some(()) => self.budget.groups += 1,
                None => return Ok(()),
            }
        }
    }
}

struct GroupSeed<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> DeserializeSeed<'de> for GroupSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(GroupVisitor {
            budget: self.budget,
        })
    }
}

struct GroupVisitor<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> Visitor<'de> for GroupVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded ACVP test-group object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut component_test = false;
        let mut conformance = false;
        let mut curve = false;
        let mut d = false;
        let mut hash_alg = false;
        let mut qx = false;
        let mut qy = false;
        let mut test_type = false;
        let mut tests = false;
        let mut tg_id = false;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "componentTest" => {
                    mark_once(&mut component_test, "componentTest")?;
                    let _: bool = map.next_value()?;
                }
                "conformance" => {
                    mark_once(&mut conformance, "conformance")?;
                    map.next_value_seed(TextSeed::token("conformance"))?;
                }
                "curve" => {
                    mark_once(&mut curve, "curve")?;
                    map.next_value_seed(TextSeed::token("curve"))?;
                }
                "d" => {
                    mark_once(&mut d, "d")?;
                    map.next_value_seed(TextSeed::scalar("d"))?;
                }
                "hashAlg" => {
                    mark_once(&mut hash_alg, "hashAlg")?;
                    map.next_value_seed(TextSeed::token("hashAlg"))?;
                }
                "qx" => {
                    mark_once(&mut qx, "qx")?;
                    map.next_value_seed(TextSeed::scalar("qx"))?;
                }
                "qy" => {
                    mark_once(&mut qy, "qy")?;
                    map.next_value_seed(TextSeed::scalar("qy"))?;
                }
                "testType" => {
                    mark_once(&mut test_type, "testType")?;
                    map.next_value_seed(TextSeed::token("testType"))?;
                }
                "tests" => {
                    mark_once(&mut tests, "tests")?;
                    map.next_value_seed(CasesSeed {
                        budget: self.budget,
                    })?;
                }
                "tgId" => {
                    mark_once(&mut tg_id, "tgId")?;
                    let _: u64 = map.next_value()?;
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        key,
                        &[
                            "componentTest",
                            "conformance",
                            "curve",
                            "d",
                            "hashAlg",
                            "qx",
                            "qy",
                            "testType",
                            "tests",
                            "tgId",
                        ],
                    ));
                }
            }
        }

        require_field(component_test, "componentTest")?;
        require_field(curve, "curve")?;
        require_field(d, "d")?;
        require_field(hash_alg, "hashAlg")?;
        require_field(qx, "qx")?;
        require_field(qy, "qy")?;
        require_field(test_type, "testType")?;
        require_field(tests, "tests")?;
        require_field(tg_id, "tgId")
    }
}

struct CasesSeed<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> DeserializeSeed<'de> for CasesSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(CasesVisitor {
            budget: self.budget,
        })
    }
}

struct CasesVisitor<'a> {
    budget: &'a mut PreflightBudget,
}

impl<'de> Visitor<'de> for CasesVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded ACVP tests array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut group_cases = 0usize;
        loop {
            if group_cases == MAX_CASES_PER_GROUP || self.budget.cases == MAX_TOTAL_CASES {
                if sequence.next_element::<IgnoredAny>()?.is_some() {
                    let message = if group_cases == MAX_CASES_PER_GROUP {
                        format!("ACVP tests exceed {MAX_CASES_PER_GROUP} per group")
                    } else {
                        format!("ACVP total cases exceed {MAX_TOTAL_CASES}")
                    };
                    return Err(de::Error::custom(message));
                }
                return Ok(());
            }
            match sequence.next_element_seed(CaseSeed)? {
                Some(()) => {
                    group_cases += 1;
                    self.budget.cases += 1;
                }
                None => return Ok(()),
            }
        }
    }
}

struct CaseSeed;

impl<'de> DeserializeSeed<'de> for CaseSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(CaseVisitor)
    }
}

struct CaseVisitor;

impl<'de> Visitor<'de> for CaseVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded ACVP test-case object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut deferred = false;
        let mut k = false;
        let mut message = false;
        let mut r = false;
        let mut random_value = false;
        let mut random_value_len = false;
        let mut s = false;
        let mut tc_id = false;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "deferred" => {
                    mark_once(&mut deferred, "deferred")?;
                    let _: bool = map.next_value()?;
                }
                "k" => {
                    mark_once(&mut k, "k")?;
                    map.next_value_seed(TextSeed::scalar("k"))?;
                }
                "message" => {
                    mark_once(&mut message, "message")?;
                    map.next_value_seed(TextSeed::message())?;
                }
                "r" => {
                    mark_once(&mut r, "r")?;
                    map.next_value_seed(TextSeed::scalar("r"))?;
                }
                "randomValue" => {
                    mark_once(&mut random_value, "randomValue")?;
                    map.next_value_seed(TextSeed::random_value())?;
                }
                "randomValueLen" => {
                    mark_once(&mut random_value_len, "randomValueLen")?;
                    let _: u64 = map.next_value()?;
                }
                "s" => {
                    mark_once(&mut s, "s")?;
                    map.next_value_seed(TextSeed::scalar("s"))?;
                }
                "tcId" => {
                    mark_once(&mut tc_id, "tcId")?;
                    let _: u64 = map.next_value()?;
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        key,
                        &[
                            "deferred",
                            "k",
                            "message",
                            "r",
                            "randomValue",
                            "randomValueLen",
                            "s",
                            "tcId",
                        ],
                    ));
                }
            }
        }

        require_field(deferred, "deferred")?;
        require_field(k, "k")?;
        require_field(message, "message")?;
        require_field(r, "r")?;
        require_field(s, "s")?;
        require_field(tc_id, "tcId")
    }
}

/// Filter for the AFT (Algorithm Functional Test) groups that match
/// curve = P-256 and hash = SHA2-256, AND use the baseline hashing
/// rule (no `conformance` tag).
///
/// Groups marked `conformance: "SP800-106"` use NIST randomized
/// hashing (a `randomValue` is prepended to the message before
/// `SHA-256`), which is NOT what this crate's
/// `ECDSA_SECP256R1_SHA256_RAW_RS64` slot specifies. Those groups
/// are filtered out so the replay assertions don't false-fail.
pub fn p256_sha256_aft_groups(file: &AcvpFile) -> impl Iterator<Item = &AcvpGroup> {
    file.test_groups
        .iter()
        .filter(|group| is_p256_sha256_aft(group))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vendored JSON MUST parse. If this fails, either the file
    /// drifted from its upstream pin (see `vendor/acvp/README.md`) or
    /// the upstream schema changed under the same commit.
    #[test]
    fn vendored_json_parses() {
        let file = load().expect("vendored ACVP JSON parses within bounds");
        assert!(!file.test_groups.is_empty());
    }

    /// There MUST be at least one P-256 / SHA-256 AFT group — that's
    /// the entire reason we're vendoring this file.
    #[test]
    fn at_least_one_p256_sha256_aft_group_exists() {
        let file = load().expect("vendored ACVP JSON parses within bounds");
        let mut groups = p256_sha256_aft_groups(&file);
        let g = groups.next().expect("at least one P-256/SHA-256 AFT group");
        assert!(!g.tests.is_empty());
    }

    fn file_json(groups: &[String]) -> String {
        format!(
            concat!(
                "{{\"algorithm\":\"ECDSA\",\"isSample\":true,",
                "\"mode\":\"sigGen\",\"revision\":\"FIPS186-5\",",
                "\"testGroups\":[{}],\"vsId\":0}}"
            ),
            groups.join(",")
        )
    }

    fn case_json(case_id: usize, message: &str) -> String {
        let scalar = "00".repeat(32);
        format!(
            concat!(
                "{{\"deferred\":false,\"k\":\"{scalar}\",",
                "\"message\":\"{message}\",\"r\":\"{scalar}\",",
                "\"s\":\"{scalar}\",\"tcId\":{case_id}}}"
            ),
            scalar = scalar,
            message = message,
            case_id = case_id,
        )
    }

    fn group_json(group_id: usize, curve: &str, cases: &[String]) -> String {
        let scalar = "00".repeat(32);
        format!(
            concat!(
                "{{\"componentTest\":false,\"curve\":\"{curve}\",",
                "\"d\":\"{scalar}\",\"hashAlg\":\"SHA2-256\",",
                "\"qx\":\"{scalar}\",\"qy\":\"{scalar}\",",
                "\"testType\":\"AFT\",\"tests\":[{}],",
                "\"tgId\":{group_id}}}"
            ),
            cases.join(","),
            curve = curve,
            scalar = scalar,
            group_id = group_id,
        )
    }

    fn groups_with_cases(group_count: usize, cases_per_group: usize, curve: &str) -> Vec<String> {
        let mut next_case = 1usize;
        (0..group_count)
            .map(|group| {
                let cases = (0..cases_per_group)
                    .map(|_| {
                        let case = case_json(next_case, "00");
                        next_case += 1;
                        case
                    })
                    .collect::<Vec<_>>();
                group_json(group + 1, curve, &cases)
            })
            .collect()
    }

    #[test]
    fn corpus_byte_limit_accepts_exact_and_rejects_limit_plus_one() {
        let mut exact = file_json(&[]).into_bytes();
        exact.resize(MAX_CORPUS_BYTES, b' ');
        parse_bounded(&exact).expect("exact corpus byte limit must pass");

        exact.push(b' ');
        let error = parse_bounded(&exact).expect_err("limit-plus-one corpus must fail");
        assert!(error.to_string().contains("exceeds 3145728-byte limit"));
    }

    #[test]
    fn group_limit_accepts_exact_and_rejects_limit_plus_one() {
        let exact = groups_with_cases(MAX_GROUPS, 0, "P-384");
        parse_bounded(file_json(&exact).as_bytes()).expect("exact group limit must pass");

        let too_many = groups_with_cases(MAX_GROUPS + 1, 0, "P-384");
        let error = parse_bounded(file_json(&too_many).as_bytes())
            .expect_err("limit-plus-one groups must fail");
        assert!(error.to_string().contains("testGroups exceed 512"));
    }

    #[test]
    fn per_group_case_limit_accepts_exact_and_rejects_limit_plus_one() {
        let exact = groups_with_cases(1, MAX_CASES_PER_GROUP, "P-384");
        parse_bounded(file_json(&exact).as_bytes()).expect("exact per-group limit must pass");

        let too_many = groups_with_cases(1, MAX_CASES_PER_GROUP + 1, "P-384");
        let error = parse_bounded(file_json(&too_many).as_bytes())
            .expect_err("limit-plus-one group cases must fail");
        assert!(error.to_string().contains("tests exceed 64 per group"));
    }

    #[test]
    fn total_case_limit_accepts_exact_and_rejects_limit_plus_one() {
        let exact = groups_with_cases(
            MAX_TOTAL_CASES / MAX_CASES_PER_GROUP,
            MAX_CASES_PER_GROUP,
            "P-384",
        );
        parse_bounded(file_json(&exact).as_bytes()).expect("exact total case limit must pass");

        let mut too_many = exact;
        too_many.push(group_json(
            too_many.len() + 1,
            "P-384",
            &[case_json(MAX_TOTAL_CASES + 1, "00")],
        ));
        let error = parse_bounded(file_json(&too_many).as_bytes())
            .expect_err("limit-plus-one total cases must fail");
        assert!(error.to_string().contains("total cases exceed 4096"));
    }

    #[test]
    fn replay_group_limit_accepts_exact_and_rejects_limit_plus_one() {
        let exact = groups_with_cases(MAX_REPLAY_GROUPS, 1, "P-256");
        parse_bounded(file_json(&exact).as_bytes()).expect("exact replay group limit must pass");

        let too_many = groups_with_cases(MAX_REPLAY_GROUPS + 1, 1, "P-256");
        let error = parse_bounded(file_json(&too_many).as_bytes())
            .expect_err("limit-plus-one replay groups must fail");
        assert!(error.to_string().contains("replay groups exceed 8"));
    }

    #[test]
    fn replay_case_limit_accepts_exact_and_rejects_limit_plus_one() {
        let exact = groups_with_cases(
            MAX_REPLAY_CASES / MAX_CASES_PER_GROUP,
            MAX_CASES_PER_GROUP,
            "P-256",
        );
        parse_bounded(file_json(&exact).as_bytes()).expect("exact replay case limit must pass");

        let too_many = groups_with_cases(
            MAX_REPLAY_CASES / MAX_CASES_PER_GROUP + 1,
            MAX_CASES_PER_GROUP,
            "P-256",
        );
        let error = parse_bounded(file_json(&too_many).as_bytes())
            .expect_err("limit-plus-one replay cases must fail");
        assert!(error.to_string().contains("replay cases exceed 256"));
    }

    #[test]
    fn message_limit_accepts_exact_and_rejects_limit_plus_two() {
        let exact_case = case_json(1, &"00".repeat(MAX_MESSAGE_HEX_CHARS / 2));
        let exact = vec![group_json(1, "P-384", &[exact_case])];
        parse_bounded(file_json(&exact).as_bytes()).expect("exact message limit must pass");

        let large_case = case_json(1, &"00".repeat(MAX_MESSAGE_HEX_CHARS / 2 + 1));
        let large = vec![group_json(1, "P-384", &[large_case])];
        let error = parse_bounded(file_json(&large).as_bytes())
            .expect_err("limit-plus-two message must fail");
        assert!(error
            .to_string()
            .contains("field message exceeds 4096 bytes"));
    }

    #[test]
    fn malformed_noncanonical_inputs_fail_before_replay() {
        let valid = file_json(&groups_with_cases(1, 1, "P-384"));

        let incomplete = valid.replacen("\"s\":\"", "\"omittedS\":\"", 1);
        assert!(parse_bounded(incomplete.as_bytes()).is_err());

        let duplicate = valid.replacen(
            "\"curve\":\"P-384\"",
            "\"curve\":\"P-384\",\"curve\":\"P-384\"",
            1,
        );
        assert!(parse_bounded(duplicate.as_bytes()).is_err());

        let unknown_nested = valid.replacen(
            "\"testGroups\"",
            "\"unknown\":[[[[[0]]]]],\"testGroups\"",
            1,
        );
        assert!(parse_bounded(unknown_nested.as_bytes()).is_err());

        let mut trailing = valid;
        trailing.push('x');
        assert!(parse_bounded(trailing.as_bytes()).is_err());
    }

    #[test]
    fn duplicate_group_and_case_ids_are_rejected() {
        let duplicate_groups = vec![group_json(1, "P-384", &[]), group_json(1, "P-384", &[])];
        assert!(parse_bounded(file_json(&duplicate_groups).as_bytes()).is_err());

        let duplicate_cases = vec![group_json(
            1,
            "P-384",
            &[case_json(1, "00"), case_json(1, "00")],
        )];
        assert!(parse_bounded(file_json(&duplicate_cases).as_bytes()).is_err());
    }
}
