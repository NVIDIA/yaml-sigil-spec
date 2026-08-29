// SPDX-FileCopyrightText: Copyright 2026 NVIDIA CORPORATION & AFFILIATES
// SPDX-License-Identifier: Apache-2.0

//! Developer-only tasks for the `rebuild-rs` workspace.
//!
//! Invoked via the `cargo xtask ...` alias defined in
//! `.cargo/config.toml`. Provides:
//!
//! ```text
//! cargo xtask ci
//! cargo xtask update-acvp [--commit <40-character-lowercase-commit>]
//! ```
//!
//! which refreshes the vendored NIST ACVP-Server ECDSA SigGen test
//! vectors through a size- and time-bounded HTTPS client.

mod ci;

use std::env;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;
use ureq::config::{Config, RedirectAuthHeaders};
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::{Agent, Error as UreqError, Proxy, ProxyProtocol};
use yamlsigil_pinned_dir::PinnedDir;

/// Pinned upstream commit of `usnistgov/ACVP-Server`. Bump this
/// constant (or pass `--commit <hash>`) when refreshing the vendored
/// vectors; the xtask rewrites both the JSON file and the vendor
/// README so the pin is always self-describing.
const DEFAULT_COMMIT: &str = "15c0f3deeefbfa8cb6cd32a99e1ca3b738c66bf0";

const UPSTREAM_REPO: &str = "usnistgov/ACVP-Server";
const UPSTREAM_PATH: &str = "gen-val/json-files/ECDSA-SigGen-FIPS186-5/internalProjection.json";
const VENDORED_FILE_NAME: &str = "ECDSA-SigGen-FIPS186-5.json";
const MAX_ACVP_CORPUS_BYTES: usize = 3 * 1024 * 1024;
const MAX_ACVP_RESPONSE_HEADER_BYTES: usize = 32 * 1024;
const MAX_ACVP_REDIRECTS: u32 = 5;
const ACVP_USER_AGENT: &str = concat!("yaml-sigil-spec-xtask/", env!("CARGO_PKG_VERSION"));
const HTTP_GLOBAL_TIMEOUT: Duration = Duration::from_secs(60);
const HTTP_RESOLVE_TIMEOUT: Duration = Duration::from_secs(10);
const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const HTTP_SEND_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const HTTP_RECV_RESPONSE_TIMEOUT: Duration = Duration::from_secs(20);
const HTTP_RECV_BODY_TIMEOUT: Duration = Duration::from_secs(45);
const PROXY_ENV_VARS: &[&str] = &[
    "ALL_PROXY",
    "all_proxy",
    "HTTPS_PROXY",
    "https_proxy",
    "HTTP_PROXY",
    "http_proxy",
];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "ci" => {
            let remaining: Vec<_> = args.collect();
            if is_help_request(&remaining) {
                print_usage();
                ExitCode::SUCCESS
            } else {
                match parse_ci_root(&remaining)
                    .and_then(|root| ci::run(&root).map_err(|error| error.to_string()))
                {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("ci failed: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
        }
        "update-acvp" => {
            let remaining: Vec<_> = args.collect();
            if is_help_request(&remaining) {
                print_usage();
                return ExitCode::SUCCESS;
            }
            match parse_update_args(remaining.into_iter()) {
                Ok(commit) => match update_acvp(&commit) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("update-acvp failed: {e}");
                        ExitCode::FAILURE
                    }
                },
                Err(e) => {
                    eprintln!("{e}");
                    print_usage();
                    ExitCode::FAILURE
                }
            }
        }
        "" | "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage:\n  \
         cargo xtask ci [--candidate-root PATH]\n  \
         cargo xtask update-acvp [--commit <40-character-lowercase-commit>]\n\n\
         Runs the repository's complete non-release validation sequence.\n\
         --candidate-root validates another repository checkout with this\n\
         protected xtask implementation.\n\n\
         Refreshes vendor/acvp/{VENDORED_FILE_NAME} and the matching\n\
         vendor/acvp/README.md to track the requested commit of\n\
         https://github.com/{UPSTREAM_REPO}."
    );
}

fn is_help_request(args: &[String]) -> bool {
    matches!(args, [arg] if matches!(arg.as_str(), "help" | "--help" | "-h"))
}

