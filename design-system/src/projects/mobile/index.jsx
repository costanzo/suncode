import { Button } from "../../components/universal/button/index.js";
import { Icon } from "../../shared/Icon.jsx";
import { ModuleLink, PageHeader, Section, Status } from "../../shared/PagePrimitives.jsx";

const sessions = [
  {
    title: "Fix login redirect",
    host: "MacBook Pro",
    project: "suncode",
    state: "Waiting for approval",
    tone: "warning",
    time: "2 min ago",
    preview: "I found one file write that needs your approval.",
  },
  {
    title: "Refactor API client",
    host: "Windows Desktop",
    project: "mobile-app",
    state: "Running",
    tone: "running",
    time: "12 min ago",
    preview: "Updating the request retry policy and its tests.",
  },
  {
    title: "Review release notes",
    host: "MacBook Pro",
    project: "design-system",
    state: "Idle",
    tone: "neutral",
    time: "Yesterday",
    preview: "The latest response is ready to review.",
  },
];

const hosts = [
  {
    name: "MacBook Pro",
    endpoint: "192.168.1.24",
    state: "Connected",
    meta: "3 projects · 2 active sessions",
    tone: "success",
  },
  {
    name: "Windows Desktop",
    endpoint: "10.0.0.18",
    state: "Offline",
    meta: "Last seen 18 min ago",
    tone: "neutral",
  },
];

function MobileFrame({
  title,
  subtitle,
  activeTab = "Sessions",
  action,
  children,
  hideNavigation = false,
}) {
  return (
    <div
      className={`mobile-frame${hideNavigation ? " mobile-frame-detail" : ""}`}
      aria-label={`${title} mobile screen specimen`}
    >
      <div className="mobile-frame-statusbar">
        <span>9:41</span>
        <span className="mobile-frame-status-icons">
          <i /> <i /> <i />
        </span>
      </div>
      <div className="mobile-frame-appbar">
        <div>
          <h3>{title}</h3>
          {subtitle && <p>{subtitle}</p>}
        </div>
        {action}
      </div>
      <div className="mobile-frame-content">{children}</div>
      {!hideNavigation && (
        <nav className="mobile-bottom-nav" aria-label="Mobile primary navigation">
          {["Sessions", "Hosts", "Settings"].map((item) => (
            <span key={item} className={item === activeTab ? "is-active" : ""}>
              <Icon
                name={item === "Sessions" ? "message" : item === "Hosts" ? "server" : "settings"}
                size={18}
              />
              <small>{item}</small>
            </span>
          ))}
        </nav>
      )}
    </div>
  );
}

function SessionRow({ session }) {
  return (
    <div className="mobile-session-row">
      <div className="mobile-row-main">
        <div className="mobile-row-title">
          <strong>{session.title}</strong>
          <span className={`mobile-state-dot is-${session.tone}`} />
        </div>
        <span className="mobile-row-preview">{session.preview}</span>
        <span className="mobile-row-meta">
          {session.host} · {session.project}
        </span>
      </div>
      <div className="mobile-row-side">
        <time>{session.time}</time>
        <span className={`mobile-state-text is-${session.tone}`}>{session.state}</span>
      </div>
    </div>
  );
}

function SessionInboxScreen() {
  return (
    <MobileFrame
      title="Sessions"
      activeTab="Sessions"
      action={
        <button className="mobile-icon-button" aria-label="Search sessions">
          <Icon name="search" size={19} />
        </button>
      }
    >
      <div className="mobile-filter-row">
        <span className="mobile-filter-chip is-selected">All hosts</span>
        <span className="mobile-filter-chip">All projects</span>
        <span className="mobile-filter-chip">
          <Icon name="more" size={14} />
        </span>
      </div>
      <div className="mobile-sync-note">
        <span className="mobile-state-dot is-success" /> Updated just now
      </div>
      <div className="mobile-session-list">
        {sessions.map((session) => (
          <SessionRow key={session.title} session={session} />
        ))}
      </div>
    </MobileFrame>
  );
}

