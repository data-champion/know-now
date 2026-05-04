//! Graph diffing and change classification crate for know-now.
//!
//! Compares two `GeneratorContract` snapshots and produces a typed
//! change set with classifications per LIFE-003.

use std::collections::BTreeMap;

use know_now_contract::contract::{
    ContractAttribute, ContractEntity, ContractRelationship, GeneratorContract,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
    Renamed,
    Compatible,
    Breaking,
    Destructive,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Entity,
    Attribute,
    Relationship,
}

#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub object_kind: ObjectKind,
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub details: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffResult {
    pub schema_version: String,
    pub changes: Vec<Change>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DiffSummary {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub renamed: usize,
    pub compatible: usize,
    pub breaking: usize,
    pub destructive: usize,
    pub ambiguous: usize,
}

impl DiffSummary {
    fn count(&mut self, kind: ChangeKind) {
        match kind {
            ChangeKind::Added => self.added += 1,
            ChangeKind::Removed => self.removed += 1,
            ChangeKind::Modified => self.modified += 1,
            ChangeKind::Renamed => self.renamed += 1,
            ChangeKind::Compatible => self.compatible += 1,
            ChangeKind::Breaking => self.breaking += 1,
            ChangeKind::Destructive => self.destructive += 1,
            ChangeKind::Ambiguous => self.ambiguous += 1,
        }
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.added
            + self.removed
            + self.modified
            + self.renamed
            + self.compatible
            + self.breaking
            + self.destructive
            + self.ambiguous
    }

    #[must_use]
    pub fn has_breaking(&self) -> bool {
        self.breaking > 0 || self.destructive > 0
    }
}

pub fn diff(left: &GeneratorContract, right: &GeneratorContract) -> DiffResult {
    let mut changes = Vec::new();

    diff_entities(&left.entities, &right.entities, &mut changes);
    diff_relationships(&left.relationships, &right.relationships, &mut changes);

    let mut summary = DiffSummary::default();
    for c in &changes {
        summary.count(c.kind);
    }

    DiffResult {
        schema_version: "1".into(),
        changes,
        summary,
    }
}

fn diff_entities(
    left: &[ContractEntity],
    right: &[ContractEntity],
    changes: &mut Vec<Change>,
) {
    let left_by_id: BTreeMap<&str, &ContractEntity> =
        left.iter().map(|e| (e.id.as_str(), e)).collect();
    let right_by_id: BTreeMap<&str, &ContractEntity> =
        right.iter().map(|e| (e.id.as_str(), e)).collect();

    for (id, old) in &left_by_id {
        if let Some(new) = right_by_id.get(id) {
            let details = diff_entity_fields(old, new);

            let attr_changes = diff_attributes(&old.attributes, &new.attributes, &old.id);
            changes.extend(attr_changes);

            if !details.is_empty() {
                let kind = classify_entity_change(&details, old, new);
                changes.push(Change {
                    kind,
                    object_kind: ObjectKind::Entity,
                    id: id.to_string(),
                    name: new.name.clone(),
                    parent_id: None,
                    details,
                });
            }
        } else {
            let kind = if try_heuristic_match_entity(old, &right_by_id).is_some() {
                ChangeKind::Ambiguous
            } else {
                ChangeKind::Removed
            };
            changes.push(Change {
                kind,
                object_kind: ObjectKind::Entity,
                id: id.to_string(),
                name: old.name.clone(),
                parent_id: None,
                details: vec![],
            });
        }
    }

    for (id, new) in &right_by_id {
        if !left_by_id.contains_key(id) {
            let kind = if try_heuristic_match_entity_reverse(new, &left_by_id).is_some() {
                ChangeKind::Ambiguous
            } else {
                ChangeKind::Added
            };
            changes.push(Change {
                kind,
                object_kind: ObjectKind::Entity,
                id: id.to_string(),
                name: new.name.clone(),
                parent_id: None,
                details: vec![],
            });
        }
    }
}

fn diff_entity_fields(old: &ContractEntity, new: &ContractEntity) -> Vec<FieldChange> {
    let mut details = Vec::new();
    if old.name != new.name {
        details.push(FieldChange {
            field: "name".into(),
            old: Some(old.name.clone()),
            new: Some(new.name.clone()),
        });
    }
    diff_opt(old.description.as_ref(), new.description.as_ref(), "description", &mut details);
    diff_opt(old.domain.as_ref(), new.domain.as_ref(), "domain", &mut details);
    diff_opt(old.module.as_ref(), new.module.as_ref(), "module", &mut details);
    diff_opt(old.entity_type.as_ref(), new.entity_type.as_ref(), "type", &mut details);
    diff_opt(old.owner.as_ref(), new.owner.as_ref(), "owner", &mut details);
    diff_opt(old.classification.as_ref(), new.classification.as_ref(), "classification", &mut details);
    diff_opt(
        old.retention_policy.as_ref(),
        new.retention_policy.as_ref(),
        "retention_policy",
        &mut details,
    );
    if old.business_key != new.business_key {
        details.push(FieldChange {
            field: "business_key".into(),
            old: Some(old.business_key.join(",")),
            new: Some(new.business_key.join(",")),
        });
    }
    if old.tags != new.tags {
        details.push(FieldChange {
            field: "tags".into(),
            old: Some(old.tags.join(",")),
            new: Some(new.tags.join(",")),
        });
    }
    details
}

fn classify_entity_change(
    details: &[FieldChange],
    old: &ContractEntity,
    new: &ContractEntity,
) -> ChangeKind {
    let has_name_change = details.iter().any(|d| d.field == "name");
    let has_bk_change = details.iter().any(|d| d.field == "business_key");

    if has_name_change && old.id == new.id {
        return ChangeKind::Renamed;
    }

    if has_bk_change {
        return ChangeKind::Breaking;
    }

    let only_additive = details.iter().all(|d| {
        matches!(d.field.as_str(), "description" | "owner" | "tags" | "classification" | "retention_policy" | "module" | "steward")
    });

    if only_additive {
        ChangeKind::Compatible
    } else {
        ChangeKind::Modified
    }
}

fn diff_attributes(
    left: &[ContractAttribute],
    right: &[ContractAttribute],
    entity_id: &str,
) -> Vec<Change> {
    let mut changes = Vec::new();
    let left_by_id: BTreeMap<&str, &ContractAttribute> =
        left.iter().map(|a| (a.id.as_str(), a)).collect();
    let right_by_id: BTreeMap<&str, &ContractAttribute> =
        right.iter().map(|a| (a.id.as_str(), a)).collect();

    for (id, old) in &left_by_id {
        if let Some(new) = right_by_id.get(id) {
            let details = diff_attribute_fields(old, new);
            if !details.is_empty() {
                let kind = classify_attribute_change(&details, old, new);
                changes.push(Change {
                    kind,
                    object_kind: ObjectKind::Attribute,
                    id: id.to_string(),
                    name: new.name.clone(),
                    parent_id: Some(entity_id.to_owned()),
                    details,
                });
            }
        } else {
            changes.push(Change {
                kind: ChangeKind::Removed,
                object_kind: ObjectKind::Attribute,
                id: id.to_string(),
                name: old.name.clone(),
                parent_id: Some(entity_id.to_owned()),
                details: vec![],
            });
        }
    }

    for (id, new) in &right_by_id {
        if !left_by_id.contains_key(id) {
            changes.push(Change {
                kind: ChangeKind::Added,
                object_kind: ObjectKind::Attribute,
                id: id.to_string(),
                name: new.name.clone(),
                parent_id: Some(entity_id.to_owned()),
                details: vec![],
            });
        }
    }

    changes
}

fn diff_attribute_fields(
    old: &ContractAttribute,
    new: &ContractAttribute,
) -> Vec<FieldChange> {
    let mut details = Vec::new();
    if old.name != new.name {
        details.push(FieldChange {
            field: "name".into(),
            old: Some(old.name.clone()),
            new: Some(new.name.clone()),
        });
    }
    diff_opt(old.logical_type.as_ref(), new.logical_type.as_ref(), "logical_type", &mut details);
    diff_opt(old.semantic_type.as_ref(), new.semantic_type.as_ref(), "semantic_type", &mut details);
    diff_opt_bool(old.required, new.required, "required", &mut details);
    diff_opt_bool(old.is_unique, new.is_unique, "unique", &mut details);
    diff_opt_bool(old.pii, new.pii, "pii", &mut details);
    diff_opt(old.description.as_ref(), new.description.as_ref(), "description", &mut details);
    diff_opt(old.attr_type.as_ref(), new.attr_type.as_ref(), "type", &mut details);
    diff_opt(old.sensitivity.as_ref(), new.sensitivity.as_ref(), "sensitivity", &mut details);
    if old.constraints != new.constraints {
        details.push(FieldChange {
            field: "constraints".into(),
            old: Some(old.constraints.join(",")),
            new: Some(new.constraints.join(",")),
        });
    }
    details
}

fn classify_attribute_change(
    details: &[FieldChange],
    old: &ContractAttribute,
    new: &ContractAttribute,
) -> ChangeKind {
    let has_name_change = details.iter().any(|d| d.field == "name");
    if has_name_change && old.id == new.id {
        return ChangeKind::Renamed;
    }

    let has_type_change = details.iter().any(|d| d.field == "logical_type");
    if has_type_change {
        return ChangeKind::Breaking;
    }

    let nullable_to_required = old.required != Some(true) && new.required == Some(true);
    if nullable_to_required {
        return ChangeKind::Breaking;
    }

    let required_to_nullable = old.required == Some(true) && new.required != Some(true);
    let unique_added = old.is_unique != Some(true) && new.is_unique == Some(true);

    if required_to_nullable || unique_added {
        return ChangeKind::Modified;
    }

    let only_docs = details.iter().all(|d| {
        matches!(d.field.as_str(), "description" | "pii" | "sensitivity")
    });
    if only_docs {
        ChangeKind::Compatible
    } else {
        ChangeKind::Modified
    }
}

fn diff_relationships(
    left: &[ContractRelationship],
    right: &[ContractRelationship],
    changes: &mut Vec<Change>,
) {
    let left_by_id: BTreeMap<&str, &ContractRelationship> =
        left.iter().map(|r| (r.id.as_str(), r)).collect();
    let right_by_id: BTreeMap<&str, &ContractRelationship> =
        right.iter().map(|r| (r.id.as_str(), r)).collect();

    for (id, old) in &left_by_id {
        if let Some(new) = right_by_id.get(id) {
            let details = diff_relationship_fields(old, new);
            if !details.is_empty() {
                let has_endpoint_change = details
                    .iter()
                    .any(|d| d.field == "from_entity" || d.field == "to_entity");
                let kind = if has_endpoint_change {
                    ChangeKind::Breaking
                } else {
                    ChangeKind::Modified
                };
                changes.push(Change {
                    kind,
                    object_kind: ObjectKind::Relationship,
                    id: id.to_string(),
                    name: format!("{} -> {}", new.from_entity, new.to_entity),
                    parent_id: None,
                    details,
                });
            }
        } else {
            changes.push(Change {
                kind: ChangeKind::Destructive,
                object_kind: ObjectKind::Relationship,
                id: id.to_string(),
                name: format!("{} -> {}", old.from_entity, old.to_entity),
                parent_id: None,
                details: vec![],
            });
        }
    }

    for (id, new) in &right_by_id {
        if !left_by_id.contains_key(id) {
            changes.push(Change {
                kind: ChangeKind::Added,
                object_kind: ObjectKind::Relationship,
                id: id.to_string(),
                name: format!("{} -> {}", new.from_entity, new.to_entity),
                parent_id: None,
                details: vec![],
            });
        }
    }
}

fn diff_relationship_fields(
    old: &ContractRelationship,
    new: &ContractRelationship,
) -> Vec<FieldChange> {
    let mut details = Vec::new();
    if old.from_entity != new.from_entity {
        details.push(FieldChange {
            field: "from_entity".into(),
            old: Some(old.from_entity.clone()),
            new: Some(new.from_entity.clone()),
        });
    }
    if old.to_entity != new.to_entity {
        details.push(FieldChange {
            field: "to_entity".into(),
            old: Some(old.to_entity.clone()),
            new: Some(new.to_entity.clone()),
        });
    }
    diff_opt(old.cardinality.as_ref(), new.cardinality.as_ref(), "cardinality", &mut details);
    diff_opt(old.from_key.as_ref(), new.from_key.as_ref(), "from_key", &mut details);
    diff_opt(old.to_key.as_ref(), new.to_key.as_ref(), "to_key", &mut details);
    diff_opt(old.description.as_ref(), new.description.as_ref(), "description", &mut details);
    details
}

fn try_heuristic_match_entity<'a>(
    old: &ContractEntity,
    right_by_id: &BTreeMap<&str, &'a ContractEntity>,
) -> Option<&'a ContractEntity> {
    right_by_id
        .values()
        .find(|new| new.name == old.name && new.id != old.id)
        .copied()
}

