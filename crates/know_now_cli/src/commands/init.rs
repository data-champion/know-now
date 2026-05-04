use std::io::{self, BufRead, Write as IoWrite};
use std::path::Path;

use crate::commands::lock::resolve_current_versions;
use crate::context::CommandContext;
use crate::output::{JsonEnvelope, OutputFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Profile {
    Minimal,
    #[value(name = "consultant-postgres-dbt")]
    ConsultantPostgresDbt,
    #[value(name = "dbt-existing-stack")]
    DbtExistingStack,
    #[value(name = "governed-team")]
    GovernedTeam,
    Demo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum GitPolicy {
    Ignore,
    Commit,
    Ask,
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Project name (creates a new directory)
    pub name: Option<String>,

    /// Project profile
    #[arg(long, value_enum)]
    pub profile: Option<Profile>,

    /// Create a demo project (alias for --profile demo)
    #[arg(long, conflicts_with = "profile")]
    pub demo: bool,

    /// Run interactive guided setup
    #[arg(long, conflicts_with_all = ["profile", "demo"])]
    pub guided: bool,

    /// Git policy for generated files
    #[arg(long, value_enum)]
    pub generated_git_policy: Option<GitPolicy>,
}

struct InitConfig {
    name: String,
    profile: Profile,
    git_policy: GitPolicy,
}

pub fn run(ctx: &CommandContext, args: &InitArgs) -> anyhow::Result<()> {
    let config = resolve_config(args)?;
    let project_dir = ctx.project_root.join(&config.name);

    if project_dir.exists() {
        anyhow::bail!("directory '{}' already exists", project_dir.display());
    }

    scaffold(&project_dir, &config)?;
    emit_output(ctx, &config, &project_dir)
}

fn resolve_config(args: &InitArgs) -> anyhow::Result<InitConfig> {
    if args.guided {
        return guided_config();
    }

    let profile = if args.demo {
        Profile::Demo
    } else {
        args.profile.unwrap_or(Profile::Minimal)
    };

    let name = match (&args.name, args.demo) {
        (Some(n), _) => n.clone(),
        (None, true) => "demo-project".to_owned(),
        (None, false) => {
            anyhow::bail!("project name required; usage: know-now init <name> --profile <profile>")
        }
    };

    let git_policy = args
        .generated_git_policy
        .unwrap_or_else(|| default_git_policy(profile));

    Ok(InitConfig {
        name,
        profile,
        git_policy,
    })
}

fn default_git_policy(profile: Profile) -> GitPolicy {
    match profile {
        Profile::Minimal => GitPolicy::Ignore,
        Profile::ConsultantPostgresDbt | Profile::DbtExistingStack => GitPolicy::Ask,
        Profile::GovernedTeam | Profile::Demo => GitPolicy::Commit,
    }
}

fn guided_config() -> anyhow::Result<InitConfig> {
    let stdin = io::stdin();
    let mut lines = stdin.lock();
    let mut out = io::stdout();

    let name = prompt_line(&mut out, &mut lines, "Project name: ")?;
    if name.is_empty() {
        anyhow::bail!("project name cannot be empty");
    }

    let db = prompt_line(
        &mut out,
        &mut lines,
        "Target database (postgres/none) [postgres]: ",
    )?;
    let has_postgres = db.is_empty() || db == "postgres";

    let _policy = prompt_line(&mut out, &mut lines, "Policy pack [dc_standard]: ")?;

    let gp_str = prompt_line(
        &mut out,
        &mut lines,
        "Generated file git policy (ignore/commit/ask) [ask]: ",
    )?;
    drop(lines);

    let git_policy = match gp_str.as_str() {
        "ignore" => GitPolicy::Ignore,
        "commit" => GitPolicy::Commit,
        "" | "ask" => GitPolicy::Ask,
        other => anyhow::bail!("unknown git policy: {other}"),
    };

    let profile = if has_postgres {
        Profile::ConsultantPostgresDbt
    } else {
        Profile::Minimal
    };

    Ok(InitConfig {
        name,
        profile,
        git_policy,
    })
}

fn prompt_line(
    out: &mut io::Stdout,
    reader: &mut io::StdinLock<'_>,
    prompt: &str,
) -> anyhow::Result<String> {
    write!(out, "{prompt}")?;
    out.flush()?;
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    Ok(buf.trim().to_owned())
}

fn scaffold(dir: &Path, config: &InitConfig) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir.join("metadata"))?;
    std::fs::create_dir_all(dir.join("generated"))?;
    std::fs::create_dir_all(dir.join("custom"))?;
    std::fs::create_dir_all(dir.join(".knownow"))?;

    std::fs::write(dir.join("generated/.gitkeep"), "")?;
    std::fs::write(dir.join("custom/.gitkeep"), "")?;

    if config.git_policy == GitPolicy::Ignore {
        std::fs::write(
            dir.join("generated/.gitignore"),
            "*\n!.gitignore\n!.gitkeep\n",
        )?;
    }

    write_config(dir, config)?;
    write_metadata(dir, config)?;

    if config.profile == Profile::Demo {
        let lockfile = resolve_current_versions().to_lockfile();
        lockfile
            .write_to(&dir.join(know_now_lock::LOCKFILE_NAME))
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    write_readme(dir, config)?;
    Ok(())
}

fn write_config(dir: &Path, config: &InitConfig) -> anyhow::Result<()> {
    let pname = profile_name(config.profile);
    let gp = git_policy_name(config.git_policy);

    let target_db = match config.profile {
        Profile::Minimal => "# target_database:\n#   kind: postgres\n#   version: \"16\"",
        _ => "target_database:\n  kind: postgres\n  version: \"16\"",
    };

    let dbt_line = match config.profile {
        Profile::ConsultantPostgresDbt | Profile::DbtExistingStack => "dbt_validation: warn",
        _ => "# dbt_validation: none",
    };

    let content = format!(
        "\
# know-now project configuration
# Generated by: know-now init --profile {pname}

project:
  name: {name}

{target_db}

policy:
  pack: dc_standard
  version: \"1.0\"

{dbt_line}

generated_git_policy: {gp}
",
        name = config.name,
    );

    std::fs::write(dir.join("know-now.yml"), content)?;
    Ok(())
}

fn write_metadata(dir: &Path, config: &InitConfig) -> anyhow::Result<()> {
    match config.profile {
        Profile::Minimal => write_minimal_metadata(dir, &config.name),
        Profile::ConsultantPostgresDbt => write_consultant_metadata(dir, &config.name),
        Profile::DbtExistingStack => write_dbt_existing_metadata(dir, &config.name),
        Profile::GovernedTeam => write_governed_metadata(dir, &config.name),
        Profile::Demo => write_demo_metadata(dir, &config.name),
    }
}

fn write_readme(dir: &Path, config: &InitConfig) -> anyhow::Result<()> {
    let pname = profile_name(config.profile);
    let content = format!(
        "\
# {name}

A know-now metadata project (`{pname}` profile).

## Directory layout

- `know-now.yml` — Project configuration.
- `metadata/` — Authoring metadata (entities, relationships, sources).
- `generated/` — Engine-generated artifacts (do not edit by hand).
- `custom/` — User-maintained files (never overwritten by engine).
- `.knownow/` — Engine working directory.

## Quick start

```sh
know-now validate          # parse and validate metadata
know-now check             # run recommended checks
know-now generate          # generate artifacts
know-now check --locked    # verify lockfile consistency
```
",
        name = config.name,
    );
    std::fs::write(dir.join("README.md"), content)?;
    Ok(())
}

fn emit_output(
    ctx: &CommandContext,
    config: &InitConfig,
    project_dir: &Path,
) -> anyhow::Result<()> {
    match ctx.format {
        OutputFormat::Json => {
            let payload = InitPayload {
                project_name: &config.name,
                profile: profile_name(config.profile),
                git_policy: git_policy_name(config.git_policy),
                path: &project_dir.display().to_string(),
            };
            let envelope = JsonEnvelope::success("init", &payload);
            println!("{}", serde_json::to_string_pretty(&envelope)?);
        }
        OutputFormat::Quiet => {}
        OutputFormat::Text | OutputFormat::Sarif => {
            println!(
                "Created project '{}' at {}",
                config.name,
                project_dir.display()
            );
            println!("  profile: {}", profile_name(config.profile));
            println!(
                "  generated-git-policy: {}",
                git_policy_name(config.git_policy)
            );
        }
    }
    Ok(())
}

fn profile_name(p: Profile) -> &'static str {
    match p {
        Profile::Minimal => "minimal",
        Profile::ConsultantPostgresDbt => "consultant-postgres-dbt",
        Profile::DbtExistingStack => "dbt-existing-stack",
        Profile::GovernedTeam => "governed-team",
        Profile::Demo => "demo",
    }
}

