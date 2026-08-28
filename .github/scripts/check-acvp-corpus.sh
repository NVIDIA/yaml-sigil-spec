#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
# SPDX-License-Identifier: Apache-2.0

# Bound the candidate ACVP snapshot before Cargo can compile candidate Rust.
# This script only inspects the fixed corpus path beneath one checkout root;
# it does not parse or execute candidate-controlled source.

set -euo pipefail

if [[ "$#" -ne 1 ]]; then
  echo "usage: check-acvp-corpus.sh <candidate-root>" >&2
  exit 2
fi

readonly candidate_root="$1"
readonly max_bytes=3145728
readonly relative_path="conformance/rebuild-rs/vendor/acvp/ECDSA-SigGen-FIPS186-5.json"
readonly corpus_path="${candidate_root}/${relative_path}"

# Every parent must remain a real directory, not a path redirection.
current_path="${candidate_root}"
for component in conformance rebuild-rs vendor acvp; do
  current_path="${current_path}/${component}"
  if [[ ! -d "${current_path}" || -L "${current_path}" ]]; then
    echo "ACVP corpus parent is not a no-follow directory: ${current_path}" >&2
    exit 1
  fi
done

# The fixed final entry must be a regular file before its bytes are counted.
if [[ ! -f "${corpus_path}" || -L "${corpus_path}" ]]; then
  echo "ACVP corpus is not a no-follow regular file: ${relative_path}" >&2
  exit 1
fi

corpus_bytes="$(wc -c < "${corpus_path}")"
readonly corpus_bytes
if [[ ! "${corpus_bytes}" =~ ^[0-9]+$ || "${corpus_bytes}" -gt "${max_bytes}" ]]; then
  echo "ACVP corpus exceeds the ${max_bytes}-byte protected-CI limit" >&2
  exit 1
fi

printf 'ACVP corpus precompile bound: %s/%s bytes\n' \
  "${corpus_bytes}" "${max_bytes}"
