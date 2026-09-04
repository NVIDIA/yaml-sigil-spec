#!/usr/bin/env python3
"""Bind one copied-ref CI run before reporting the App-owned required check."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol


API_ROOT = "https://api.github.com"
API_VERSION = "2026-03-10"
MAX_EVENT_BYTES = 256 * 1024
MAX_RESPONSE_BYTES = 2 * 1024 * 1024
TERMINAL_JOB_CONCLUSIONS = {
    "action_required",
    "cancelled",
    "failure",
    "neutral",
    "skipped",
    "stale",
    "success",
    "timed_out",
}


class ReporterError(RuntimeError):
    """A fail-closed reporter validation error."""


class Api(Protocol):
    """Minimal GitHub API boundary used by production and fixture tests."""

    def get(self, path: str) -> Any:
        """Fetch and decode one JSON response."""

    def post(self, path: str, payload: dict[str, Any]) -> Any:
        """Create one GitHub object and decode its JSON response."""

class GitHubApi:
    """Bounded JSON client for api.github.com."""

    def __init__(self, token: str) -> None:
        if not token or "\n" in token or "\r" in token:
            raise ReporterError("GitHub API token is missing or malformed")
        self._token = token

    def get(self, path: str) -> Any:
        return self._request("GET", path, None)

    def post(self, path: str, payload: dict[str, Any]) -> Any:
        return self._request("POST", path, payload)

    def _request(
        self, method: str, path: str, payload: dict[str, Any] | None
    ) -> Any:
        if path.startswith("/") or ".." in path or any(c in path for c in "\r\n"):
            raise ReporterError("GitHub API path is malformed")
        body = None if payload is None else json.dumps(payload).encode("utf-8")
        request = urllib.request.Request(
            f"{API_ROOT}/{path}",
            data=body,
            method=method,
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {self._token}",
                "Content-Type": "application/json",
                "User-Agent": "yaml-sigil-required-ci-reporter/1",
                "X-GitHub-Api-Version": API_VERSION,
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                raw = response.read(MAX_RESPONSE_BYTES + 1)
        except urllib.error.HTTPError as error:
            raise ReporterError(
                f"GitHub API {method} {path} returned HTTP {error.code}"
            ) from error
        except urllib.error.URLError as error:
            raise ReporterError(f"GitHub API {method} {path} failed") from error
        if len(raw) > MAX_RESPONSE_BYTES:
            raise ReporterError(f"GitHub API {method} {path} response is oversized")
        try:
            return json.loads(raw)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ReporterError(
                f"GitHub API {method} {path} returned invalid JSON"
            ) from error


@dataclass(frozen=True)
class Policy:
    """Protected constants supplied by the default-branch workflow."""

    repository: str
    workflow_id: int
    workflow_path: str
    job_name: str
    check_name: str
    app_slug: str

    def validate(self) -> None:
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", self.repository):
            raise ReporterError("expected repository is malformed")
        if self.workflow_id <= 0:
            raise ReporterError("expected workflow ID is malformed")
        if not re.fullmatch(r"\.github/workflows/[A-Za-z0-9_.-]+\.ya?ml", self.workflow_path):
            raise ReporterError("expected workflow path is malformed")
        for label, value in (
            ("job name", self.job_name),
            ("check name", self.check_name),
            ("App slug", self.app_slug),
        ):
            if not value or len(value) > 128 or any(c in value for c in "\r\n"):
                raise ReporterError(f"expected {label} is malformed")


@dataclass(frozen=True)
class Binding:
    """Freshly verified candidate run state."""

    run_id: int
    run_attempt: int
    pull_number: int
    head_branch: str
    head_sha: str
    conclusion: str
    details_url: str

    @property
    def check_conclusion(self) -> str:
        return "success" if self.conclusion == "success" else "failure"

    @property
    def external_id(self) -> str:
        return f"yaml-sigil-required-ci:{self.run_id}:{self.run_attempt}"


def _mapping(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ReporterError(f"{label} is not an object")
    return value


def _sequence(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list):
        raise ReporterError(f"{label} is not an array")
    return value


def _integer(value: Any, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ReporterError(f"{label} is not a positive integer")
    return value


def _text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or any(c in value for c in "\r\n"):
        raise ReporterError(f"{label} is not one nonempty line")
    return value


def _sha(value: Any, label: str) -> str:
    value = _text(value, label)
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise ReporterError(f"{label} is not a lowercase full SHA")
    return value


def _repository_name(value: Any, label: str) -> str:
    return _text(_mapping(value, label).get("full_name"), f"{label} full name")


def read_event(path: Path) -> dict[str, Any]:
    """Read one bounded GitHub event document."""

    try:
        metadata = path.stat()
    except OSError as error:
        raise ReporterError("cannot inspect the workflow event") from error
    if not path.is_file() or metadata.st_size > MAX_EVENT_BYTES:
        raise ReporterError("workflow event is missing or oversized")
    try:
        value = json.loads(path.read_bytes())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ReporterError("workflow event is not valid JSON") from error
    return _mapping(value, "workflow event")


def bind_candidate(api: Api, event: dict[str, Any], policy: Policy) -> Binding:
    """Bind the triggering delivery to one fresh copied-ref CI attempt."""

    policy.validate()
    if event.get("action") != "completed":
        raise ReporterError("workflow event action is not completed")
    if _repository_name(event.get("repository"), "event repository") != policy.repository:
        raise ReporterError("workflow event repository is unexpected")

    delivered = _mapping(event.get("workflow_run"), "delivered workflow run")
    run_id = _integer(delivered.get("id"), "delivered run ID")
    run_attempt = _integer(delivered.get("run_attempt"), "delivered run attempt")
    if _integer(delivered.get("workflow_id"), "delivered workflow ID") != policy.workflow_id:
        raise ReporterError("delivered workflow ID is unexpected")

    repository_path = f"repos/{policy.repository}"
    run = _mapping(api.get(f"{repository_path}/actions/runs/{run_id}"), "workflow run")
    if _integer(run.get("id"), "run ID") != run_id:
        raise ReporterError("fetched run ID does not match the delivery")
    if _integer(run.get("run_attempt"), "run attempt") != run_attempt:
        raise ReporterError("the delivered run attempt is stale")
    if _integer(run.get("workflow_id"), "workflow ID") != policy.workflow_id:
        raise ReporterError("workflow ID is unexpected")
    if _text(run.get("path"), "workflow path") != policy.workflow_path:
        raise ReporterError("workflow path is unexpected")
    if _text(run.get("event"), "workflow event") != "push":
        raise ReporterError("candidate workflow was not triggered by a push")
    if _text(run.get("status"), "workflow status") != "completed":
        raise ReporterError("candidate workflow is not completed")
    if _repository_name(run.get("repository"), "run repository") != policy.repository:
        raise ReporterError("workflow run repository is unexpected")

    head_branch = _text(run.get("head_branch"), "run head branch")
    branch_match = re.fullmatch(r"pull-request/([1-9][0-9]*)", head_branch)
    if branch_match is None:
        raise ReporterError("workflow run is not an exact copied pull-request ref")
    pull_number = int(branch_match.group(1))
    head_sha = _sha(run.get("head_sha"), "run head SHA")
    details_url = _text(run.get("html_url"), "run URL")

    for field, expected, label in (
        ("id", run_id, "delivered run ID"),
        ("run_attempt", run_attempt, "delivered run attempt"),
        ("workflow_id", policy.workflow_id, "delivered workflow ID"),
        ("head_branch", head_branch, "delivered head branch"),
        ("head_sha", head_sha, "delivered head SHA"),
    ):
        if delivered.get(field) != expected:
            raise ReporterError(f"{label} does not match fetched state")
    if _repository_name(delivered.get("repository"), "delivered repository") != policy.repository:
        raise ReporterError("delivered run repository is unexpected")

    pull = _mapping(api.get(f"{repository_path}/pulls/{pull_number}"), "pull request")
    if _integer(pull.get("number"), "pull request number") != pull_number:
        raise ReporterError("pull request number is unexpected")
    if pull.get("state") != "open":
        raise ReporterError("pull request is not open")
    pull_base = _mapping(pull.get("base"), "pull request base")
    if pull_base.get("ref") != "main":
        raise ReporterError("pull request base branch is not main")
    if _repository_name(pull_base.get("repo"), "base repository") != policy.repository:
        raise ReporterError("pull request base repository is unexpected")
    pull_head = _mapping(pull.get("head"), "pull request head")
    if _sha(pull_head.get("sha"), "current pull request head SHA") != head_sha:
        raise ReporterError("pull request head moved after candidate execution")

    encoded_ref = urllib.parse.quote(f"heads/{head_branch}", safe="/")
    copied_ref = _mapping(
        api.get(f"{repository_path}/git/ref/{encoded_ref}"), "copied Git ref"
    )
    if copied_ref.get("ref") != f"refs/heads/{head_branch}":
        raise ReporterError("copied Git ref name is unexpected")
    copied_object = _mapping(copied_ref.get("object"), "copied Git object")
    if copied_object.get("type") != "commit" or _sha(
        copied_object.get("sha"), "copied Git ref SHA"
    ) != head_sha:
        raise ReporterError("copied Git ref does not equal the current pull request head")

    jobs = _mapping(
        api.get(
            f"{repository_path}/actions/runs/{run_id}/attempts/{run_attempt}/jobs?per_page=100"
        ),
        "job inventory",
    )
    job_items = _sequence(jobs.get("jobs"), "job inventory jobs")
    if jobs.get("total_count") != len(job_items) or len(job_items) > 100:
        raise ReporterError("job inventory is incomplete or oversized")
    authoritative = [job for job in job_items if isinstance(job, dict) and job.get("name") == policy.job_name]
    if len(authoritative) != 1:
        raise ReporterError("authoritative Linux job is missing or duplicated")
    job = authoritative[0]
    if _integer(job.get("run_id"), "job run ID") != run_id:
        raise ReporterError("authoritative job belongs to another run")
    if _integer(job.get("run_attempt"), "job run attempt") != run_attempt:
        raise ReporterError("authoritative job belongs to another attempt")
    if _sha(job.get("head_sha"), "job head SHA") != head_sha:
        raise ReporterError("authoritative job ran another head")
    if job.get("status") != "completed":
        raise ReporterError("authoritative job is not completed")
    conclusion = job.get("conclusion")
    if conclusion not in TERMINAL_JOB_CONCLUSIONS:
        raise ReporterError("authoritative job conclusion is not a bounded terminal result")

    artifacts = _mapping(
        api.get(f"{repository_path}/actions/runs/{run_id}/artifacts?per_page=1"),
        "artifact inventory",
    )
    artifact_items = _sequence(artifacts.get("artifacts"), "artifact inventory artifacts")
    if artifacts.get("total_count") != 0 or artifact_items:
        raise ReporterError("candidate workflow retained an artifact")

    return Binding(
        run_id=run_id,
        run_attempt=run_attempt,
        pull_number=pull_number,
        head_branch=head_branch,
        head_sha=head_sha,
        conclusion=conclusion,
        details_url=details_url,
    )


def verify_app_scope(api: Api, policy: Policy, observed_app_slug: str) -> None:
    """Require the token action's App identity and exact repository scope."""

    if observed_app_slug != policy.app_slug:
        raise ReporterError("reporting token belongs to an unexpected App")
    installation = _mapping(
        api.get("installation/repositories?per_page=100"),
        "installation repository inventory",
    )
    repositories = _sequence(
        installation.get("repositories"), "installation repositories"
    )
    names = [
        repository.get("full_name")
        for repository in repositories
        if isinstance(repository, dict)
    ]
    if installation.get("total_count") != 1 or names != [policy.repository]:
        raise ReporterError("reporting token is not scoped to exactly this repository")


