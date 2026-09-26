import { useRef, useState } from "react";

import { Button } from "../../../../components/universal/button/index.js";
import { SingleDropdown } from "../../../../components/universal/dropdown/index.js";

import { Icon } from "../../../../shared/Icon.jsx";
import { SettingRow } from "../components/SettingRow.jsx";

export function NetworkPanel({ onSave }) {
  const [proxyMode, setProxyMode] = useState("custom");
  const [proxyUrl, setProxyUrl] = useState("http://proxy.company.test:8080");
  const [proxyUsername, setProxyUsername] = useState("developer");
  const [proxyPassword, setProxyPassword] = useState("");
  const [proxyPasswordStored, setProxyPasswordStored] = useState(true);
  const [proxyBypass, setProxyBypass] = useState("localhost\n.internal.company.test\n10.0.0.0/8");
  const [verify, setVerify] = useState(true);
  const [useSystemCertificates, setUseSystemCertificates] = useState(true);
  const [certificatePath, setCertificatePath] = useState(
    "/Users/shuyi/.config/suncode/certs/dev-ca.pem",
  );
  const certificateInputRef = useRef(null);
  const chooseCertificate = () => {
    certificateInputRef.current?.click();
  };
  const chooseFallbackCertificate = (event) => {
    const file = event.target.files?.[0];
    if (file) setCertificatePath(`/Users/shuyi/Downloads/${file.name}`);
    event.target.value = "";
  };
  return (
    <div className="settings-panel-content">
      <div className="settings-panel-heading">
        <h2>Network</h2>
        <p>Configure how the embedded Rust agent reaches HTTP and HTTPS services.</p>
      </div>
      <div className="settings-panel-section">
        <span className="settings-section-label">Proxy</span>
        <SettingRow
          label="Proxy mode"
          hint="Applies to model providers, WebFetch, and remote HTTP MCP servers."
        >
          <SingleDropdown
            options={[
              { value: "no_proxy", label: "No proxy" },
              { value: "system", label: "System proxy" },
              { value: "custom", label: "Custom proxy" },
            ]}
            value={proxyMode}
            onChange={setProxyMode}
            ariaLabel="Proxy mode"
            className="settings-dropdown"
          />
        </SettingRow>
        {proxyMode === "custom" && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Custom proxy</span>
            <SettingRow
              label="Proxy URL"
              hint="Use an HTTP or HTTPS URL without embedded credentials."
            >
              <input
                className="field mono"
                value={proxyUrl}
                onChange={(event) => setProxyUrl(event.target.value)}
                aria-label="Proxy URL"
                spellCheck="false"
              />
            </SettingRow>
            <SettingRow label="Username" hint="Optional Basic proxy authentication username.">
              <input
                className="field mono"
                value={proxyUsername}
                onChange={(event) => setProxyUsername(event.target.value)}
                aria-label="Proxy username"
                autoComplete="off"
              />
            </SettingRow>
            <SettingRow
              label="Password"
              hint={
                proxyPasswordStored
                  ? "Password stored. Leave empty to keep it or remove it explicitly."
                  : "Optional Basic proxy authentication password."
              }
            >
              <div className="settings-secret-field">
                <input
                  className="field mono"
                  type="password"
                  value={proxyPassword}
                  onChange={(event) => setProxyPassword(event.target.value)}
                  aria-label="Proxy password"
                  placeholder={proxyPasswordStored ? "Password stored" : "Optional"}
                  autoComplete="new-password"
                />
                {proxyPasswordStored && (
                  <button type="button" onClick={() => setProxyPasswordStored(false)}>
                    Remove
                  </button>
                )}
              </div>
            </SettingRow>
            <SettingRow
              label="Bypass rules"
              hint="One hostname, domain suffix, IP address, CIDR range, or * per line. Loopback is always direct."
            >
              <textarea
                className="field mono settings-proxy-bypass"
                value={proxyBypass}
                onChange={(event) => setProxyBypass(event.target.value)}
                aria-label="Proxy bypass rules"
                spellCheck="false"
              />
            </SettingRow>
          </div>
        )}
        <div className="settings-actions">
          <Button variant="primary" size="sm" onClick={() => onSave("Proxy settings applied.")}>
            Save proxy settings
          </Button>
          <span className="settings-save-status" role="status">
            {proxyMode === "no_proxy"
              ? "Direct connections"
              : proxyMode === "system"
                ? "System proxy"
                : "Custom proxy"}
          </span>
        </div>
      </div>
      <div className="settings-divider" />
      <div className="settings-panel-section">
        <span className="settings-section-label">HTTPS security</span>
        <SettingRow
          label="Verify server certificates"
          hint="Validate certificate chains and hostnames for model provider and WebFetch requests."
        >
          <label className="settings-switch">
            <input
              type="checkbox"
              checked={verify}
              onChange={(event) => setVerify(event.target.checked)}
              aria-label="Verify server certificates"
            />
            <span className="settings-switch-track">
              <span />
            </span>
            <b>{verify ? "On" : "Off"}</b>
          </label>
        </SettingRow>
        {verify && (
          <div className="settings-subsection">
            <span className="settings-subsection-label">Certificate trust source</span>
            <SettingRow
              label="Use system certificates"
              hint="Trust the operating system certificate store for provider and WebFetch HTTPS requests."
            >
              <label className="settings-switch">
                <input
                  type="checkbox"
                  checked={useSystemCertificates}
                  onChange={(event) => setUseSystemCertificates(event.target.checked)}
                  aria-label="Use system certificates"
                />
                <span className="settings-switch-track">
                  <span />
                </span>
                <b>{useSystemCertificates ? "On" : "Off"}</b>
              </label>
            </SettingRow>
            <SettingRow
              label="Certificate path"
              hint={
                useSystemCertificates
                  ? "Disable system certificates to provide a custom certificate file."
                  : "Choose a PEM, CRT, CER, or DER certificate file for custom trust."
              }
            >
              <div
                className={`settings-directory-field settings-file-selector ${useSystemCertificates ? "is-disabled" : ""}`}
              >
                <input
                  className="field mono"
                  value={useSystemCertificates ? "" : certificatePath}
                  onChange={(event) => setCertificatePath(event.target.value)}
                  aria-label="Custom certificate path"
                  placeholder={
                    useSystemCertificates
                      ? "Using system certificates"
                      : "/path/to/custom-certificate.pem"
                  }
                  spellCheck="false"
                  disabled={useSystemCertificates}
                />
                <button
                  type="button"
                  aria-label="Choose certificate file"
                  title="Choose file"
                  onClick={chooseCertificate}
                  disabled={useSystemCertificates}
                >
                  <Icon name="folder" size={16} />
                </button>
                <input
                  ref={certificateInputRef}
                  type="file"
                  accept=".pem,.crt,.cer,.der"
                  aria-hidden="true"
                  tabIndex="-1"
                  onChange={chooseFallbackCertificate}
                />
              </div>
            </SettingRow>
          </div>
        )}
        {!verify && (
          <div className="settings-warning">
            <Icon name="platform" size={16} />
            <div>
              <strong>Certificate verification is off</strong>
              <span>
                SunCode will accept invalid certificates and hostnames, similar to{" "}
                <code>curl -k</code>. This can expose provider credentials and fetched content to
                man-in-the-middle attacks.
              </span>
            </div>
          </div>
        )}
      </div>
      <div className="settings-actions">
        <Button variant="primary" size="sm" onClick={onSave}>
          Save HTTPS setting
        </Button>
        <span className="settings-save-status" role="status">
          {verify
            ? useSystemCertificates
              ? "System trust store"
              : "Custom certificate required"
            : "Review required"}
        </span>
      </div>
    </div>
  );
}
