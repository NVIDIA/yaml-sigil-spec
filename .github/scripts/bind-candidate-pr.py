#!/usr/bin/env python3
"""Bind one open copied-ref pull request before candidate materialization."""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from typing import Any, Protocol


API_ROOT = "https://api.github.com"
API_VERSION = "2026-03-10"
MAX_RESPONSE_BYTES = 1024 * 1024


class BindingError(RuntimeError):
    """The anonymous pull-request binding failed closed."""


class Api(Protocol):
    """Minimal anonymous GitHub read boundary used by tests and production."""

    def get(self, path: str) -> Any:
        """Fetch and decode one JSON response."""


class AnonymousGitHubApi:
    """Bounded GitHub API client that never accepts or sends a credential."""

    def get(self, path: str) -> Any:
        if path.startswith("/") or ".." in path or any(c in path for c in "\r\n"):
            raise BindingError("GitHub API path is malformed")
        request = urllib.request.Request(
            f"{API_ROOT}/{path}",
            method="GET",
            headers={
                "Accept": "application/vnd.github+json",
                "User-Agent": "yaml-sigil-candidate-pr-binding/1",
                "X-GitHub-Api-Version": API_VERSION,
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                raw = response.read(MAX_RESPONSE_BYTES + 1)
        except urllib.error.HTTPError as error:
            raise BindingError(f"GitHub API read returned HTTP {error.code}") from error
        except urllib.error.URLError as error:
            raise BindingError("GitHub API read failed") from error
        if len(raw) > MAX_RESPONSE_BYTES:
            raise BindingError("GitHub API response is oversized")
        try:
            return json.loads(raw)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise BindingError("GitHub API returned invalid JSON") from error


def _mapping(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise BindingError(f"{label} is not an object")
    return value


def _text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or any(c in value for c in "\r\n"):
        raise BindingError(f"{label} is not one nonempty line")
    return value


def _sha(value: Any, label: str) -> str:
    value = _text(value, label)
    if re.fullmatch(r"[0-9a-f]{40}", value) is None:
        raise BindingError(f"{label} is not a lowercase full SHA")
    return value


def _repository(value: Any, label: str) -> str:
    return _text(_mapping(value, label).get("full_name"), f"{label} full name")


def bind_candidate_pr(
    api: Api,
    repository: str,
    copied_ref: str,
    head_sha: str,
    base_sha: str,
) -> None:
    """Require one open PR, copied ref, head, and protected-main snapshot."""

    if (
        re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is None
        or ".." in repository
    ):
        raise BindingError("repository is malformed")
    copied = re.fullmatch(r"pull-request/([1-9][0-9]*)", copied_ref)
    if copied is None:
        raise BindingError("copied ref is malformed")
    head_sha = _sha(head_sha, "expected head SHA")
    base_sha = _sha(base_sha, "expected base SHA")
    number = int(copied.group(1))

    prefix = f"repos/{repository}"
    pull = _mapping(api.get(f"{prefix}/pulls/{number}"), "pull request")
    base = _mapping(pull.get("base"), "pull request base")
    head = _mapping(pull.get("head"), "pull request head")
    if (
        pull.get("number") != number
        or pull.get("state") != "open"
        or _text(base.get("ref"), "pull request base ref") != "main"
        or _repository(base.get("repo"), "pull request base repository")
        != repository
        or _sha(base.get("sha"), "pull request base SHA") != base_sha
        or _sha(head.get("sha"), "pull request head SHA") != head_sha
    ):
        raise BindingError("pull request no longer binds the copied candidate")

    copied_readback = _mapping(
        api.get(f"{prefix}/git/ref/heads/{copied_ref}"), "copied ref"
    )
    copied_object = _mapping(copied_readback.get("object"), "copied ref object")
    if (
        copied_readback.get("ref") != f"refs/heads/{copied_ref}"
        or copied_object.get("type") != "commit"
        or _sha(copied_object.get("sha"), "copied ref SHA") != head_sha
    ):
        raise BindingError("copied ref no longer points to the reviewed head")

    main_readback = _mapping(api.get(f"{prefix}/git/ref/heads/main"), "main ref")
    main_object = _mapping(main_readback.get("object"), "main ref object")
    if (
        main_readback.get("ref") != "refs/heads/main"
        or main_object.get("type") != "commit"
        or _sha(main_object.get("sha"), "main ref SHA") != base_sha
    ):
        raise BindingError("main changed after protected policy was staged")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--repository", required=True)
    result.add_argument("--copied-ref", required=True)
    result.add_argument("--head-sha", required=True)
    result.add_argument("--base-sha", required=True)
    return result


def main() -> int:
    arguments = parser().parse_args()
    try:
        bind_candidate_pr(
            AnonymousGitHubApi(),
            arguments.repository,
            arguments.copied_ref,
            arguments.head_sha,
            arguments.base_sha,
        )
    except BindingError as error:
        print(f"candidate PR binding: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