fn parse_update_args<I: Iterator<Item = String>>(mut args: I) -> Result<String, String> {
    let mut explicit_commit = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--commit" => {
                if explicit_commit.is_some() {
                    return Err("--commit may be supplied only once".to_string());
                }
                explicit_commit = Some(
                    args.next()
                        .ok_or_else(|| "--commit needs a hash argument".to_string())?,
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let commit = explicit_commit.unwrap_or_else(|| DEFAULT_COMMIT.to_string());
    validate_commit(&commit)?;
    Ok(commit)
}

fn validate_commit(commit: &str) -> Result<(), String> {
    let valid = commit.len() == 40
        && commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err("commit hash must be exactly 40 lowercase hexadecimal characters".into());
    }
    Ok(())
}

fn acvp_url(commit: &str) -> String {
    format!("https://raw.githubusercontent.com/{UPSTREAM_REPO}/{commit}/{UPSTREAM_PATH}")
}

fn update_acvp(commit: &str) -> io::Result<()> {
    validate_commit(commit)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?;
    let root = workspace_root().canonicalize()?;
    let url = acvp_url(commit);
    let dest = root.join("vendor/acvp").join(VENDORED_FILE_NAME);
    let readme_path = root.join("vendor/acvp/README.md");

    println!("fetching: {url}");
    println!("dest:     {}", dest.display());

    let mut client = UreqAcvpClient::from_environment()?;
    let bytes = download_acvp(&mut client, &url)?;

    let size = u64::try_from(bytes.len())
        .map_err(|_| io::Error::other("download size does not fit u64"))?;

    // Do not pin or replace a destination until the complete response has
    // passed the HTTP and byte-size checks above.
    let root_dir = PinnedDir::open(&root)?;
    let vendor_root = root_dir.ensure_child("vendor")?;
    let vendor_dir = vendor_root.ensure_child("acvp")?;
    vendor_dir.replace_regular_file(VENDORED_FILE_NAME, &bytes)?;

    let readme = render_readme(commit, size);
    vendor_dir.replace_regular_file("README.md", readme.as_bytes())?;

    println!("wrote: {} ({size} bytes)", dest.display());
    println!("wrote: {}", readme_path.display());
    Ok(())
}

struct AcvpHttpResponse {
    status: u16,
    content_length: Option<u64>,
    content_encodings: Vec<String>,
    body: Box<dyn Read>,
}

trait AcvpHttpClient {
    fn get(&mut self, url: &str) -> io::Result<AcvpHttpResponse>;
}

struct UreqAcvpClient {
    agent: Agent,
}

impl UreqAcvpClient {
    fn from_environment() -> io::Result<Self> {
        let proxy = proxy_from_environment()?;
        Ok(Self {
            agent: acvp_http_config(proxy).new_agent(),
        })
    }
}

impl AcvpHttpClient for UreqAcvpClient {
    fn get(&mut self, url: &str) -> io::Result<AcvpHttpResponse> {
        let response = self.agent.get(url).call().map_err(map_ureq_request_error)?;
        let status = response.status().as_u16();
        let content_encodings = response
            .headers()
            .get_all(ureq::http::header::CONTENT_ENCODING)
            .iter()
            .map(|value| {
                value
                    .to_str()
                    .map(str::to_owned)
                    .map_err(|_| invalid_response("ACVP response has an invalid content encoding"))
            })
            .collect::<io::Result<Vec<_>>>()?;
        let content_length = response.body().content_length();
        let body = response.into_body().into_reader();

        Ok(AcvpHttpResponse {
            status,
            content_length,
            content_encodings,
            body: Box::new(body),
        })
    }
}

fn acvp_http_config(proxy: Option<Proxy>) -> Config {
    Agent::config_builder()
        .http_status_as_error(false)
        .https_only(true)
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::Rustls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .proxy(proxy)
        .max_redirects(MAX_ACVP_REDIRECTS)
        .max_redirects_will_error(true)
        .redirect_auth_headers(RedirectAuthHeaders::Never)
        .save_redirect_history(false)
        .user_agent(ACVP_USER_AGENT)
        .accept("application/json")
        .accept_encoding("identity")
        .timeout_global(Some(HTTP_GLOBAL_TIMEOUT))
        .timeout_resolve(Some(HTTP_RESOLVE_TIMEOUT))
        .timeout_connect(Some(HTTP_CONNECT_TIMEOUT))
        .timeout_send_request(Some(HTTP_SEND_REQUEST_TIMEOUT))
        .timeout_recv_response(Some(HTTP_RECV_RESPONSE_TIMEOUT))
        .timeout_recv_body(Some(HTTP_RECV_BODY_TIMEOUT))
        .max_response_header_size(MAX_ACVP_RESPONSE_HEADER_BYTES)
        .build()
}

