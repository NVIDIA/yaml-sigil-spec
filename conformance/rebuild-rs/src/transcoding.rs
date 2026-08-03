// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/transcoding/` fixtures.
//!
//! These fixtures exercise the semantic YAML-string and signature-octet
//! round-trip requirements in `transcoding.md`. The empty, Boolean-like,
//! null-like, and numeric-looking case categories come from the plain-scalar
//! tag-resolution patterns in
//! [YAML 1.2.2 §10.3.2](https://yaml.org/spec/1.2.2/#1032-tag-resolution).
//! The fixture strings and inverse-encoded octets are locally selected rather
//! than copied test vectors.
//!
//! Signature text uses the URL-safe unpadded base64 encoder in [`crate::b64`].
//! That module records the RFC 4648 source, applicable terms, and exact
//! alphabet used by this generator. Unit tests below prove that each locally
//! selected octet string encodes to the YAML-looking text named by its case.
//!
//! Each case produces an independently written YAML/protobuf pair. The YAML
//! member uses one accepted scalar presentation as fixture input; it does not
//! prescribe the scalar presentation a conforming transcoder emits.

use crate::b64;
use crate::util::write_bytes;
use crate::wire::{signed_yaml_artifact, yss};
use yamlsigil_pinned_dir::PinnedDir;

/// Locally selected payload that satisfies the YAML envelope requirements.
const PAYLOAD: &[u8] = b"payload: example\n";

/// YAML spelling of algorithm slot `1`, as assigned by `README.md`.
const YAML_ALG: &str = "ED25519_PUREEDDSA_RAW_RS64_CANONICAL";

/// Protobuf enum number for the same slot, as assigned by
/// `proto/yaml_sigil/v1alpha1/yaml_sigil.proto`.
const PROTO_ALG: u64 = 1;

struct Case {
    stem: &'static str,
    signature_octets: &'static [u8],
    signature_text: &'static str,
}

const CASES: &[Case] = &[
    Case {
        stem: "empty",
        signature_octets: &[],
        signature_text: "",
    },
    Case {
        stem: "boolean-like-true",
        signature_octets: &[0xB6, 0xBB, 0x9E],
        signature_text: "true",
    },
    Case {
        stem: "null-like-null",
        signature_octets: &[0x9E, 0xE9, 0x65],
        signature_text: "null",
    },
    Case {
        stem: "numeric-looking-1234",
        signature_octets: &[0xD7, 0x6D, 0xF8],
        signature_text: "1234",
    },
];

fn yaml_artifact(case: &Case) -> Vec<u8> {
    format!(
        "payload: example\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: {YAML_ALG}\n\
         signature: \"{}\"\n",
        case.signature_text
    )
    .into_bytes()
}

fn protobuf_artifact(case: &Case) -> Vec<u8> {
    let signature = yss(PROTO_ALG, None, case.signature_octets);
    signed_yaml_artifact(PAYLOAD, &signature)
}

pub fn generate(dir: &PinnedDir) -> std::io::Result<()> {
    for case in CASES {
        assert_eq!(
            b64::urlsafe_unpadded(case.signature_octets),
            case.signature_text,
            "fixture octets must encode to the named YAML string"
        );

        write_bytes(dir, &format!("{}.yaml", case.stem), &yaml_artifact(case))?;
        write_bytes(
            dir,
            &format!("{}.binpb", case.stem),
            &protobuf_artifact(case),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn locally_selected_octets_encode_to_expected_strings() {
        for case in CASES {
            assert_eq!(
                b64::urlsafe_unpadded(case.signature_octets),
                case.signature_text
            );
        }
    }

    #[test]
    fn fixture_stems_are_unique() {
        let stems: HashSet<_> = CASES.iter().map(|case| case.stem).collect();
        assert_eq!(stems.len(), CASES.len());
    }

    #[test]
    fn paired_artifacts_share_exact_payload_bytes() {
        for case in CASES {
            assert!(yaml_artifact(case).starts_with(PAYLOAD));
            assert!(protobuf_artifact(case).starts_with(&crate::wire::lendel(1, PAYLOAD)));
        }
    }

    #[test]
    fn yaml_fixtures_present_signature_values_as_strings() {
        for case in CASES {
            let expected = format!("signature: \"{}\"\n", case.signature_text);
            assert!(yaml_artifact(case).ends_with(expected.as_bytes()));
        }
    }
}