fn try_heuristic_match_entity_reverse<'a>(
    new: &ContractEntity,
    left_by_id: &BTreeMap<&str, &'a ContractEntity>,
) -> Option<&'a ContractEntity> {
    left_by_id
        .values()
        .find(|old| old.name == new.name && old.id != new.id)
        .copied()
}

fn diff_opt(
    old: Option<&String>,
    new: Option<&String>,
    field: &str,
    details: &mut Vec<FieldChange>,
) {
    if old != new {
        details.push(FieldChange {
            field: field.into(),
            old: old.cloned(),
            new: new.cloned(),
        });
    }
}

fn diff_opt_bool(
    old: Option<bool>,
    new: Option<bool>,
    field: &str,
    details: &mut Vec<FieldChange>,
) {
    if old != new {
        details.push(FieldChange {
            field: field.into(),
            old: old.map(|b| b.to_string()),
            new: new.map(|b| b.to_string()),
        });
    }
}

pub fn format_text(result: &DiffResult) -> String {
    if result.changes.is_empty() {
        return "No changes detected.".into();
    }

    let mut lines = Vec::new();
    let s = &result.summary;
    lines.push(format!(
        "{} change(s): {} added, {} removed, {} modified, {} renamed, {} compatible, {} breaking, {} destructive, {} ambiguous",
        s.total(), s.added, s.removed, s.modified, s.renamed, s.compatible, s.breaking, s.destructive, s.ambiguous
    ));
    lines.push(String::new());

    for change in &result.changes {
        let prefix = match change.kind {
            ChangeKind::Added => "+",
            ChangeKind::Removed => "-",
            ChangeKind::Modified => "~",
            ChangeKind::Renamed => ">",
            ChangeKind::Compatible => "=",
            ChangeKind::Breaking => "!",
            ChangeKind::Destructive => "X",
            ChangeKind::Ambiguous => "?",
        };
        let obj = match change.object_kind {
            ObjectKind::Entity => "entity",
            ObjectKind::Attribute => "attribute",
            ObjectKind::Relationship => "relationship",
        };
        let parent = change
            .parent_id
            .as_ref()
            .map_or(String::new(), |p| format!(" (in {p})"));
        lines.push(format!(
            "  {prefix} {obj} {id} \"{name}\"{parent}",
            id = change.id,
            name = change.name,
        ));
        for detail in &change.details {
            let old = detail.old.as_deref().unwrap_or("(none)");
            let new = detail.new.as_deref().unwrap_or("(none)");
            lines.push(format!(
                "      {field}: {old} -> {new}",
                field = detail.field,
            ));
        }
    }

    lines.join("\n")
}