fn proxy_from_environment() -> io::Result<Option<Proxy>> {
    let configured = PROXY_ENV_VARS
        .iter()
        .find_map(|name| env::var_os(name).filter(|value| !value.is_empty()));
    let Some(configured) = configured else {
        return Ok(None);
    };

    // `try_from_env` deliberately skips malformed higher-precedence values.
    // Validate that value first so a bad proxy setting cannot silently fall
    // through to a lower-precedence proxy or a direct connection.
    parse_supported_proxy(&configured)?;
    validate_proxy(Proxy::try_from_env(), true)
}

fn parse_supported_proxy(value: &std::ffi::OsStr) -> io::Result<Proxy> {
    let value = value.to_str().ok_or_else(proxy_configuration_error)?;
    let proxy = Proxy::new(value).map_err(|_| proxy_configuration_error())?;
    validate_proxy(Some(proxy), true)?.ok_or_else(proxy_configuration_error)
}

fn proxy_configuration_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "proxy environment is invalid or uses an unsupported protocol",
    )
}

fn validate_proxy(proxy: Option<Proxy>, configured: bool) -> io::Result<Option<Proxy>> {
    let Some(proxy) = proxy else {
        if configured {
            return Err(proxy_configuration_error());
        }
        return Ok(None);
    };

    if !matches!(proxy.protocol(), ProxyProtocol::Http | ProxyProtocol::Https) {
        return Err(proxy_configuration_error());
    }

    Ok(Some(proxy))
}

fn download_acvp(client: &mut impl AcvpHttpClient, url: &str) -> io::Result<Vec<u8>> {
    let mut response = client.get(url)?;
    if response.status != 200 {
        return Err(invalid_response(format!(
            "ACVP download returned HTTP status {}; expected 200",
            response.status
        )));
    }

    let has_identity_encoding = match response.content_encodings.as_slice() {
        [] => true,
        [encoding] => encoding.eq_ignore_ascii_case("identity"),
        _ => false,
    };
    if !has_identity_encoding {
        return Err(invalid_response(
            "ACVP response uses an unsupported content encoding",
        ));
    }

    if response
        .content_length
        .is_some_and(|length| length > MAX_ACVP_CORPUS_BYTES as u64)
    {
        return Err(acvp_body_limit_error());
    }

    read_bounded_acvp_body(response.body.as_mut())
}

