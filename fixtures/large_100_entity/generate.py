#!/usr/bin/env python3
"""Generate a 100-entity fixture for know-now compatibility testing.

Usage:
    python3 generate.py > metadata/project.yml
"""

import random

LOGICAL_TYPES = [
    "integer", "string", "boolean", "decimal", "date",
    "timestamp", "uuid", "text", "bigint", "float",
]

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
]

ATTR_SUFFIXES = [
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


def main():
    random.seed(42)  # deterministic output

    lines = ['version: "1.0"', "entities:"]

    for ent_name in ENTITY_NAMES[:100]:
        ent_id = f"ent_{ent_name}"
        lines.append(f"  - id: {ent_id}")
        lines.append(f"    name: {ent_name}")

        num_attrs = random.randint(3, 5)
        chosen = random.sample(ATTR_SUFFIXES, num_attrs)
        # Ensure an 'id' attribute is always present
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

    print("\n".join(lines))


if __name__ == "__main__":
    main()