pub fn format_json(result: &DiffResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::{ContractProject, ContractTrace};

    fn empty_contract() -> GeneratorContract {
        GeneratorContract {
            contract_version: "1.0".into(),
            project: Some(ContractProject {
                name: "test".into(),
                description: None,
                owner: None,
                tags: vec![],
            }),
            target_database: None,
            entities: vec![],
            relationships: vec![],
            source_systems: vec![],
            quality_rules: vec![],
            governance: None,
            open_questions: vec![],
            assumptions: vec![],
            trace: ContractTrace::default(),
        }
    }

    fn entity(id: &str, name: &str) -> ContractEntity {
        ContractEntity {
            id: id.into(),
            name: name.into(),
            display_name: None,
            domain: None,
            module: None,
            owner: None,
            steward: None,
            classification: None,
            retention_policy: None,
            description: None,
            entity_type: None,
            tags: vec![],
            business_key: vec!["id".into()],
            attributes: vec![],
        }
    }

    fn attr(id: &str, name: &str) -> ContractAttribute {
        ContractAttribute {
            id: id.into(),
            name: name.into(),
            logical_type: Some("string".into()),
            semantic_type: None,
            sensitivity: None,
            pii: None,
            required: None,
            is_unique: None,
            constraints: vec![],
            description: None,
            attr_type: None,
        }
    }

    fn rel(id: &str, from: &str, to: &str) -> ContractRelationship {
        ContractRelationship {
            id: id.into(),
            from_entity: from.into(),
            to_entity: to.into(),
            cardinality: Some("many_to_one".into()),
            from_key: Some("fk_id".into()),
            to_key: Some("id".into()),
            description: None,
        }
    }

    #[test]
    fn no_changes() {
        let c = empty_contract();
        let result = diff(&c, &c);
        assert!(result.changes.is_empty());
        assert_eq!(result.summary.total(), 0);
    }

    #[test]
    fn entity_added() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.added, 1);
        assert_eq!(result.changes[0].kind, ChangeKind::Added);
        assert_eq!(result.changes[0].object_kind, ObjectKind::Entity);
    }

    #[test]
    fn entity_removed() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer"));
        let right = empty_contract();
        let result = diff(&left, &right);
        assert_eq!(result.summary.removed, 1);
        assert_eq!(result.changes[0].kind, ChangeKind::Removed);
    }

    #[test]
    fn entity_renamed() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer"));
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "client"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.renamed, 1);
        assert_eq!(result.changes[0].kind, ChangeKind::Renamed);
    }

    #[test]
    fn entity_description_change_is_compatible() {
        let mut left = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.description = Some("old".into());
        left.entities.push(e);
        let mut right = empty_contract();
        let mut e2 = entity("ent_1", "customer");
        e2.description = Some("new".into());
        right.entities.push(e2);
        let result = diff(&left, &right);
        assert_eq!(result.summary.compatible, 1);
    }

    #[test]
    fn business_key_change_is_breaking() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer"));
        let mut right = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.business_key = vec!["email".into()];
        right.entities.push(e);
        let result = diff(&left, &right);
        assert_eq!(result.summary.breaking, 1);
    }

    #[test]
    fn attribute_added() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer"));
        let mut right = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.attributes.push(attr("attr_1", "email"));
        right.entities.push(e);
        let result = diff(&left, &right);
        assert_eq!(result.summary.added, 1);
        assert_eq!(result.changes[0].object_kind, ObjectKind::Attribute);
        assert_eq!(result.changes[0].parent_id.as_deref(), Some("ent_1"));
    }

    #[test]
    fn attribute_removed() {
        let mut left = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.attributes.push(attr("attr_1", "email"));
        left.entities.push(e);
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.removed, 1);
    }

    #[test]
    fn attribute_type_change_is_breaking() {
        let mut left = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.attributes.push(attr("attr_1", "email"));
        left.entities.push(e);
        let mut right = empty_contract();
        let mut e2 = entity("ent_1", "customer");
        let mut a = attr("attr_1", "email");
        a.logical_type = Some("integer".into());
        e2.attributes.push(a);
        right.entities.push(e2);
        let result = diff(&left, &right);
        assert_eq!(result.summary.breaking, 1);
    }

    #[test]
    fn attribute_nullable_to_required_is_breaking() {
        let mut left = empty_contract();
        let mut e = entity("ent_1", "customer");
        let mut a = attr("attr_1", "email");
        a.required = None;
        e.attributes.push(a);
        left.entities.push(e);
        let mut right = empty_contract();
        let mut e2 = entity("ent_1", "customer");
        let mut a2 = attr("attr_1", "email");
        a2.required = Some(true);
        e2.attributes.push(a2);
        right.entities.push(e2);
        let result = diff(&left, &right);
        assert_eq!(result.summary.breaking, 1);
    }

    #[test]
    fn attribute_description_change_is_compatible() {
        let mut left = empty_contract();
        let mut e = entity("ent_1", "customer");
        let mut a = attr("attr_1", "email");
        a.description = Some("old".into());
        e.attributes.push(a);
        left.entities.push(e);
        let mut right = empty_contract();
        let mut e2 = entity("ent_1", "customer");
        let mut a2 = attr("attr_1", "email");
        a2.description = Some("new".into());
        e2.attributes.push(a2);
        right.entities.push(e2);
        let result = diff(&left, &right);
        assert_eq!(result.summary.compatible, 1);
    }

    #[test]
    fn relationship_added() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.relationships.push(rel("rel_1", "order", "customer"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.added, 1);
        assert_eq!(result.changes[0].object_kind, ObjectKind::Relationship);
    }

    #[test]
    fn relationship_removed_is_destructive() {
        let mut left = empty_contract();
        left.relationships.push(rel("rel_1", "order", "customer"));
        let right = empty_contract();
        let result = diff(&left, &right);
        assert_eq!(result.summary.destructive, 1);
    }

    #[test]
    fn relationship_endpoint_change_is_breaking() {
        let mut left = empty_contract();
        left.relationships.push(rel("rel_1", "order", "customer"));
        let mut right = empty_contract();
        right
            .relationships
            .push(rel("rel_1", "order", "account"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.breaking, 1);
    }

    #[test]
    fn ambiguous_on_id_mismatch_name_match() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_old", "customer"));
        let mut right = empty_contract();
        right.entities.push(entity("ent_new", "customer"));
        let result = diff(&left, &right);
        assert_eq!(result.summary.ambiguous, 2);
    }

    #[test]
    fn text_format_no_changes() {
        let c = empty_contract();
        let result = diff(&c, &c);
        assert_eq!(format_text(&result), "No changes detected.");
    }

    #[test]
    fn text_format_with_changes() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer"));
        let result = diff(&left, &right);
        let text = format_text(&result);
        assert!(text.contains("1 change(s)"));
        assert!(text.contains("+ entity ent_1"));
    }

    #[test]
    fn json_format_roundtrips() {
        let left = empty_contract();
        let mut right = empty_contract();
        right.entities.push(entity("ent_1", "customer"));
        let result = diff(&left, &right);
        let json = format_json(&result);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema_version"], "1");
        assert!(parsed["changes"].is_array());
    }

    #[test]
    fn has_breaking_flag() {
        let mut left = empty_contract();
        left.entities.push(entity("ent_1", "customer"));
        let mut right = empty_contract();
        let mut e = entity("ent_1", "customer");
        e.business_key = vec!["email".into()];
        right.entities.push(e);
        let result = diff(&left, &right);
        assert!(result.summary.has_breaking());
    }

    #[test]
    fn performance_100_entities() {
        let mut left = empty_contract();
        let mut right = empty_contract();
        for i in 0..100 {
            let mut e = entity(&format!("ent_{i}"), &format!("entity_{i}"));
            for j in 0..10 {
                e.attributes
                    .push(attr(&format!("attr_{i}_{j}"), &format!("attr_{j}")));
            }
            left.entities.push(e.clone());
            if i % 10 == 0 {
                let mut modified = e;
                modified.description = Some("changed".into());
                right.entities.push(modified);
            } else {
                right.entities.push(e);
            }
        }
        let start = std::time::Instant::now();
        let result = diff(&left, &right);
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 1000,
            "diff of 100 entities took {}ms (budget: <1000ms)",
            elapsed.as_millis()
        );
        assert_eq!(result.summary.compatible, 10);
    }
}
