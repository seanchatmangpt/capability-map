# Tenancy and Authorization

## Current State (Stub)

Tenant isolation is implemented as namespace scoping. Each tenant gets:

- A `urn:cpmp:tenant:<name>:` IRI namespace prefix for their scan results
- A metadata file at `~/.cpmp/tenants/<name>.json`

## CLI

```bash
cpmp tenant create <name>
cpmp tenant list
```

## Future Implementation

Full enterprise tenancy requires:

1. **OIDC/SAML identity boundary** — token validation at the CLI boundary
2. **Tenant-scoped graph namespaces** — every quad written to a named graph `<urn:cpmp:tenant:<name>>`
3. **Workspace-scoped scan roots** — tenants only scan their approved root paths
4. **Role model**:
   - `reader` — can query graphs and run `policy check`
   - `scanner` — can run `computer discover`
   - `publisher` — can load and version graphs in Open Ontologies
   - `approver` — can approve policy exceptions
   - `auditor` — can view lineage, receipts, and version history
   - `admin` — can create tenants and manage service accounts

## Authorization

```bash
# Future: token-gated commands
CPMP_TOKEN=<oidc_token> cpmp computer discover ~/my-project
```

All tokens would be validated against the tenant's JWKS endpoint before any scan or load operation.