fn read_bounded_acvp_body(body: &mut dyn Read) -> io::Result<Vec<u8>> {
    let sentinel_limit = MAX_ACVP_CORPUS_BYTES + 1;
    let mut bytes = Vec::with_capacity(sentinel_limit);
    let mut chunk = [0_u8; 16 * 1024];

    loop {
        let remaining = sentinel_limit - bytes.len();
        if remaining == 0 {
            return Err(acvp_body_limit_error());
        }
        let to_read = remaining.min(chunk.len());
        let read = body.read(&mut chunk[..to_read]).map_err(|error| {
            io::Error::new(error.kind(), "failed to read ACVP HTTPS response body")
        })?;
        if read == 0 {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > MAX_ACVP_CORPUS_BYTES {
            return Err(acvp_body_limit_error());
        }
    }
}

fn acvp_body_limit_error() -> io::Error {
    invalid_response(format!(
        "download exceeds the {MAX_ACVP_CORPUS_BYTES}-byte ACVP corpus limit"
    ))
}

fn invalid_response(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn map_ureq_request_error(error: UreqError) -> io::Error {
    match error {
        UreqError::StatusCode(status) => invalid_response(format!(
            "ACVP download returned HTTP status {status}; expected 200"
        )),
        UreqError::Timeout(phase) => io::Error::new(
            io::ErrorKind::TimedOut,
            format!("ACVP HTTPS request timed out during {phase}"),
        ),
        UreqError::HostNotFound => {
            io::Error::new(io::ErrorKind::NotFound, "ACVP HTTPS host was not found")
        }
        UreqError::TooManyRedirects => invalid_response(format!(
            "ACVP HTTPS request exceeded {MAX_ACVP_REDIRECTS} redirects"
        )),
        UreqError::RequireHttpsOnly(_) => {
            invalid_response("ACVP HTTPS request refused a non-HTTPS redirect")
        }
        UreqError::InvalidProxyUrl | UreqError::ConnectProxyFailed(_) => {
            io::Error::other("ACVP HTTPS proxy connection failed")
        }
        UreqError::Tls(_) | UreqError::Rustls(_) => {
            io::Error::other("ACVP HTTPS certificate validation failed")
        }
        UreqError::Io(error) => io::Error::new(error.kind(), "ACVP HTTPS transport failed"),
        _ => io::Error::other("ACVP HTTPS request failed"),
    }
}

/// Compute the workspace root by walking up from the xtask crate's
/// `CARGO_MANIFEST_DIR`. Falls back to the current working directory
/// if the env var is unset (which shouldn't happen under `cargo run`).
fn workspace_root() -> PathBuf {
    if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest = PathBuf::from(dir);
        if let Some(parent) = manifest.parent() {
            return parent.to_path_buf();
        }
    }
    env::current_dir().expect("cwd available")
}

fn repository_root() -> PathBuf {
    workspace_root()
        .parent()
        .and_then(std::path::Path::parent)
        .expect("rebuild-rs is nested under conformance/")
        .to_path_buf()
}

fn parse_ci_root(args: &[String]) -> Result<PathBuf, String> {
    let candidate = match args {
        [] => repository_root(),
        [flag, value] if flag == "--candidate-root" && !value.is_empty() => PathBuf::from(value),
        [flag, _] if flag == "--candidate-root" => {
            return Err("--candidate-root needs a nonempty path".to_string());
        }
        _ => return Err("ci accepts only --candidate-root PATH".to_string()),
    };
    let candidate = candidate.canonicalize().map_err(|error| {
        format!(
            "cannot resolve candidate root {}: {error}",
            candidate.display()
        )
    })?;
    let rebuild_root = candidate.join("conformance/rebuild-rs");
    if !rebuild_root.join("Cargo.toml").is_file() {
        return Err(format!(
            "candidate root {} lacks conformance/rebuild-rs/Cargo.toml",
            candidate.display()
        ));
    }
    Ok(rebuild_root)
}

fn render_readme(commit: &str, size: u64) -> String {
    let raw_url =
        format!("https://raw.githubusercontent.com/{UPSTREAM_REPO}/{commit}/{UPSTREAM_PATH}");
    let tree_url = format!("https://github.com/{UPSTREAM_REPO}/blob/{commit}/{UPSTREAM_PATH}");
    format!(
        "# Vendored ACVP test vectors\n\
         \n\
         This directory tracks a single file from the NIST\n\
         [Automated Cryptographic Validation Protocol (ACVP)](https://pages.nist.gov/ACVP/)\n\
         server's reference test-vector tree.\n\
         \n\
         | Field | Value |\n\
         | --- | --- |\n\
         | Upstream repo | <https://github.com/{UPSTREAM_REPO}> |\n\
         | Commit | `{commit}` |\n\
         | Upstream path | `{UPSTREAM_PATH}` |\n\
         | Browse on GitHub | <{tree_url}> |\n\
         | Vendored as | `vendor/acvp/{VENDORED_FILE_NAME}` ({size} bytes) |\n\
         | Pinned by | `xtask/src/main.rs` `DEFAULT_COMMIT` |\n\
         \n\
         ## What this is\n\
         \n\
         A NIST ACVP \"AFT\" (Algorithm Functional Test) vector set\n\
         for ECDSA signature generation under FIPS 186-5. Each test\n\
         group pins `(d, Q, k, message, r, s)` — i.e. the private key,\n\
         public key, ephemeral nonce, message, and expected signature\n\
         components — so the rebuilder can replay the sign and assert\n\
         byte-equality against the published `(r, s)`. The file\n\
         covers multiple curve / hash combinations; the rebuilder\n\
         filters for `curve = P-256` and `hashAlg = SHA2-256`.\n\
         \n\
         ## Resource limits\n\
         \n\
         Protected CI and the native rebuilder accept at most 3 MiB of\n\
         encoded JSON, 512 test groups, 64 cases per group, and 4,096\n\
         cases in total. The selected P-256 / SHA2-256 replay is further\n\
         limited to eight groups and 256 cases. Scalar-like hex fields are\n\
         capped at 160 characters, messages at 4,096 characters, and\n\
         randomized-hashing values at 256 characters. The rebuilder reads\n\
         one anchored no-follow byte snapshot, validates these limits before\n\
         retaining collections, and deserializes that same snapshot before\n\
         replay. A refresh outside these limits requires an explicit review\n\
         and coordinated limit change. `cargo xtask ci` exercises every\n\
         exact-boundary and limit-plus-one regression.\n\
         \n\
         The National Institute of Standards and Technology is explicitly\n\
         acknowledged as the source of this test data. The local file name\n\
         was changed; its contents were not modified. The NIST notice that\n\
         governs this snapshot is reproduced in\n\
         [`THIRD_PARTY_NOTICES.md`](../../../../THIRD_PARTY_NOTICES.md) and\n\
         must remain with distributions of this vendored file.\n\
         \n\
         ## Manual verification\n\
         \n\
         To confirm the vendored bytes by hand, fetch the same file at\n\
         the pinned commit and compare SHA-256 hashes:\n\
         \n\
         ```sh\n\
         # Compute the hash of the upstream file at the pinned commit.\n\
         curl -sL '{raw_url}' | sha256sum\n\
         \n\
         # Compare against the hash of the vendored copy.\n\
         sha256sum vendor/acvp/{VENDORED_FILE_NAME}\n\
         ```\n\
         \n\
         The two outputs MUST match. If they don't, the vendored file\n\
         has drifted from its upstream pin and that diff is itself a\n\
         finding to surface.\n\
         \n\
         ## Refreshing\n\
         \n\
         To bump the pin to a newer commit, edit `DEFAULT_COMMIT` in\n\
         `xtask/src/main.rs` and run:\n\
         \n\
         ```sh\n\
         cargo xtask update-acvp\n\
         ```\n\
         \n\
         Or pass an explicit full 40-character lowercase hexadecimal commit\n\
         hash (without bumping the default):\n\
         \n\
         ```sh\n\
         cargo xtask update-acvp --commit <40-character-lowercase-commit>\n\
         ```\n\
         \n\
         The updater accepts only an HTTP 200 response over HTTPS, follows at\n\
         most five HTTPS redirects, and uses the platform certificate verifier.\n\
         It honors supported HTTP and HTTPS proxy and `NO_PROXY` environment\n\
         settings, does not retry, requests identity encoding, and bounds\n\
         response headers, timeouts, and the 3 MiB response body before\n\
         replacing either pinned file.\n\
         \n\
         The xtask rewrites both the JSON file and this README. This\n\
         file is regenerated on every run; do not edit it by hand.\n",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);
    const VALID_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    struct FakeHttpClient {
        response: Option<io::Result<AcvpHttpResponse>>,
        requested_url: Option<String>,
    }

    impl FakeHttpClient {
        fn returning(response: AcvpHttpResponse) -> Self {
            Self {
                response: Some(Ok(response)),
                requested_url: None,
            }
        }
    }

    impl AcvpHttpClient for FakeHttpClient {
        fn get(&mut self, url: &str) -> io::Result<AcvpHttpResponse> {
            self.requested_url = Some(url.to_string());
            self.response.take().expect("one fake response")
        }
    }

    struct PanicReader;

    impl io::Read for PanicReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            panic!("response body must not be read")
        }
    }

    struct PrefixThenError {
        sent_prefix: bool,
    }

    impl io::Read for PrefixThenError {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.sent_prefix {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "sensitive transport detail",
                ));
            }
            self.sent_prefix = true;
            let prefix = b"partial";
            buffer[..prefix.len()].copy_from_slice(prefix);
            Ok(prefix.len())
        }
    }

    fn response_with_body(status: u16, body: Vec<u8>) -> AcvpHttpResponse {
        AcvpHttpResponse {
            status,
            content_length: Some(body.len() as u64),
            content_encodings: Vec::new(),
            body: Box::new(Cursor::new(body)),
        }
    }

    fn assert_provided_header(value: &ureq::config::AutoHeaderValue, expected: &str) {
        match value {
            ureq::config::AutoHeaderValue::Provided(value) => assert_eq!(value.as_str(), expected),
            other => panic!("expected provided header {expected:?}, got {other:?}"),
        }
    }

    #[test]
    fn ci_candidate_root_is_repository_scoped() {
        let root = repository_root();
        let parsed = parse_ci_root(&["--candidate-root".to_string(), root.display().to_string()])
            .expect("repository root is a valid candidate");
        assert_eq!(
            parsed,
            root.canonicalize().unwrap().join("conformance/rebuild-rs")
        );

        assert!(parse_ci_root(&["--candidate-root".to_string()]).is_err());
        assert!(parse_ci_root(&["--unknown".to_string(), "value".to_string()]).is_err());
    }

    fn test_dir(label: &str) -> PathBuf {
        let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!(
            "yamlsigil-xtask-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test directory");
        path
    }

    #[test]
    fn parse_update_uses_default_commit_when_no_args() {
        let it: std::vec::IntoIter<String> = Vec::<String>::new().into_iter();
        let got = parse_update_args(it).expect("no args is valid");
        assert_eq!(got, DEFAULT_COMMIT);
    }

    #[test]
    fn parse_update_accepts_explicit_commit() {
        let it = vec!["--commit".into(), VALID_COMMIT.into()].into_iter();
        let got = parse_update_args(it).expect("explicit commit is valid");
        assert_eq!(got, VALID_COMMIT);
    }

    #[test]
    fn parse_update_rejects_unknown_arg() {
        let it = vec!["--bogus".into()].into_iter();
        assert!(parse_update_args(it).is_err());
    }

    #[test]
    fn parse_update_rejects_missing_commit_value() {
        let it = vec!["--commit".into()].into_iter();
        assert!(parse_update_args(it).is_err());
    }

    #[test]
    fn parse_update_rejects_duplicate_commit() {
        let it = vec![
            "--commit".into(),
            VALID_COMMIT.into(),
            "--commit".into(),
            DEFAULT_COMMIT.into(),
        ]
        .into_iter();
        assert!(parse_update_args(it).is_err());
    }

    #[test]
    fn parse_update_rejects_noncanonical_commit_hashes() {
        let invalid = [
            String::new(),
            "a".repeat(39),
            "a".repeat(41),
            "A".repeat(40),
            format!("{}g", "a".repeat(39)),
            format!("{}/", "a".repeat(39)),
            format!("{}?", "a".repeat(39)),
            format!("{} ", "a".repeat(39)),
        ];

        for commit in invalid {
            let args = vec!["--commit".to_string(), commit].into_iter();
            assert!(parse_update_args(args).is_err());
        }
    }

    #[test]
    fn default_commit_is_canonical() {
        validate_commit(DEFAULT_COMMIT).expect("default commit must remain canonical");
    }

    #[test]
    fn update_revalidates_commit_before_side_effects() {
        let error = update_acvp("not-a-commit").expect_err("invalid commit must fail");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn acvp_http_config_is_explicit_and_bounded() {
        let config = acvp_http_config(None);
        assert!(!config.http_status_as_error());
        assert!(config.https_only());
        assert_eq!(config.max_redirects(), MAX_ACVP_REDIRECTS);
        assert!(config.max_redirects_will_error());
        assert_eq!(config.redirect_auth_headers(), RedirectAuthHeaders::Never);
        assert!(!config.save_redirect_history());
        assert_eq!(
            config.max_response_header_size(),
            MAX_ACVP_RESPONSE_HEADER_BYTES
        );
        assert!(config.proxy().is_none());
        assert_eq!(config.tls_config().provider(), TlsProvider::Rustls);
        assert!(matches!(
            config.tls_config().root_certs(),
            RootCerts::PlatformVerifier
        ));
        assert!(!config.tls_config().disable_verification());
        assert_provided_header(config.user_agent(), ACVP_USER_AGENT);
        assert_provided_header(config.accept(), "application/json");
        assert_provided_header(config.accept_encoding(), "identity");

        let timeouts = config.timeouts();
        assert_eq!(timeouts.global, Some(HTTP_GLOBAL_TIMEOUT));
        assert_eq!(timeouts.resolve, Some(HTTP_RESOLVE_TIMEOUT));
        assert_eq!(timeouts.connect, Some(HTTP_CONNECT_TIMEOUT));
        assert_eq!(timeouts.send_request, Some(HTTP_SEND_REQUEST_TIMEOUT));
        assert_eq!(timeouts.recv_response, Some(HTTP_RECV_RESPONSE_TIMEOUT));
        assert_eq!(timeouts.recv_body, Some(HTTP_RECV_BODY_TIMEOUT));
    }

    #[test]
    fn proxy_validation_fails_closed_without_disclosing_configuration() {
        assert!(validate_proxy(None, false).unwrap().is_none());
        let missing = validate_proxy(None, true).expect_err("invalid proxy must fail");
        assert_eq!(missing.kind(), io::ErrorKind::InvalidInput);

        let malformed = parse_supported_proxy(std::ffi::OsStr::new(
            "http://user:password@proxy invalid:8080",
        ))
        .expect_err("malformed proxy must fail");
        let malformed_detail = malformed.to_string();
        assert!(!malformed_detail.contains("user"));
        assert!(!malformed_detail.contains("password"));
        assert!(!malformed_detail.contains("proxy.invalid"));

        let supported =
            Proxy::new("http://user:password@proxy.invalid:8080").expect("HTTP proxy is supported");
        assert!(validate_proxy(Some(supported), true).unwrap().is_some());

        let error = parse_supported_proxy(std::ffi::OsStr::new(
            "socks5://user:password@proxy.invalid:1080",
        ))
        .expect_err("SOCKS proxy must be rejected");
        let detail = error.to_string();
        assert!(!detail.contains("user"));
        assert!(!detail.contains("password"));
        assert!(!detail.contains("proxy.invalid"));
    }

    #[test]
    fn download_uses_the_fixed_commit_url() {
        let url = acvp_url(VALID_COMMIT);
        let mut client = FakeHttpClient::returning(response_with_body(200, b"[]".to_vec()));
        let bytes = download_acvp(&mut client, &url).expect("bounded response is valid");
        assert_eq!(bytes, b"[]");
        assert_eq!(client.requested_url.as_deref(), Some(url.as_str()));
        assert!(url.starts_with("https://raw.githubusercontent.com/usnistgov/ACVP-Server/"));
        assert!(url.contains(VALID_COMMIT));
    }

    #[test]
    fn download_accepts_exact_body_limit() {
        let body = vec![b'x'; MAX_ACVP_CORPUS_BYTES];
        let mut client = FakeHttpClient::returning(AcvpHttpResponse {
            status: 200,
            content_length: None,
            content_encodings: Vec::new(),
            body: Box::new(Cursor::new(body)),
        });
        let bytes = download_acvp(&mut client, "https://example.invalid")
            .expect("exact body limit must be accepted");
        assert_eq!(bytes.len(), MAX_ACVP_CORPUS_BYTES);
    }

    #[test]
    fn download_rejects_limit_plus_one_with_unknown_or_lying_length() {
        for content_length in [None, Some(1)] {
            let body = vec![b'x'; MAX_ACVP_CORPUS_BYTES + 1];
            let mut client = FakeHttpClient::returning(AcvpHttpResponse {
                status: 200,
                content_length,
                content_encodings: Vec::new(),
                body: Box::new(Cursor::new(body)),
            });
            let error = download_acvp(&mut client, "https://example.invalid")
                .expect_err("limit plus one must be rejected");
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn declared_oversize_is_rejected_without_reading_body() {
        let mut client = FakeHttpClient::returning(AcvpHttpResponse {
            status: 200,
            content_length: Some(MAX_ACVP_CORPUS_BYTES as u64 + 1),
            content_encodings: Vec::new(),
            body: Box::new(PanicReader),
        });
        let error = download_acvp(&mut client, "https://example.invalid")
            .expect_err("declared oversize must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn non_200_statuses_are_rejected_without_reading_error_bodies() {
        for status in [204, 302, 404, 500] {
            let mut client = FakeHttpClient::returning(AcvpHttpResponse {
                status,
                content_length: None,
                content_encodings: Vec::new(),
                body: Box::new(PanicReader),
            });
            let error = download_acvp(&mut client, "https://example.invalid")
                .expect_err("non-200 status must be rejected");
            assert!(error.to_string().contains(&status.to_string()));
        }
    }

    #[test]
    fn content_encoding_is_identity_only_and_case_insensitive() {
        for identity in ["identity", "IDENTITY"] {
            let mut client = FakeHttpClient::returning(AcvpHttpResponse {
                status: 200,
                content_length: Some(2),
                content_encodings: vec![identity.to_string()],
                body: Box::new(Cursor::new(b"[]".to_vec())),
            });
            assert_eq!(
                download_acvp(&mut client, "https://example.invalid").unwrap(),
                b"[]"
            );
        }

        for encodings in [
            vec!["gzip".to_string()],
            vec!["identity".to_string(), "identity".to_string()],
            vec!["identity, gzip".to_string()],
        ] {
            let mut client = FakeHttpClient::returning(AcvpHttpResponse {
                status: 200,
                content_length: None,
                content_encodings: encodings,
                body: Box::new(PanicReader),
            });
            let error = download_acvp(&mut client, "https://example.invalid")
                .expect_err("encoded or ambiguous response must be rejected");
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
    }

    #[test]
    fn response_body_errors_are_redacted() {
        let mut client = FakeHttpClient::returning(AcvpHttpResponse {
            status: 200,
            content_length: None,
            content_encodings: Vec::new(),
            body: Box::new(PrefixThenError { sent_prefix: false }),
        });
        let error = download_acvp(&mut client, "https://example.invalid")
            .expect_err("midstream failure must propagate");
        assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        assert!(!error.to_string().contains("sensitive"));
    }

    #[test]
    fn request_errors_are_redacted() {
        let proxy = map_ureq_request_error(UreqError::ConnectProxyFailed(
            "user:password@proxy.invalid".to_string(),
        ));
        let redirect = map_ureq_request_error(UreqError::RequireHttpsOnly(
            "http://example.invalid/sensitive".to_string(),
        ));
        for error in [proxy, redirect] {
            let detail = error.to_string();
            assert!(!detail.contains("password"));
            assert!(!detail.contains("sensitive"));
            assert!(!detail.contains("proxy.invalid"));
        }

        let timeout = map_ureq_request_error(UreqError::Timeout(ureq::Timeout::Connect));
        assert_eq!(timeout.kind(), io::ErrorKind::TimedOut);
        assert!(timeout.to_string().contains("connect"));
    }

    /// README MUST embed the commit hash so the pin is auditable
    /// from the vendor README alone.
    #[test]
    fn render_readme_embeds_commit() {
        let out = render_readme(VALID_COMMIT, 42);
        assert!(out.contains(VALID_COMMIT));
        assert!(out.contains("42 bytes"));
        assert!(out.contains(UPSTREAM_REPO));
    }

    #[test]
    fn rendered_readme_matches_checked_in_companion() {
        let root = workspace_root();
        let corpus_size = fs::metadata(root.join("vendor/acvp").join(VENDORED_FILE_NAME))
            .expect("read vendored corpus metadata")
            .len();
        let checked_in = fs::read_to_string(root.join("vendor/acvp/README.md"))
            .expect("read checked-in ACVP README");

        assert_eq!(render_readme(DEFAULT_COMMIT, corpus_size), checked_in);
    }

    #[cfg(unix)]
    #[test]
    fn replace_regular_file_rejects_symlink_destination() {
        use std::os::unix::fs::symlink;

        let dir = test_dir("reject-destination-symlink");
        let target = dir.join("target.txt");
        fs::write(&target, b"keep").expect("write target");
        let destination = dir.join("destination.txt");
        symlink(&target, &destination).expect("create destination symlink");
        let pinned = PinnedDir::open(&dir).expect("pin vendor directory");

        let error = pinned
            .replace_regular_file("destination.txt", b"replace")
            .expect_err("destination symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&target).expect("read target"), b"keep");
        drop(pinned);
        fs::remove_file(destination).expect("remove destination symlink");
        fs::remove_file(target).expect("remove target");
        fs::remove_dir(dir).expect("remove test directory");
    }

    #[cfg(unix)]
    #[test]
    fn ensure_directory_rejects_symlink() {
        use std::os::unix::fs::symlink;

        let root = test_dir("reject-directory-symlink");
        let target = root.join("target");
        fs::create_dir(&target).expect("create target directory");
        let linked = root.join("linked");
        symlink(&target, &linked).expect("create directory symlink");
        let pinned = PinnedDir::open(&root).expect("pin workspace root");

        let error = pinned
            .ensure_child("linked")
            .expect_err("directory symlink must fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        drop(pinned);
        fs::remove_file(linked).expect("remove directory symlink");
        fs::remove_dir(target).expect("remove target directory");
        fs::remove_dir(root).expect("remove test root");
    }
}