fn git_policy_name(gp: GitPolicy) -> &'static str {
    match gp {
        GitPolicy::Ignore => "ignore",
        GitPolicy::Commit => "commit",
        GitPolicy::Ask => "ask",
    }
}

#[derive(serde::Serialize)]
struct InitPayload<'a> {
    project_name: &'a str,
    profile: &'a str,
    git_policy: &'a str,
    path: &'a str,
}

// ---------------------------------------------------------------------------
// Metadata templates
// ---------------------------------------------------------------------------

fn write_minimal_metadata(dir: &Path, name: &str) -> anyhow::Result<()> {
    let project = format!(
        "\
version: \"1.0\"
project:
  name: {name}
  description: A know-now metadata project.
  owner: data-team
entities:
  - id: ent_example
    name: example
    description: An example entity. Replace with your own.
    business_key: [example_id]
    attributes:
      - id: attr_example_example_id
        name: example_id
        logical_type: integer
        required: true
        unique: true
        description: Primary identifier.
      - id: attr_example_name
        name: name
        logical_type: string
        description: Display name.
"
    );
    std::fs::write(dir.join("metadata/project.yml"), project)?;
    Ok(())
}

fn write_consultant_metadata(dir: &Path, name: &str) -> anyhow::Result<()> {
    let project = format!(
        "\
version: \"1.0\"
project:
  name: {name}
  description: A know-now metadata project.
  owner: data-team
  tags:
    - postgres
    - dbt
target_database:
  kind: postgres
  version: \"16\"
policy:
  pack: dc_standard
  version: \"1.0\"
domains:
  - id: dom_core
    name: core
    description: Core business domain.
modules:
  - id: mod_staging
    name: staging
    description: Staging models.
"
    );
    std::fs::write(dir.join("metadata/project.yml"), project)?;

    std::fs::write(dir.join("metadata/entities.yml"), CONSULTANT_ENTITIES)?;
    Ok(())
}

