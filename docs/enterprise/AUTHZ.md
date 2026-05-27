# Authorization

## Current State

Authorization is implemented at the **CLI invocation boundary** only. No token validation. Access = filesystem access.

## Planned Role Model

| Role | Commands Allowed | Grants |
|------|-----------------|--------|
| `reader` | `graph query`, `policy check`, `tenant list`, `audit lineage` | Read-only access |
| `scanner` | All reader commands + `computer discover` | Scan new paths |
| `publisher` | All scanner commands + `graph load`, `graph version` | Write to Open Ontologies store |
| `approver` | All publisher commands + approval commands | Approve policy exceptions |
| `auditor` | All reader commands + receipt verification | View full audit trail |
| `admin` | All commands + `tenant create` | Full control |

## Token Enforcement (Planned)

```bash
CPMP_TOKEN=<oidc_token> cpmp computer discover ~/my-project
```

The token would be:
1. Validated against the tenant's JWKS endpoint
2. Decoded to extract claims (tenant, role, workspace)
3. Checked against the resource's tenant scope before scan begins

## Service Accounts

Future: machine-issued tokens for CI pipelines:

```toml
# ~/.cpmp/config.toml
[auth]
token_url = "https://sso.example.com/token"
client_id = "cpmp-ci"
client_secret_env = "CPMP_CLIENT_SECRET"
```