function SessionDetailScreen() {
  return (
    <MobileFrame
      title="Fix login redirect"
      activeTab="Sessions"
      hideNavigation
      action={
        <button className="mobile-icon-button" aria-label="Session actions">
          <Icon name="more" size={19} />
        </button>
      }
    >
      <div className="mobile-connection-banner">
        <span className="mobile-state-dot is-success" /> Connected to MacBook Pro
      </div>
      <div className="mobile-message mobile-message-user">
        Fix the redirect loop after login and add a regression test.
      </div>
      <div className="mobile-message mobile-message-agent">
        <strong>SunCode</strong>
        <p>
          I found the redirect loop in <code>auth/redirect.ts</code>. The fix is ready, but writing
          the file requires your approval.
        </p>
        <div className="mobile-approval-card">
          <div>
            <span className="mobile-state-dot is-warning" /> Write 1 file
          </div>
          <code>src/auth/redirect.ts</code>
          <div className="mobile-inline-actions">
            <Button variant="primary">Allow once</Button>
            <Button variant="quiet">Deny</Button>
          </div>
        </div>
      </div>
      <div className="mobile-composer">
        <span>Message SunCode…</span>
        <button className="mobile-send-button" aria-label="Send message">
          <Icon name="arrow-up" size={16} />
        </button>
      </div>
    </MobileFrame>
  );
}

function HostListScreen() {
  return (
    <MobileFrame
      title="Hosts"
      subtitle="2 connected targets"
      activeTab="Hosts"
      action={
        <button className="mobile-icon-button" aria-label="Add host">
          <Icon name="plus" size={20} />
        </button>
      }
    >
      <div className="mobile-host-list">
        {hosts.map((host) => (
          <div className="mobile-host-row" key={host.name}>
            <span className="mobile-host-icon">
              <Icon name="server" size={20} />
            </span>
            <div className="mobile-row-main">
              <strong>{host.name}</strong>
              <span className="mobile-row-meta">
                <code>{host.endpoint}</code> · {host.meta}
              </span>
            </div>
            <span className={`mobile-state-text is-${host.tone}`}>{host.state}</span>
          </div>
        ))}
      </div>
      <div className="mobile-pairing-sheet">
        <div>
          <strong>Pair a new Desktop</strong>
          <p>Scan the one-time QR code shown by SunCode Desktop.</p>
        </div>
        <Button variant="primary" icon="computer">
          Scan QR code
        </Button>
      </div>
    </MobileFrame>
  );
}

function SettingsScreen() {
  return (
    <MobileFrame title="Settings" activeTab="Settings">
      <div className="mobile-settings-group">
        <span className="mobile-section-label">Appearance</span>
        <div className="mobile-setting-row">
          <span>
            <Icon name="moon" size={18} /> Theme
          </span>
          <strong>System</strong>
          <Icon name="chevron-right" size={16} />
        </div>
        <div className="mobile-setting-row">
          <span>
            <Icon name="keyboard" size={18} /> Notifications
          </span>
          <strong>On</strong>
          <Icon name="chevron-right" size={16} />
        </div>
      </div>
      <div className="mobile-settings-group">
        <span className="mobile-section-label">Security</span>
        <div className="mobile-setting-row">
          <span>
            <Icon name="lock" size={18} /> Paired devices
          </span>
          <strong>1</strong>
          <Icon name="chevron-right" size={16} />
        </div>
        <div className="mobile-setting-row">
          <span>
            <Icon name="refresh" size={18} /> Offline cache
          </span>
          <strong>128 MB</strong>
          <Icon name="chevron-right" size={16} />
        </div>
      </div>
      <div className="mobile-settings-group">
        <span className="mobile-section-label">About</span>
        <div className="mobile-setting-row">
          <span>
            <Icon name="mobile" size={18} /> App version
          </span>
          <code>0.1.0</code>
        </div>
        <div className="mobile-setting-row">
          <span>
            <Icon name="foundation" size={18} /> Protocol
          </span>
          <code>remote.v1</code>
        </div>
      </div>
    </MobileFrame>
  );
}

function PairingScannerScreen() {
  return (
    <MobileFrame
      title="Scan QR code"
      hideNavigation
      action={
        <button className="mobile-icon-button" aria-label="Close scanner">
          <Icon name="close" size={19} />
        </button>
      }
    >
      <div className="mobile-scanner-copy">
        <strong>Pair a new Desktop</strong>
        <span>Use the camera to scan the one-time code shown by SunCode Desktop.</span>
      </div>
      <div className="mobile-scanner-view" aria-label="Camera scanner frame">
        <span className="scanner-corner top-left" />
        <span className="scanner-corner top-right" />
        <span className="scanner-corner bottom-left" />
        <span className="scanner-corner bottom-right" />
        <span className="scanner-line" />
        <span className="scanner-hint">Align QR code inside the frame</span>
      </div>
      <div className="mobile-scanner-status">
        <span className="mobile-state-dot is-neutral" /> Camera permission is required to scan.
      </div>
    </MobileFrame>
  );
}

