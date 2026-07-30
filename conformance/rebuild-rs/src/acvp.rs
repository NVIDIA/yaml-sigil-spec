// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Vendored NIST ACVP-Server ECDSA SigGen FIPS 186-5 test vectors.
//!
//! The data file is embedded at compile time via `include_str!`; the
//! exact upstream pin and how to verify the bytes by hand are
//! documented in [`../vendor/acvp/README.md`](../vendor/acvp/README.md).
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

use serde::Deserialize;

/// Embedded copy of the vendored ACVP file (see `vendor/acvp/`).
pub const VENDORED_JSON: &str = include_str!("../vendor/acvp/ECDSA-SigGen-FIPS186-5.json");

/// The upstream commit hash the vendored file was pulled from.
/// Kept here in addition to `vendor/acvp/README.md` so the generator
/// can stamp it into the per-fixture `.expected.txt` sidecar.
pub const VENDORED_COMMIT: &str = "15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0";

/// The upstream path within the ACVP-Server repo.
pub const VENDORED_PATH: &str = "gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json";

#[derive(Deserialize)]
pub struct AcvpFile {
    #[serde(rename = "testGroups")]
    pub test_groups: Vec<AcvpGroup>,
}

#[derive(Deserialize)]
pub struct AcvpGroup {
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

#[derive(Deserialize)]
pub struct AcvpCase {
    #[serde(rename = "tcId")]
    pub tc_id: u64,
    pub k: String,
    pub message: String,
    pub r: String,
    pub s: String,
}

/// Parse the vendored JSON. Panics on schema drift — the file is
/// pinned to a specific upstream commit and any structural change is
/// itself a finding that needs surfacing, not silently degraded.
pub fn load() -> AcvpFile {
    serde_json::from_str(VENDORED_JSON).expect("vendored ACVP JSON parses")
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
    file.test_groups.iter().filter(|g| {
        g.curve == "P-256"
            && g.hash_alg == "SHA2-256"
            && g.test_type == "AFT"
            && g.conformance.is_none()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The vendored JSON MUST parse. If this fails, either the file
    /// drifted from its upstream pin (see `vendor/acvp/README.md`) or
    /// the upstream schema changed under the same commit.
    #[test]
    fn vendored_json_parses() {
        let file = load();
        assert!(!file.test_groups.is_empty());
    }

    /// There MUST be at least one P-256 / SHA-256 AFT group — that's
    /// the entire reason we're vendoring this file.
    #[test]
    fn at_least_one_p256_sha256_aft_group_exists() {
        let file = load();
        let mut groups = p256_sha256_aft_groups(&file);
        let g = groups.next().expect("at least one P-256/SHA-256 AFT group");
        assert!(!g.tests.is_empty());
    }
}