def _validate_check(
    value: Any, policy: Policy, binding: Binding, expected_id: int | None = None
) -> dict[str, Any]:
    check = _mapping(value, "required check")
    check_id = _integer(check.get("id"), "required check ID")
    if expected_id is not None and check_id != expected_id:
        raise ReporterError("required check ID changed during readback")
    if (
        check.get("name") != policy.check_name
        or check.get("head_sha") != binding.head_sha
        or check.get("external_id") != binding.external_id
        or check.get("status") != "completed"
        or check.get("conclusion") != binding.check_conclusion
        or _mapping(check.get("app"), "required check App").get("slug")
        != policy.app_slug
    ):
        raise ReporterError("App-owned required check readback is not exact")
    return check


def report_check(api: Api, policy: Policy, binding: Binding) -> int:
    """Create one completed check, or accept its exact idempotent readback."""

    repository_path = f"repos/{policy.repository}"
    query = urllib.parse.urlencode(
        {"check_name": policy.check_name, "filter": "all", "per_page": "100"}
    )
    inventory = _mapping(
        api.get(f"{repository_path}/commits/{binding.head_sha}/check-runs?{query}"),
        "required check inventory",
    )
    checks = _sequence(inventory.get("check_runs"), "required checks")
    if inventory.get("total_count") != len(checks) or len(checks) > 100:
        raise ReporterError("required check inventory is incomplete or oversized")
    matches = [
        check
        for check in checks
        if isinstance(check, dict)
        and check.get("external_id") == binding.external_id
        and isinstance(check.get("app"), dict)
        and check["app"].get("slug") == policy.app_slug
    ]
    if len(matches) > 1:
        raise ReporterError("duplicate App-owned checks exist for this run attempt")
    if matches:
        return _integer(
            _validate_check(matches[0], policy, binding).get("id"),
            "required check ID",
        )

    payload = {
        "name": policy.check_name,
        "head_sha": binding.head_sha,
        "status": "completed",
        "conclusion": binding.check_conclusion,
        "external_id": binding.external_id,
        "details_url": binding.details_url,
        "output": {
            "title": "Authorized candidate CI result",
            "summary": (
                "The exact copied pull-request head completed the authoritative "
                f"Linux job with conclusion `{binding.conclusion}`."
            ),
        },
    }
    created = _validate_check(
        api.post(f"{repository_path}/check-runs", payload), policy, binding
    )
    check_id = _integer(created.get("id"), "required check ID")
    readback = api.get(f"{repository_path}/check-runs/{check_id}")
    _validate_check(readback, policy, binding, check_id)
    return check_id