fn write_dbt_existing_metadata(dir: &Path, name: &str) -> anyhow::Result<()> {
    let project = format!(
        "\
version: \"1.0\"
project:
  name: {name}
  description: A know-now metadata project for an existing dbt stack.
  owner: data-team
  tags:
    - postgres
    - dbt
    - existing-stack
target_database:
  kind: postgres
  version: \"16\"
policy:
  pack: dc_standard
  version: \"1.0\"
domains:
  - id: dom_core
    name: core
    description: Core business domain.
modules:
  - id: mod_staging
    name: staging
    description: Staging models.
"
    );
    std::fs::write(dir.join("metadata/project.yml"), project)?;

    std::fs::write(dir.join("metadata/entities.yml"), CONSULTANT_ENTITIES)?;
    Ok(())
}

fn write_governed_metadata(dir: &Path, name: &str) -> anyhow::Result<()> {
    let project = format!(
        "\
version: \"1.0\"
project:
  name: {name}
  description: A governed know-now metadata project.
  owner: data-team
  tags:
    - postgres
    - governed
target_database:
  kind: postgres
  version: \"16\"
policy:
  pack: dc_standard
  version: \"1.0\"
domains:
  - id: dom_sales
    name: sales
    description: Sales and commercial domain.
  - id: dom_operations
    name: operations
    description: Operations and logistics.
modules:
  - id: mod_core
    name: core
    description: Core business models.
  - id: mod_reporting
    name: reporting
    description: Reporting and analytics.
governance:
  data_owner: data-team
  data_steward: data-governance
  classification_default: internal
  retention_default: 5y
"
    );
    std::fs::write(dir.join("metadata/project.yml"), project)?;

    std::fs::write(dir.join("metadata/entities.yml"), GOVERNED_ENTITIES)?;
    Ok(())
}