function PairingConfirmationScreen() {
  return (
    <MobileFrame title="Confirm pairing" hideNavigation>
      <div className="mobile-pairing-identity">
        <span className="mobile-host-icon">
          <Icon name="server" size={22} />
        </span>
        <div>
          <strong>MacBook Pro</strong>
          <span>SunCode Desktop · remote.v1</span>
        </div>
        <span className="mobile-state-text is-success">Verified</span>
      </div>
      <div className="mobile-detail-list">
        <div>
          <span>Remote Server</span>
          <code>relay.suncode.dev</code>
        </div>
        <div>
          <span>Endpoint</span>
          <code>192.168.1.24</code>
        </div>
        <div>
          <span>Fingerprint</span>
          <code>SHA256: 7A:31:…:D9</code>
        </div>
      </div>
      <div className="mobile-warning-note">
        <Icon name="lock" size={17} />
        <span>Confirm only if this identity matches the Desktop you opened.</span>
      </div>
      <div className="mobile-stacked-actions">
        <Button variant="primary">Confirm pairing</Button>
        <Button variant="quiet">Cancel</Button>
      </div>
    </MobileFrame>
  );
}

function NewSessionScreen() {
  return (
    <MobileFrame title="New session" hideNavigation>
      <div className="mobile-form-stack">
        <label className="mobile-form-field">
          <span>Host</span>
          <strong>MacBook Pro</strong>
          <Icon name="chevron-right" size={16} />
        </label>
        <label className="mobile-form-field">
          <span>Project</span>
          <strong>suncode</strong>
          <Icon name="chevron-right" size={16} />
        </label>
        <label className="mobile-form-field is-input">
          <span>Session title</span>
          <strong>Fix login redirect</strong>
        </label>
        <label className="mobile-form-field is-textarea">
          <span>First message</span>
          <strong>Describe what you want SunCode to do…</strong>
        </label>
      </div>
      <div className="mobile-form-note">
        <Icon name="message" size={16} />
        <span>A new primary session will be created on the selected Desktop.</span>
      </div>
      <div className="mobile-stacked-actions">
        <Button variant="primary">Create session</Button>
        <Button variant="quiet">Cancel</Button>
      </div>
    </MobileFrame>
  );
}

function QuestionScreen() {
  return (
    <MobileFrame
      title="Fix login redirect"
      hideNavigation
      action={
        <button className="mobile-icon-button" aria-label="Session actions">
          <Icon name="more" size={19} />
        </button>
      }
    >
      <div className="mobile-connection-banner">
        <span className="mobile-state-dot is-warning" /> Waiting for your answer
      </div>
      <div className="mobile-message mobile-message-agent">
        <strong>SunCode</strong>
        <p>Which redirect behavior should I preserve for an already authenticated user?</p>
      </div>
      <div className="mobile-question-card">
        <span className="mobile-section-label">Choose one</span>
        <button className="mobile-choice-row is-selected">
          <span className="mobile-radio-dot" /> Return to the requested page
        </button>
        <button className="mobile-choice-row">
          <span className="mobile-radio-dot" /> Always open the dashboard
        </button>
        <button className="mobile-choice-row">
          <span className="mobile-radio-dot" /> Let me answer with text
        </button>
        <div className="mobile-question-input">Add an explanation (optional)</div>
        <Button variant="primary">Send answer</Button>
      </div>
    </MobileFrame>
  );
}

function ReconnectScreen() {
  return (
    <MobileFrame
      title="Sessions"
      activeTab="Sessions"
      action={
        <button className="mobile-icon-button" aria-label="Retry connection">
          <Icon name="refresh" size={19} />
        </button>
      }
    >
      <div className="mobile-reconnect-panel">
        <span className="mobile-reconnect-icon">
          <Icon name="server" size={24} />
        </span>
        <strong>Connection lost</strong>
        <p>
          Remote Server cannot reach MacBook Pro. Cached sessions remain available while we
          reconnect.
        </p>
        <div className="mobile-reconnect-progress">
          <span />
        </div>
        <span className="mobile-state-text is-neutral">Retrying in 4 seconds</span>
        <Button variant="primary" icon="refresh">
          Retry now
        </Button>
      </div>
      <div className="mobile-session-list is-dimmed">
        {sessions.slice(0, 2).map((session) => (
          <SessionRow key={session.title} session={session} />
        ))}
      </div>
    </MobileFrame>
  );
}

