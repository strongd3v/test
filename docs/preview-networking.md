# Project previews

Run an HTTP development server in a project and open a Browser tab in that
session. The empty tab lists its running services. **Open** navigates to
`http://<device>.<project>.localhost:7331`; additional services receive persistent
names such as `<device>.<project>-api.localhost:7331`.

The daemon owns this listener, discovery and peer connections. Closing a browser
tab does not stop discovery or change an address. Account teardown cancels the
proxy, discovery and existing peer streams before the next profile starts. A
busy proxy port produces an inline error instead of changing the URL.

## Discovery and identities

On Linux, the scanner joins the current user's `/proc` process cwd, ancestry,
creation time and socket descriptors to listening TCP sockets. On macOS it uses
`lsof` and `ps` for equivalent metadata. Only loopback-reachable listeners whose
cwd belongs to a known local project are probed. The deepest matching project
wins. Commands with explicit authentication arguments (`login`, `auth`,
`authenticate`, `signin`, `sign-in`, `sso`, `oauth`, or `oauth2`, before `--`)
are excluded before any connection, even when started by a Space terminal or
agent. Their listeners may be one-shot browser callbacks: for example,
`infisical login` fails with EOF if it receives a discovery HEAD without the
browser's JSON body. This command check does not identify custom callback
servers without an authentication argument.

Unrelated listeners and non-HTTP services are excluded. Discovery runs every
two seconds; bounded HEAD probes accept valid HTTP status responses (including
authentication and application errors). Confirmed HTTP listeners are not
probed again during that socket's lifetime; non-HTTP results use a capped
exponential backoff.

Space terminal/task/agent descendants are marked as Space-owned. Framework
commands identify Vite, Next.js, Astro, Miniflare and Node servers; otherwise the
list uses a generic HTTP label. Before a local backend connection, the daemon
rechecks the listener's process identity and cwd to reject stale port reuse.

The profile's `previews.json` stores project and service IDs and hostname labels
separately from observed PIDs and ports. Command identities omit common port
options. Device-name collisions get persisted suffixes. The first service keeps
the project hostname; additional services get descriptive suffixes. Two
indistinguishable instances of the same command in the same cwd get separate
slots; their individual identities cannot be inferred across simultaneous
restarts without additional application-provided identity.

`WatchPreviews` resolves the session's current cwd on each catalog or workspace
change. On the viewing device it selects either the local services or the
advertised services for the session's owning device. Changing the active
checkout therefore changes the list without copying paths or entering ports.

## HTTP and stream transport

On macOS 14 and later, WebKit receives a per-domain HTTP CONNECT configuration
for preview hostnames, avoiding older macOS DNS behavior without system changes.
The CONNECT endpoint admits only known preview names at the proxy port, and its
tunnels share bounded limits and account cancellation. Other website traffic
retains normal routing.

The loopback proxy validates the Host against its catalog, then opens an opaque
service ID through a transport-independent multiplexer. Locally, framed streams
travel over a socket pair. Remotely, the identical frames travel over one ordered,
reliable WebRTC DataChannel. Remote metadata never authorizes an arbitrary TCP
address: the hosting device resolves only its own live service IDs.

Frames have a one-byte kind, a big-endian 32-bit stream ID, and a bounded payload.
Kinds are OPEN, DATA, END, CANCEL, WS_OPEN, WS_DATA, WS_CLOSE, READY and CREDIT.
Socket transport prefixes frames with a big-endian 32-bit length; DataChannel
messages already provide framing. Peers allocate opposite stream-ID parity.
DATA payloads are at most 8 KiB. Each stream has a 64 KiB receive window; connection
queues and the SCTP send buffer are bounded. Up to 64 streams share a connection.
END half-closes; CANCEL tears down both directions. Graceful shutdown drains the
last buffered bytes, whereas dropping an unfinished request cancels promptly.

Hyper streams request and response bodies. The proxy removes hop-by-hop headers,
preserves Host and Origin together (including Next.js Server Actions), sets
X-Forwarded-Host, and rewrites absolute localhost redirects. Set-Cookie and other
end-to-end headers remain intact. WebSocket upgrades retain their byte stream,
subprotocol and close handshake, allowing Vite HMR through the same URL.

## Validation

`cargo test --locked -p space-preview` covers real process/cwd isolation, non-HTTP
exclusion, live disappearance, persistent aliases, port changes, concurrent
streams, slow readers, cancellation, large bodies, streaming HTTP headers and
redirects, WebSocket traffic, and a real WebRTC pair in both directions.

CI runs networking tests on Linux and macOS.

Build `cargo build -p space-ui --example preview-fixture --features browser-fixture`.
Run the fixture with an output directory, an available display and `VITE_BINARY`
pointing to an installed `vite/bin/vite.js`. It starts real Vite/API processes in
an isolated project, discovers them through daemon RPC and waits for a native
click on Vite's Open button. It then verifies HMR, disappearance and a port-change
restart while capturing the native UI. Screenshots/videos belong in PR user
attachments, not the repository.