fn write_demo_metadata(dir: &Path, name: &str) -> anyhow::Result<()> {
    let project = format!(
        "\
version: \"1.0\"
project:
  name: {name}
  description: Demo e-commerce data model generated by know-now init --demo.
  owner: data-team
  tags:
    - demo
    - ecommerce
target_database:
  kind: postgres
  version: \"16\"
policy:
  pack: dc_standard
  version: \"1.0\"
domains:
  - id: dom_sales
    name: sales
    description: Customer and order management.
  - id: dom_catalog
    name: catalog
    description: Product catalog and inventory.
modules:
  - id: mod_core
    name: core
    description: Core business entities.
governance:
  data_owner: data-team
  data_steward: data-governance
  classification_default: internal
  retention_default: 5y
"
    );
    std::fs::write(dir.join("metadata/project.yml"), project)?;

    std::fs::write(dir.join("metadata/entities.yml"), DEMO_ENTITIES)?;
    std::fs::write(dir.join("metadata/relationships.yml"), DEMO_RELATIONSHIPS)?;
    std::fs::write(dir.join("metadata/sources.yml"), DEMO_SOURCES)?;
    Ok(())
}

const CONSULTANT_ENTITIES: &str = "\
version: \"1.0\"
entities:
  - id: ent_customer
    name: customer
    description: A customer record.
    domain: dom_core
    module: mod_staging
    type: dimension
    business_key: [customer_id]
    attributes:
      - id: attr_customer_customer_id
        name: customer_id
        logical_type: integer
        required: true
        unique: true
        description: Primary customer identifier.
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        description: Customer email address.
      - id: attr_customer_name
        name: full_name
        logical_type: string
        required: true
        description: Customer display name.
      - id: attr_customer_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
";

const GOVERNED_ENTITIES: &str = "\
version: \"1.0\"
entities:
  - id: ent_customer
    name: customer
    description: A person or organization that places orders.
    domain: dom_sales
    module: mod_core
    owner: sales-team
    steward: data-governance
    classification: internal
    type: dimension
    business_key: [customer_id]
    attributes:
      - id: attr_customer_customer_id
        name: customer_id
        logical_type: uuid
        required: true
        unique: true
        description: Stable customer identifier.
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        required: true
        description: Customer email address.
      - id: attr_customer_name
        name: full_name
        logical_type: string
        required: true
        description: Customer display name.
      - id: attr_customer_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
  - id: ent_order
    name: order
    description: A commercial order transaction.
    domain: dom_sales
    module: mod_core
    owner: sales-team
    steward: finance-team
    classification: internal
    type: fact
    business_key: [order_id]
    attributes:
      - id: attr_order_order_id
        name: order_id
        logical_type: uuid
        required: true
        unique: true
        description: Stable order identifier.
      - id: attr_order_customer_id
        name: customer_id
        logical_type: uuid
        required: true
        description: Owning customer reference.
      - id: attr_order_total_amount
        name: total_amount
        logical_type: decimal
        required: true
        description: Total order amount.
      - id: attr_order_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_key: customer_id
    to_key: customer_id
    description: Each order belongs to one customer.
";

