## 2024-11-20 - Sensitive Information Exposure in Logs (CWE-532)
**Vulnerability:** The orchestrator was logging full command arguments at INFO level in `orchestrator/src/mcp/transport.rs`. This meant that if a user passed secrets (like API keys) as command-line arguments to an MCP server (e.g., `npx server --api-key SECRET`), these secrets would be written to the application logs, which are often stored insecurely or accessible to unauthorized personnel.
**Learning:** Even "informational" logging can be a security vulnerability if it includes user-controlled data that might contain secrets. Developers often overlook this when debugging or monitoring process spawns.
**Prevention:** Always sanitize or redact arguments in logs. Use DEBUG level for potentially sensitive data, and ensure production logs are filtered to INFO or higher. Alternatively, implement specific redaction for known sensitive arguments, though this is error-prone. The safest default is to log only the command name and the number of arguments at INFO level.

## 2024-11-20 - [Insecure VM Root Filesystem Configuration]
**Vulnerability:** The Firecracker VM was configured with a writable root filesystem (`is_read_only: false`) pointing to a shared resource (`./resources/rootfs.ext4`).
**Learning:** Default configurations often prioritize convenience (writable) over security (isolation). In a multi-tenant or ephemeral VM environment, shared writable resources create race conditions and persistence risks.
**Prevention:** Always default to immutable infrastructure. Explicitly configure read-only access for shared resources. Use copy-on-write (e.g., overlayfs) if writes are needed.
