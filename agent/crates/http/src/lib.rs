//! Shared TLS trust and proxy settings for SunCode's outbound HTTP clients.
//!
//! reqwest's async and blocking builders are different types, so this crate
//! resolves settings into reqwest values and a [`TrustRoots`] decision; each
//! client applies them to its own builder.

use suncode_common::{BusinessError, HttpProxyConfiguration, HttpProxyMode};

const PEM_MARKER: &[u8] = b"-----BEGIN CERTIFICATE-----";
/// A DER certificate is an ASN.1 SEQUENCE.
const DER_SEQUENCE_TAG: u8 = 0x30;

/// How a client should trust server certificates.
pub enum TrustRoots {
    /// Skip verification entirely.
    Disabled,
    /// Verify against the operating system trust store plus `extra` roots.
    System { extra: Vec<reqwest::Certificate> },
    /// Verify against `roots` only.
    Only { roots: Vec<reqwest::Certificate> },
}

/// Resolves trust settings, reading and validating `certificate_path` when
/// verification is enabled.
pub fn trust_roots(
    verify_certificates: bool,
    use_system_certificates: bool,
    certificate_path: Option<&std::path::Path>,
) -> Result<TrustRoots, BusinessError> {
    if !verify_certificates {
        return Ok(TrustRoots::Disabled);
    }
    let custom = match certificate_path {
        Some(path) => {
            let bytes = std::fs::read(path)
                .map_err(|_| fatal("certificate_unavailable", "certificate file is unavailable"))?;
            load_certificates(&bytes)?
        }
        None => Vec::new(),
    };
    Ok(if use_system_certificates {
        TrustRoots::System { extra: custom }
    } else {
        TrustRoots::Only { roots: custom }
    })
}

/// Parses a PEM bundle or a single DER certificate. With rustls, reqwest
/// defers parsing to the handshake, so validate here to fail early.
pub fn load_certificates(bytes: &[u8]) -> Result<Vec<reqwest::Certificate>, BusinessError> {
    let invalid = || {
        fatal(
            "certificate_invalid",
            "certificate file is not a valid PEM or DER certificate",
        )
    };
    if bytes
        .windows(PEM_MARKER.len())
        .any(|window| window == PEM_MARKER)
    {
        let certificates = reqwest::Certificate::from_pem_bundle(bytes).map_err(|_| invalid())?;
        if certificates.is_empty() {
            return Err(invalid());
        }
        return Ok(certificates);
    }
    if bytes.first() != Some(&DER_SEQUENCE_TAG) {
        return Err(invalid());
    }
    reqwest::Certificate::from_der(bytes)
        .map(|certificate| vec![certificate])
        .map_err(|_| invalid())
}

/// How a client should route requests through a proxy.
pub enum ProxyRoute {
    /// Connect directly, ignoring system proxy settings.
    Direct,
    /// Use reqwest's system proxy detection.
    System,
    /// Use only this proxy, with its bypass list applied.
    Custom(Box<reqwest::Proxy>),
}

pub fn proxy_route(configuration: &HttpProxyConfiguration) -> Result<ProxyRoute, BusinessError> {
    match configuration.mode {
        HttpProxyMode::NoProxy => Ok(ProxyRoute::Direct),
        HttpProxyMode::System => Ok(ProxyRoute::System),
        HttpProxyMode::Custom => {
            let mut proxy = reqwest::Proxy::all(&configuration.url)
                .map_err(|_| fatal("invalid_arguments", "custom proxy URL is invalid"))?;
            if !configuration.username.is_empty() {
                proxy = proxy.basic_auth(&configuration.username, &configuration.password);
            }
            Ok(ProxyRoute::Custom(Box::new(proxy.no_proxy(
                reqwest::NoProxy::from_string(&configuration.no_proxy_value()),
            ))))
        }
    }
}

/// Applies resolved trust and proxy settings to a reqwest client builder.
/// Implemented for both the async and blocking builders.
pub trait ApplyHttpSettings: Sized {
    fn apply_trust(self, trust: TrustRoots) -> Self;
    fn apply_proxy(self, route: ProxyRoute) -> Self;
}

macro_rules! impl_apply {
    ($builder:ty) => {
        impl ApplyHttpSettings for $builder {
            fn apply_trust(self, trust: TrustRoots) -> Self {
                match trust {
                    TrustRoots::Disabled => self.tls_danger_accept_invalid_certs(true),
                    TrustRoots::System { extra } if extra.is_empty() => self,
                    TrustRoots::System { extra } => self.tls_certs_merge(extra),
                    TrustRoots::Only { roots } => self.tls_certs_only(roots),
                }
            }

            fn apply_proxy(self, route: ProxyRoute) -> Self {
                match route {
                    ProxyRoute::Direct => self.no_proxy(),
                    ProxyRoute::System => self,
                    ProxyRoute::Custom(proxy) => self.no_proxy().proxy(*proxy),
                }
            }
        }
    };
}

impl_apply!(reqwest::ClientBuilder);
#[cfg(feature = "blocking")]
impl_apply!(reqwest::blocking::ClientBuilder);

fn fatal(code: &str, message: &str) -> BusinessError {
    BusinessError::new(code, message).with_retryable(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_a_system_pem_bundle() {
        // Present on macOS and most Linux distributions; skip where absent.
        let Some(bundle) = ["/etc/ssl/cert.pem", "/etc/ssl/certs/ca-certificates.crt"]
            .iter()
            .find_map(|path| std::fs::read(path).ok())
        else {
            return;
        };
        assert!(!load_certificates(&bundle).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_certificate_contents() {
        for contents in [
            b"not a certificate".as_slice(),
            b"-----BEGIN CERTIFICATE-----\n@@@\n-----END CERTIFICATE-----\n",
        ] {
            assert_eq!(
                load_certificates(contents).err().unwrap().code,
                "certificate_invalid"
            );
        }
    }

    #[test]
    fn disabled_verification_skips_the_certificate_file() {
        let missing = std::path::Path::new("/nonexistent/suncode-certificate.pem");
        assert!(matches!(
            trust_roots(false, true, Some(missing)),
            Ok(TrustRoots::Disabled)
        ));
        assert_eq!(
            trust_roots(true, true, Some(missing)).err().unwrap().code,
            "certificate_unavailable"
        );
    }
}
