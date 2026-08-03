// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/yaml-decomposition/` fixtures.
//!
//! ## Constrained marker profile — `artifact-decomposition.md`
//!
//! The `YamlSigil` artifact is split by a single literal line that
//! begins at column zero and contains only the three octets `0x2D 0x2D
//! 0x2D` followed by `0x0A` (LF) or `0x0D 0x0A` (CRLF). The repo-level
//! `artifact-decomposition.md` states:
//!
//! > The marker MUST be the byte sequence `0x2D 0x2D 0x2D` (`---`)
//! > occurring at column zero of a line and immediately followed by
//! > either `0x0A` (LF) or `0x0D 0x0A` (CRLF). No other YAML
//! > document-end markers, indented `---`, or in-stream `---`
//! > strings are recognised by this profile. The split point `M` is
//! > the *last* marker in the artifact; everything before `M` is the
//! > payload, everything from `M` onward is the signature document.
//!
//! ## Encoding preconditions — `artifact-decomposition.md`
//!
//! The same document makes encoding checks the first step:
//!
//! > `A` MUST be valid UTF-8.
//! > `A` MUST NOT begin with any byte-order mark. The UTF-8 BOM octets
//! > `EF BB BF` at offset 0 are invalid.
//!
//! The fixtures below exercise that rule across the LF / CRLF variants,
//! encoding preconditions, the empty-payload edge (marker at offset 0),
//! the no-marker edge (which decomposes to `Unsigned`), and the "extra
//! marker inside the signature carrier" case (which validates the
//! *last* marker is the decomposition point). A payload-side YAML
//! document-end marker exercises the rule that `...` has no byte-layer
//! significance. The marker-dense fixture exercises the same selection rule
//! across many candidates. The implementation must retain only the latest
//! candidate rather than a collection of marker offsets.

use crate::b64::placeholder_sig;
use crate::util::write_bytes;
use yamlsigil_pinned_dir::PinnedDir;

pub fn generate(dir: &PinnedDir) -> std::io::Result<()> {
    let sig = placeholder_sig();
    assert_eq!(sig.len(), 86);

    // 1. signed-single-lf.yaml
    let f1 = format!(
        "some: random\n\
         yaml: document\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "signed-single-lf.yaml", f1.as_bytes())?;

    // 2. signed-single-crlf.yaml — CRLF throughout
    let f2 = format!(
        "some: random\r\n\
         yaml: document\r\n\
         ---\r\n\
         schema: YamlSigilSignature.v1alpha1\r\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\r\n\
         signature: {sig}\r\n"
    );
    write_bytes(dir, "signed-single-crlf.yaml", f2.as_bytes())?;

    // 3. signed-multi.yaml — two payload docs + final marker
    let f3 = format!(
        "some: random\n\
         yaml: document\n\
         ---\n\
         some: other-random\n\
         yaml: document\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "signed-multi.yaml", f3.as_bytes())?;

    // 4. empty-payload.yaml — marker at offset 0
    let f4 = format!(
        "---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "empty-payload.yaml", f4.as_bytes())?;

    // 5. no-marker.yaml — plain YAML, no `---`
    write_bytes(
        dir,
        "no-marker.yaml",
        b"some: random\nyaml: document\nmore: payload\n",
    )?;

    // 6. extra-marker-inside-carrier.yaml — tests last-marker selection
    let f6 = format!(
        "some: random\n\
         yaml: document\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n\
         ---\n\
         extra: trailer\n"
    );
    write_bytes(dir, "extra-marker-inside-carrier.yaml", f6.as_bytes())?;

    // 7. marker-at-eof-empty-body.yaml — marker present, no body
    write_bytes(
        dir,
        "marker-at-eof-empty-body.yaml",
        b"some: random\nyaml: document\n---\n",
    )?;

    // 8. invalid-utf8-no-marker.yaml — encoding precondition fails before scan
    write_bytes(
        dir,
        "invalid-utf8-no-marker.yaml",
        b"some: random\nyaml: \x80\n",
    )?;

    // 9. invalid-utf8-before-marker.yaml — invalid UTF-8 fails before marker selection
    let mut f9 = b"some: random\nyaml: ".to_vec();
    f9.push(0x80);
    f9.extend_from_slice(
        format!(
            "\n---\n\
             schema: YamlSigilSignature.v1alpha1\n\
             alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
             signature: {sig}\n"
        )
        .as_bytes(),
    );
    write_bytes(dir, "invalid-utf8-before-marker.yaml", &f9)?;

    // 10. bom-signed.yaml — BOM at offset 0 fails even when the rest is signed
    let mut f10 = b"\xEF\xBB\xBF".to_vec();
    f10.extend_from_slice(f1.as_bytes());
    write_bytes(dir, "bom-signed.yaml", &f10)?;

    // 11. bom-no-marker.yaml — BOM at offset 0 fails before no-marker handling
    write_bytes(
        dir,
        "bom-no-marker.yaml",
        b"\xEF\xBB\xBFsome: random\nyaml: document\n",
    )?;

    // 12. marker-dense.yaml — many earlier markers remain payload.
    //     Functional fixtures cannot measure allocation directly, but
    //     this input exercises last-marker selection without granting a
    //     reason to retain the earlier 256 candidate offsets.
    let mut f12 = String::new();
    for i in 0..=256 {
        f12.push_str(&format!("part: {i}\n---\n"));
    }
    f12.push_str(&format!(
        "schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    ));
    write_bytes(dir, "marker-dense.yaml", f12.as_bytes())?;

    // 13. document-end-in-payload.yaml — `...` remains payload bytes and
    //     does not compete with the final constrained signing marker.
    let f13 = format!(
        "some: random\n\
         ...\n\
         ---\n\
         schema: YamlSigilSignature.v1alpha1\n\
         alg: ED25519_PUREEDDSA_RAW_RS64_CANONICAL\n\
         signature: {sig}\n"
    );
    write_bytes(dir, "document-end-in-payload.yaml", f13.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    /// The constrained-marker profile fixes the marker bytes
    /// exactly. Pin them at the test level so any future drift in
    /// what the generator emits gets surfaced here too.
    #[test]
    fn marker_bytes_are_three_dashes_plus_line_terminator() {
        let lf_marker: &[u8] = b"---\n";
        let crlf_marker: &[u8] = b"---\r\n";
        assert_eq!(lf_marker, &[0x2D, 0x2D, 0x2D, 0x0A]);
        assert_eq!(crlf_marker, &[0x2D, 0x2D, 0x2D, 0x0D, 0x0A]);
    }

    #[test]
    fn precondition_fixture_bytes_are_invalid_as_yaml_artifacts() {
        let mut invalid_utf8 = b"some: ".to_vec();
        invalid_utf8.push(0x80);
        invalid_utf8.push(b'\n');
        assert!(std::str::from_utf8(&invalid_utf8).is_err());
        assert_eq!(&b"\xEF\xBB\xBF"[..], &[0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn marker_dense_shape_has_256_payload_candidates_and_one_final_marker() {
        let mut artifact = String::new();
        for i in 0..=256 {
            artifact.push_str(&format!("part: {i}\n---\n"));
        }
        artifact.push_str("schema: YamlSigilSignature.v1alpha1\n");
        assert_eq!(artifact.match_indices("\n---\n").count(), 257);
    }

    #[test]
    fn document_end_marker_is_not_a_constrained_marker() {
        let artifact = "some: random\n...\n---\nschema: value\n";
        assert!(artifact.contains("\n...\n"));
        assert_eq!(artifact.match_indices("\n---\n").count(), 1);
    }
}
