# Retention and Backup

## Retention Policy (Stub)

Default retention: **unlimited** (all receipts and reports retained).

Planned:

| Asset | Default Retention |
|-------|------------------|
| Scan receipts | 90 days |
| Graph version snapshots | 30 named versions |
| Reports | Until superseded by a newer hash |
| Audit trail | Permanent |

## Backup Paths

The Open Ontologies store can be backed up using:

```bash
open-ontologies save <output.ttl>   # export current graph to Turtle
```

Future `cpmp-enterprise-backup` module will:

1. Call `open-ontologies save` on a schedule
2. Verify the saved Turtle parses cleanly
3. Archive to a configurable destination (local path, S3, GCS)
4. Emit a backup receipt with BLAKE3 hash of the saved file

## Recovery

```bash
open-ontologies history             # list available snapshots
open-ontologies rollback <version>  # restore named version
```

Or restore from a saved Turtle:

```bash
open-ontologies load <backup.ttl>
```

## Current Status

- **Receipt retention**: All receipts written to `<out>/receipts/` — no auto-deletion
- **Graph backup**: Manual via `open-ontologies save` — no scheduled backup yet
- **Module status**: `cpmp-enterprise-backup` — stub documented, not implemented