function CacheScreen() {
  return (
    <MobileFrame title="Offline cache" activeTab="Settings">
      <div className="mobile-cache-summary">
        <div>
          <strong>128 MB</strong>
          <span>of 512 MB used</span>
        </div>
        <div className="mobile-cache-meter">
          <span />
        </div>
        <Button variant="quiet">Manage</Button>
      </div>
      <div className="mobile-cache-list">
        <div>
          <span>
            <Icon name="message" size={18} /> Sessions
          </span>
          <strong>94 MB</strong>
        </div>
        <div>
          <span>
            <Icon name="files" size={18} /> Attachments
          </span>
          <strong>28 MB</strong>
        </div>
        <div>
          <span>
            <Icon name="server" size={18} /> Host metadata
          </span>
          <strong>6 MB</strong>
        </div>
      </div>
      <div className="mobile-offline-note">
        <span className="mobile-state-dot is-neutral" />
        <span>
          Cached messages and activity remain readable offline. New messages wait locally until
          connection returns.
        </span>
      </div>
      <Button variant="danger">Clear offline cache</Button>
    </MobileFrame>
  );
}

function HostDetailScreen() {
  return (
    <MobileFrame
      title="MacBook Pro"
      activeTab="Hosts"
      action={
        <button className="mobile-icon-button" aria-label="Host actions">
          <Icon name="more" size={19} />
        </button>
      }
    >
      <div className="mobile-host-detail-header">
        <span className="mobile-host-icon">
          <Icon name="server" size={24} />
        </span>
        <div>
          <strong>Connected</strong>
          <span>Last sync just now</span>
        </div>
        <span className="mobile-state-dot is-success" />
      </div>
      <div className="mobile-detail-list">
        <div>
          <span>Remote Server</span>
          <code>relay.suncode.dev</code>
        </div>
        <div>
          <span>Endpoint</span>
          <code>192.168.1.24</code>
        </div>
        <div>
          <span>Protocol</span>
          <code>remote.v1</code>
        </div>
      </div>
      <div className="mobile-project-heading">
        <strong>Projects</strong>
        <span>3</span>
      </div>
      <div className="mobile-project-list">
        <div>
          <Icon name="folder" size={18} />
          <span>
            <strong>suncode</strong>
            <small>2 active sessions</small>
          </span>
          <Icon name="chevron-right" size={16} />
        </div>
        <div>
          <Icon name="folder" size={18} />
          <span>
            <strong>mobile-app</strong>
            <small>1 idle session</small>
          </span>
          <Icon name="chevron-right" size={16} />
        </div>
        <div>
          <Icon name="folder" size={18} />
          <span>
            <strong>design-system</strong>
            <small>No active sessions</small>
          </span>
          <Icon name="chevron-right" size={16} />
        </div>
      </div>
    </MobileFrame>
  );
}

function TabletDualPaneScreen() {
  return (
    <div className="mobile-tablet-shell" aria-label="Tablet two-pane layout specimen">
      <aside className="mobile-tablet-rail">
        <strong>SunCode</strong>
        <span className="is-active">
          <Icon name="message" size={18} /> Sessions
        </span>
        <span>
          <Icon name="server" size={18} /> Hosts
        </span>
        <span>
          <Icon name="settings" size={18} /> Settings
        </span>
      </aside>
      <section className="mobile-tablet-list">
        <div className="mobile-tablet-title">
          <h3>Sessions</h3>
          <button className="mobile-icon-button" aria-label="New session">
            <Icon name="plus" size={18} />
          </button>
        </div>
        <div className="mobile-session-list">
          {sessions.map((session) => (
            <SessionRow key={session.title} session={session} />
          ))}
        </div>
      </section>
      <section className="mobile-tablet-detail">
        <div className="mobile-tablet-title">
          <div>
            <h3>Fix login redirect</h3>
            <span>MacBook Pro · suncode</span>
          </div>
          <span className="mobile-state-text is-warning">Waiting for approval</span>
        </div>
        <div className="mobile-message mobile-message-user">
          Fix the redirect loop after login and add a regression test.
        </div>
        <div className="mobile-message mobile-message-agent">
          <strong>SunCode</strong>
          <p>The fix is ready and waiting for your approval.</p>
          <div className="mobile-approval-card">
            <span>
              <span className="mobile-state-dot is-warning" /> Write 1 file
            </span>
            <code>src/auth/redirect.ts</code>
            <div className="mobile-inline-actions">
              <Button variant="primary">Allow once</Button>
              <Button variant="quiet">Deny</Button>
            </div>
          </div>
        </div>
        <div className="mobile-composer">
          <span>Message SunCode…</span>
          <button className="mobile-send-button" aria-label="Send message">
            <Icon name="arrow-up" size={16} />
          </button>
        </div>
      </section>
    </div>
  );
}