const DEMO_ENTITIES: &str = "\
version: \"1.0\"
entities:
  - id: ent_customer
    name: customer
    description: A person or organization that places orders.
    domain: dom_sales
    module: mod_core
    owner: sales-team
    steward: data-governance
    classification: internal
    type: dimension
    tags: [core, commercial]
    business_key: [customer_id]
    attributes:
      - id: attr_customer_customer_id
        name: customer_id
        logical_type: uuid
        required: true
        unique: true
        description: Stable customer identifier.
      - id: attr_customer_email
        name: email
        logical_type: string
        semantic_type: email
        required: true
        description: Primary customer email address.
      - id: attr_customer_phone
        name: phone
        logical_type: string
        semantic_type: phone
        description: Customer phone number.
      - id: attr_customer_name
        name: full_name
        logical_type: string
        required: true
        description: Full customer display name.
      - id: attr_customer_status
        name: status
        logical_type: string
        required: true
        description: Customer lifecycle status (active, churned, prospect).
      - id: attr_customer_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
      - id: attr_customer_updated_at
        name: updated_at
        logical_type: timestamp
        required: true
        description: UTC last-update timestamp.

  - id: ent_order
    name: order
    description: Commercial order transaction.
    domain: dom_sales
    module: mod_core
    owner: sales-team
    steward: finance-team
    classification: internal
    type: fact
    tags: [core, finance]
    business_key: [order_id]
    attributes:
      - id: attr_order_order_id
        name: order_id
        logical_type: uuid
        required: true
        unique: true
        description: Stable order identifier.
      - id: attr_order_customer_id
        name: customer_id
        logical_type: uuid
        required: true
        description: Owning customer identifier.
      - id: attr_order_total_amount
        name: total_amount
        logical_type: decimal
        required: true
        description: Total order amount in transaction currency.
      - id: attr_order_status
        name: order_status
        logical_type: string
        required: true
        description: Order processing status.
      - id: attr_order_submitted_date
        name: submitted_date
        logical_type: date
        required: true
        description: Calendar date when order was submitted.
      - id: attr_order_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
      - id: attr_order_updated_at
        name: updated_at
        logical_type: timestamp
        required: true
        description: UTC last-update timestamp.

  - id: ent_product
    name: product
    description: Sellable product definition.
    domain: dom_catalog
    module: mod_core
    owner: catalog-team
    classification: internal
    type: dimension
    tags: [catalog]
    business_key: [product_id]
    attributes:
      - id: attr_product_product_id
        name: product_id
        logical_type: uuid
        required: true
        unique: true
        description: Stable product identifier.
      - id: attr_product_name
        name: product_name
        logical_type: string
        required: true
        description: Product display name.
      - id: attr_product_category
        name: category_code
        logical_type: string
        required: true
        description: Product category code.
      - id: attr_product_unit_price
        name: unit_price
        logical_type: decimal
        required: true
        description: Standard catalog price.
      - id: attr_product_status
        name: product_status
        logical_type: string
        required: true
        description: Product lifecycle state (active, discontinued).
      - id: attr_product_created_at
        name: created_at
        logical_type: timestamp
        required: true
        description: UTC creation timestamp.
      - id: attr_product_updated_at
        name: updated_at
        logical_type: timestamp
        required: true
        description: UTC last-update timestamp.
";

const DEMO_RELATIONSHIPS: &str = "\
version: \"1.0\"
relationships:
  - id: rel_order_customer
    from_entity: order
    to_entity: customer
    cardinality: many_to_one
    from_key: customer_id
    to_key: customer_id
    description: Each order belongs to one customer.
";

const DEMO_SOURCES: &str = "\
version: \"1.0\"
sources:
  - name: crm
    kind: application
    description: CRM system (Shopify, HubSpot, etc.)
    tables:
      - name: customers
        entity: customer
        columns:
          - source: id
            target: customer_id
          - source: email
            target: email
          - source: full_name
            target: full_name
          - source: created_at
            target: created_at
      - name: orders
        entity: order
        columns:
          - source: id
            target: order_id
          - source: customer_id
            target: customer_id
          - source: total_price
            target: total_amount
          - source: created_at
            target: created_at

open_questions:
  - id: q_customer_ltv
    question: Should customer lifetime value include refunded orders?
    entity: customer
    priority: high

assumptions:
  - id: asm_single_currency
    statement: Order totals are stored in transaction currency, not normalized.
    entity: order
    risk: medium
";
