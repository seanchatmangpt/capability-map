# Public Vocabulary Firewall

## Purpose

The public vocabulary firewall prevents private project terms from crossing into the Open Ontologies store with false authority.

## Allowed Vocabularies (Public)

| Prefix | Namespace | Used For |
|--------|-----------|----------|
| `prov:` | `http://www.w3.org/ns/prov#` | Activity, wasGeneratedBy, startedAtTime |
| `dcat:` | `http://www.w3.org/ns/dcat#` | Catalog, dataset, distribution |
| `doap:` | `http://usefulinc.com/ns/doap#` | Project, Repository |
| `spdx:` | `http://spdx.org/rdf/terms#` | File, Checksum, checksumValue, algorithm |
| `skos:` | `http://www.w3.org/2004/02/skos/core#` | Concept, prefLabel, notation |
| `dcterms:` | `http://purl.org/dc/terms/` | identifier, issued, description, format |
| `sh:` | `http://www.w3.org/ns/shacl#` | NodeShape, property, path, minCount |
| `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | type |
| `xsd:` | `http://www.w3.org/2001/XMLSchema#` | dateTime, integer, string |
| `owl:` | `http://www.w3.org/2002/07/owl#` | Class, ObjectProperty, DatatypeProperty |

## Allowed Private Namespaces

Private namespaces are allowed ONLY for **instance IRIs** (subjects), not predicates:

- `urn:cpmp:catalog:<id>` — catalog instances
- `urn:cpmp:scan:<id>` — scan activity instances
- `urn:cpmp:checksum:<hash>` — checksum instances
- `urn:cpmp:capability:<name>` — capability instances
- `urn:cpmp:project:local` — project instance
- `urn:cpmp:tenant:<name>` — tenant namespace instances

## Forbidden

| Pattern | Reason |
|---------|--------|
| `gall:` predicates | Private namespace laundered as authority |
| `ggen:status` | Private term in authority position |
| Any `ggen:` or `gall:` predicate carrying classification authority | Use `skos:notation` instead |
| Hand-written Turtle without IRI validation | All IRIs must pass `NamedNode::new()` gate |

## Enforcement

The firewall is enforced at three layers:

1. **Construction time**: `NamedNode::new(iri)?` — rejects invalid IRIs before they enter the store
2. **Policy pack**: `no-private-predicate-authority` — scans Turtle content for known violations
3. **Open Ontologies load**: Structural validation on load catches escaped violations