function MobilePageSection({ title, description, children }) {
  return (
    <Section title={title} description={description} className="mobile-project-section">
      {children}
    </Section>
  );
}

export function MobileProjectPage() {
  return (
    <>
      <PageHeader
        title="Mobile"
        description="The CMP mobile client for cross-host session control, secure Desktop pairing, and offline-first review."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection
        title="Application shell"
        description="Compact phones use a three-destination bottom bar; larger layouts promote the same destinations to a rail without changing the information architecture."
      >
        <div className="mobile-specimen-grid">
          <SessionInboxScreen />
          <HostListScreen />
        </div>
      </MobilePageSection>
      <Section
        id="mobile-modules"
        title="Mobile surfaces"
        description="Each surface has a stable route so its states can be reviewed without turning the overview into a long scroll."
      >
        <div className="module-card-grid">
          <ModuleLink
            to="/projects/mobile/shell"
            icon="mobile"
            title="App shell"
            description="Bottom navigation, adaptive rail, theme behavior, and connection status."
            path="Compact + expanded width"
          />
          <ModuleLink
            to="/projects/mobile/sessions"
            icon="message"
            title="Sessions"
            description="Global inbox, Host/project filters, primary-session detail, composer, approvals, and offline cache."
            path="Global across Hosts"
          />
          <ModuleLink
            to="/projects/mobile/hosts"
            icon="server"
            title="Hosts"
            description="Secure pairing, one-time QR flow, multi-project targets, and connection recovery."
            path="Desktop ↔ Remote Server ↔ Mobile"
          />
          <ModuleLink
            to="/projects/mobile/settings"
            icon="settings"
            title="Settings"
            description="System/light/dark appearance, notifications, security, cache, and version information."
            path="System-aware preferences"
          />
        </div>
      </Section>
    </>
  );
}

