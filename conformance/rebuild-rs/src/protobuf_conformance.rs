// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Generator for `conformance/protobuf-conformance/` fixtures.
//!
//! ## Last-one-wins for scalar / singular submessage fields
//!
//! The [Protocol Buffers encoding spec](https://protobuf.dev/programming-guides/encoding/#optional)
//! defines the runtime rule for repeated occurrences of singular
//! fields:
//!
//! > Normally, an encoded message would never have more than one
//! > instance of a non-repeated field. However, parsers are expected
//! > to handle the case in which they do. For numeric types and
//! > strings, if the same field appears multiple times, the parser
//! > accepts the last value it sees.
//!
//! > For embedded message fields, the parser merges multiple instances
//! > of the same field, as if with the
//! > `Message::MergeFrom` method...
//!
//! ## Unknown field handling
//!
//! From the same spec, on unknown fields:
//!
//! > A new parser version may add a new field to a message. When the
//! > new parser parses a message produced by an older binary, those
//! > old binaries' fields will be unknown to the new parser. Older
//! > parsers will likewise see fields added by newer binaries as
//! > unknown. In either case, parsers MUST preserve unknown fields
//! > or surface them; behaviour is implementation-defined.
//!
//! `transcription-api.md` (the repo-level spec) interprets that
//! latitude with `OuterConformance` levels (`STRICT`, `PERMISSIVE`,
//! `SIGNATURE_STRICT`). The fixtures below pin the on-wire bytes that
//! drive each level's decision.

use std::path::Path;

use crate::util::write_bytes;
use crate::wire::{lendel, tag, varint, varint_field, yss};

pub fn generate(dir: &Path) -> std::io::Result<()> {
    let sig64 = [0u8; 64];
    let payload: &[u8] = b"hello: world\n";
    let inner = yss(1, None, &sig64);

    // 1. Sanity baseline
    let mut f1 = lendel(1, payload);
    f1.extend(lendel(2, &inner));
    write_bytes(dir, "valid-baseline.binpb", &f1)?;

    // 2. Duplicate outer payload
    let mut f2 = lendel(1, payload);
    f2.extend(lendel(1, b"second: payload\n"));
    f2.extend(lendel(2, &inner));
    write_bytes(dir, "duplicate-outer-payload.binpb", &f2)?;

    // 3. Duplicate outer signature submessage
    let inner_alt = yss(1, None, &[0xff; 64]);
    let mut f3 = lendel(1, payload);
    f3.extend(lendel(2, &inner));
    f3.extend(lendel(2, &inner_alt));
    write_bytes(dir, "duplicate-outer-signature.binpb", &f3)?;

    // 4. Unknown outer field at number 5 (varint)
    let mut unknown = tag(5, 0);
    unknown.extend(varint(42));
    let mut f4 = lendel(1, payload);
    f4.extend(lendel(2, &inner));
    f4.extend(unknown);
    write_bytes(dir, "unknown-outer-field.binpb", &f4)?;

    // 5. Inner duplicate alg varint
    let mut bad_inner = varint_field(1, 1);
    bad_inner.extend(varint_field(1, 2));
    bad_inner.extend(lendel(3, &sig64));
    let mut f5 = lendel(1, payload);
    f5.extend(lendel(2, &bad_inner));
    write_bytes(dir, "inner-strict-duplicate-alg.binpb", &f5)?;

    // 6. Present-empty outer signature submessage
    let mut f6 = lendel(1, payload);
    f6.extend(lendel(2, b""));
    write_bytes(dir, "present-empty-outer-signature.binpb", &f6)?;

    // 7. Binary payload not representable in YAML envelope. The
    //    payload is a single octet `0x72` (the RFC 8032 §7.1 Test 2
    //    message-byte choice, reused here as a compact non-YAML-
    //    envelope-fit example: one byte, no trailing line terminator).
    //    A protobuf-form verifier MUST accept this artifact (no YAML
    //    envelope rules apply); protobuf -> YAML transcoding MUST
    //    fail at the YAML Compose step with `InvalidPayloadBytes`.
    let binary_payload: &[u8] = b"\x72";
    let inner_binary = yss(1, None, &sig64);
    let mut f7 = lendel(1, binary_payload);
    f7.extend(lendel(2, &inner_binary));
    write_bytes(dir, "binary-payload-no-yaml-fit.binpb", &f7)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The baseline fixture's outer payload field starts with the
    /// known tag/length prefix for `field=1, wire=LEN, len=13`.
    #[test]
    fn baseline_starts_with_expected_tag_and_length() {
        let payload: &[u8] = b"hello: world\n";
        let prefix = lendel(1, payload);
        // 0x0A = (1 << 3) | 2
        // 0x0D = 13 (varint)
        assert_eq!(prefix[0], 0x0A);
        assert_eq!(prefix[1], 0x0D);
        assert_eq!(&prefix[2..], payload);
    }
}
