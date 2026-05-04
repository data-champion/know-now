#!/usr/bin/env python3
"""Generate a 200-entity, 1000-relationship stress fixture for know-now.

Usage:
    cd fixtures/stress_large
    python3 generate.py

Produces metadata/entities.yml and metadata/relationships.yml.
Deterministic via fixed seed.
"""

import os
import random

LOGICAL_TYPES = [
    "integer", "string", "boolean", "decimal", "date",
    "timestamp", "uuid", "text", "bigint", "float",
]

DOMAINS = ["sales", "operations", "finance", "hr", "logistics",
           "marketing", "engineering", "support", "analytics", "compliance"]

ENTITY_NAMES = [
    "account", "transaction", "product", "customer", "invoice",
    "payment", "shipment", "warehouse", "employee", "department",
    "project", "task", "ticket", "event", "session",
    "campaign", "channel", "region", "store", "supplier",
    "category", "tag", "comment", "review", "rating",
    "subscription", "plan", "feature", "license", "contract",
    "budget", "expense", "revenue", "forecast", "target",
    "metric", "report", "dashboard", "alert", "notification",
    "message", "thread", "group", "role", "permission",
    "audit_log", "changelog", "snapshot", "archive", "backup",
    "config", "setting", "preference", "template", "workflow",
    "approval", "request", "response", "feedback", "survey",
    "question", "answer", "quiz", "course", "lesson",
    "student", "instructor", "grade", "certificate", "badge",
    "achievement", "reward", "coupon", "discount", "promotion",
    "offer", "deal", "lead", "opportunity", "pipeline",
    "stage", "milestone", "deliverable", "artifact", "asset",
    "resource", "allocation", "schedule", "calendar", "booking",
    "reservation", "venue", "room", "equipment", "inventory",
    "stock", "order_line", "return_item", "refund", "credit",
    "debit", "transfer", "reconciliation", "journal", "ledger",
    "batch", "queue", "worker", "job", "result",
    "schema", "index", "partition", "replica", "shard",
    "tenant", "org", "team", "member", "invite",
    "token", "scope", "policy", "rule", "condition",
    "action", "trigger", "handler", "middleware", "plugin",
    "registry", "catalog", "manifest", "release", "version",
    "patch", "hotfix", "branch", "commit", "diff",
    "pull_request", "merge", "conflict", "resolution", "rollback",
    "deployment", "environment", "cluster", "node", "pod",
    "service", "endpoint", "route", "gateway", "proxy",
    "cache", "buffer", "stream", "sink", "source",
    "connector", "adapter", "bridge", "transformer", "enricher",
    "validator", "sanitizer", "encoder", "decoder", "compressor",
    "encryptor", "signer", "verifier", "authenticator", "authorizer",
    "scheduler", "dispatcher", "collector", "aggregator", "sampler",
    "profiler", "tracer", "monitor", "observer", "inspector",
    "analyzer", "classifier", "predictor", "recommender", "ranker",
    "scorer", "indexer", "crawler", "scraper", "parser",
    "formatter", "renderer", "presenter", "exporter", "importer",
]

ATTR_TEMPLATES = [
    ("id", "integer", True),
    ("name", "string", True),
    ("code", "string", False),
    ("status", "string", True),
    ("description", "text", False),
    ("amount", "decimal", False),
    ("count", "integer", False),
    ("flag", "boolean", False),
    ("date", "date", False),
    ("created_at", "timestamp", True),
    ("updated_at", "timestamp", False),
    ("email", "string", False),
    ("reference", "uuid", False),
    ("score", "float", False),
    ("quantity", "integer", False),
]


def generate_domains() -> str:
    lines = ["domains:"]
    for domain in DOMAINS:
        lines.append(f"  - id: {domain}")
        lines.append(f"    name: {domain}")
    return "\n".join(lines) + "\n"


def generate_entities(count: int, rng: random.Random) -> str:
    names = ENTITY_NAMES[:count]
    lines = ["entities:"]

    for i, ent_name in enumerate(names):
        ent_id = f"ent_{ent_name}"
        domain = DOMAINS[i % len(DOMAINS)]

        lines.append(f"  - id: {ent_id}")
        lines.append(f"    name: {ent_name}")
        lines.append(f"    domain: {domain}")
        lines.append(f"    business_key: [id]")

        num_attrs = rng.randint(3, 8)
        chosen = rng.sample(ATTR_TEMPLATES, min(num_attrs, len(ATTR_TEMPLATES)))
        if not any(s[0] == "id" for s in chosen):
            chosen[0] = ("id", "integer", True)

        lines.append("    attributes:")
        for suffix, ltype, req in chosen:
            attr_name = suffix if suffix == "id" else f"{ent_name}_{suffix}"
            attr_id = f"attr_{ent_name}_{suffix}"
            lines.append(f"      - id: {attr_id}")
            lines.append(f"        name: {attr_name}")
            lines.append(f"        logical_type: {ltype}")
            if req:
                lines.append("        required: true")

    return "\n".join(lines) + "\n"


def generate_relationships(entity_count: int, rel_count: int, rng: random.Random) -> str:
    names = ENTITY_NAMES[:entity_count]
    lines = ["relationships:"]

    cardinalities = ["many_to_one", "one_to_many", "many_to_many"]
    used_pairs = set()

    for i in range(rel_count):
        from_idx = rng.randint(0, entity_count - 1)
        to_idx = rng.randint(0, entity_count - 1)
        while to_idx == from_idx or (from_idx, to_idx) in used_pairs:
            from_idx = rng.randint(0, entity_count - 1)
            to_idx = rng.randint(0, entity_count - 1)
        used_pairs.add((from_idx, to_idx))

        from_name = names[from_idx]
        to_name = names[to_idx]
        card = rng.choice(cardinalities)
        rel_id = f"rel_{from_name}_{to_name}"

        lines.append(f"  - id: {rel_id}")
        lines.append(f"    from_entity: {from_name}")
        lines.append(f"    to_entity: {to_name}")
        lines.append(f"    cardinality: {card}")

    return "\n".join(lines) + "\n"


def main():
    rng = random.Random(42)
    entity_count = 200
    rel_count = 1000

    os.makedirs("metadata", exist_ok=True)

    domains_yaml = generate_domains()
    with open("metadata/domains.yml", "w") as f:
        f.write(domains_yaml)

    entities_yaml = generate_entities(entity_count, rng)
    with open("metadata/entities.yml", "w") as f:
        f.write(entities_yaml)

    relationships_yaml = generate_relationships(entity_count, rel_count, rng)
    with open("metadata/relationships.yml", "w") as f:
        f.write(relationships_yaml)

    print(f"Generated {entity_count} entities and {rel_count} relationships")


if __name__ == "__main__":
    main()