export function MobileShellPage() {
  return (
    <>
      <PageHeader
        title="Mobile app shell"
        description="The shared container and navigation contract for every mobile surface."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection
        title="Navigation and themes"
        description="The same semantic tokens carry across light and dark modes; System follows the device appearance."
      >
        <div className="mobile-specimen-grid">
          <SessionInboxScreen />
          <SettingsScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Expanded layout"
        description="At tablet width, the bottom destinations become a rail and Sessions becomes a persistent list/detail composition."
      >
        <TabletDualPaneScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileSessionsPage() {
  return (
    <>
      <PageHeader
        title="Mobile sessions"
        description="A global primary-session inbox across every paired Host, with explicit Host and project context on every row."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Global inbox">
        <div className="mobile-specimen-grid">
          <SessionInboxScreen />
          <SessionDetailScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Offline contract"
        description="Cached content remains readable when the Remote Server or Desktop is unavailable. Sending is disabled until the connection is restored; unsent text remains in the composer and is never silently dropped."
      >
        <div className="mobile-offline-specimen">
          <span className="mobile-state-dot is-neutral" />
          <strong>Offline — showing cached content</strong>
          <span>Last synced 8 min ago</span>
          <Button variant="quiet" icon="refresh">
            Retry connection
          </Button>
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="New session and question"
        description="Creating a session is explicit about Host and Project; a structured Question keeps the turn suspended until the user answers."
      >
        <div className="mobile-specimen-grid">
          <NewSessionScreen />
          <QuestionScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Remote Server recovery"
        description="Live WebSocket state is visible, but cached session content remains readable while the relay reconnects."
      >
        <ReconnectScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileHostsPage() {
  return (
    <>
      <PageHeader
        title="Mobile hosts"
        description="Manage paired SunCode Desktop targets. One Host may expose several projects and active primary sessions."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Host management">
        <div className="mobile-specimen-grid">
          <HostListScreen />
          <HostListScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Pairing contract"
        description="Desktop creates a one-time QR payload. A successful pairing invalidates that QR immediately; the mobile client stores a revocable credential and connects through the Remote Server over HTTP/WebSocket."
      >
        <div className="mobile-contract-grid">
          <div>
            <span className="mobile-contract-step">1</span>
            <strong>Scan</strong>
            <p>Read the one-time Desktop QR payload.</p>
          </div>
          <div>
            <span className="mobile-contract-step">2</span>
            <strong>Confirm</strong>
            <p>Show Host identity and endpoint before trust.</p>
          </div>
          <div>
            <span className="mobile-contract-step">3</span>
            <strong>Connect</strong>
            <p>Remote Server upgrades the live channel to WebSocket.</p>
          </div>
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Pairing states"
        description="The scanner reads a one-time Desktop payload; confirmation shows the Host identity and relay details before trust is stored."
      >
        <div className="mobile-specimen-grid">
          <PairingScannerScreen />
          <PairingConfirmationScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Host detail"
        description="Host identity and connection state sit above the Projects list. Mobile can inspect and use the Host but cannot revoke other devices."
      >
        <HostDetailScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileSettingsPage() {
  return (
    <>
      <PageHeader
        title="Mobile settings"
        description="App-level preferences stay separate from Host and project configuration."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Settings and appearance">
        <div className="mobile-specimen-grid">
          <SettingsScreen />
          <SettingsScreen />
        </div>
      </MobilePageSection>
      <MobilePageSection
        title="Offline cache"
        description="Cache management reports what remains available offline without implying that unsent content has reached Desktop."
      >
        <CacheScreen />
      </MobilePageSection>
      <MobilePageSection
        title="Preference rules"
        description="Theme is System, Light, or Dark. Security settings can clear local credentials and cache, but the mobile client cannot revoke other devices from a Host."
      >
        <div className="mobile-preference-list">
          <div>
            <Icon name="moon" size={18} />
            <strong>Theme</strong>
            <span>System / Light / Dark</span>
          </div>
          <div>
            <Icon name="lock" size={18} />
            <strong>Device authority</strong>
            <span>Own device only</span>
          </div>
          <div>
            <Icon name="refresh" size={18} />
            <strong>Offline cache</strong>
            <span>Readable while disconnected</span>
          </div>
        </div>
      </MobilePageSection>
    </>
  );
}

export function MobilePairingPage() {
  return (
    <>
      <PageHeader
        title="QR pairing"
        description="One-time QR scanning and explicit Host identity confirmation."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Pairing flow">
        <div className="mobile-specimen-grid">
          <PairingScannerScreen />
          <PairingConfirmationScreen />
        </div>
      </MobilePageSection>
    </>
  );
}

export function MobileNewSessionPage() {
  return (
    <>
      <PageHeader
        title="New session"
        description="Create a primary session by selecting the Host, Project, title, and first message."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Create session">
        <NewSessionScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileQuestionPage() {
  return (
    <>
      <PageHeader
        title="Question response"
        description="A structured answer surface for a turn waiting on user input."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Waiting for answer">
        <QuestionScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileReconnectPage() {
  return (
    <>
      <PageHeader
        title="Remote Server recovery"
        description="Reconnect the WebSocket channel while keeping cached sessions readable."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Disconnected state">
        <ReconnectScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileCachePage() {
  return (
    <>
      <PageHeader
        title="Offline cache"
        description="Inspect local session, attachment, and Host metadata cache."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Cache details">
        <CacheScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileHostDetailPage() {
  return (
    <>
      <PageHeader
        title="Host detail"
        description="Connection identity, relay endpoint, protocol state, and the Projects exposed by one Desktop."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="MacBook Pro">
        <HostDetailScreen />
      </MobilePageSection>
    </>
  );
}

export function MobileAdaptivePage() {
  return (
    <>
      <PageHeader
        title="Adaptive tablet layout"
        description="Expanded widths promote navigation to a rail and keep the Session list beside its detail."
        status="Review reference"
        tone="implemented"
      />
      <MobilePageSection title="Two-pane composition">
        <TabletDualPaneScreen />
      </MobilePageSection>
    </>
  );
}