def append_outputs(binding: Binding) -> None:
    """Append bounded scalar outputs to GitHub's runner-owned output file."""

    output_path = os.environ.get("GITHUB_OUTPUT")
    if not output_path:
        raise ReporterError("GITHUB_OUTPUT is not set")
    with open(output_path, "a", encoding="utf-8", newline="\n") as output:
        output.write(f"head_sha={binding.head_sha}\n")
        output.write(f"conclusion={binding.conclusion}\n")
        output.write(f"run_id={binding.run_id}\n")
        output.write(f"run_attempt={binding.run_attempt}\n")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("operation", choices=("inspect", "report"))
    result.add_argument("--event", required=True, type=Path)
    result.add_argument("--repository", required=True)
    result.add_argument("--workflow-id", required=True, type=int)
    result.add_argument("--workflow-path", required=True)
    result.add_argument("--job-name", required=True)
    result.add_argument("--check-name", required=True)
    result.add_argument("--app-slug", required=True)
    return result


def run(arguments: argparse.Namespace) -> None:
    policy = Policy(
        repository=arguments.repository,
        workflow_id=arguments.workflow_id,
        workflow_path=arguments.workflow_path,
        job_name=arguments.job_name,
        check_name=arguments.check_name,
        app_slug=arguments.app_slug,
    )
    event = read_event(arguments.event)
    read_api = GitHubApi(os.environ.get("GITHUB_TOKEN", ""))
    binding = bind_candidate(read_api, event, policy)
    if arguments.operation == "inspect":
        append_outputs(binding)
        return

    app_api = GitHubApi(os.environ.get("APP_TOKEN", ""))
    verify_app_scope(app_api, policy, os.environ.get("APP_SLUG", ""))
    # Repeat every mutable binding after the App token exists and immediately
    # before the only write. A moved head, rerun, ref, or artifact fails closed.
    binding = bind_candidate(read_api, event, policy)
    check_id = report_check(app_api, policy, binding)
    print(f"reported {policy.check_name} check {check_id} for {binding.head_sha}")


def main() -> int:
    try:
        run(parser().parse_args())
    except ReporterError as error:
        print(f"required CI reporter: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
