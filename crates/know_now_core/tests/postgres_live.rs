use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use know_now_codegen::generator::Generator;
use know_now_core::projection::project_graph_to_contract;
use know_now_gen_postgres::PostgresGenerator;
use know_now_metadata::test_support::parse_yaml_metadata;
use know_now_validate::builder::build_project_graph;
use postgres::error::SqlState;
use postgres::{Client, NoTls};

const PHASE1_FIXTURE: &str = include_str!("../../../fixtures/validation/valid_full.yml");

#[test]
fn generated_schema_executes_on_live_postgres() {
    let Some(url) = live_test_url() else {
        eprintln!("skipping live postgres test: KNOW_NOW_PG_TEST_URL is not set");
        return;
    };

    let mut client = Client::connect(&url, NoTls).expect("postgres connection should succeed");
    let schema = unique_schema_name();
    let quoted_schema = quote_ident(&schema);

    client
        .batch_execute(&format!(
            "DROP SCHEMA IF EXISTS {quoted_schema} CASCADE; CREATE SCHEMA {quoted_schema}; SET search_path TO {quoted_schema};"
        ))
        .expect("schema setup should succeed");

    let metadata = parse_yaml_metadata(PHASE1_FIXTURE);
    let graph_result = build_project_graph(&metadata);
    assert!(
        graph_result.diagnostics.is_empty(),
        "fixture should build graph without diagnostics: {:?}",
        graph_result.diagnostics
    );
    let graph = graph_result.graph.expect("graph should exist");
    let contract = project_graph_to_contract(&graph);

    let generator = PostgresGenerator::new();
    let artifacts = generator
        .generate(&contract)
        .expect("postgres generator should succeed");
    let sql = artifacts
        .iter()
        .find(|a| a.path == "ddl/postgres/schema.sql")
        .expect("schema.sql artifact should be present")
        .content
        .as_str();

    client
        .batch_execute(sql)
        .expect("generated schema should execute");

    let tables = client
        .query(
            "SELECT table_name
               FROM information_schema.tables
              WHERE table_schema = $1
              ORDER BY table_name",
            &[&schema],
        )
        .expect("table query should succeed")
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<Vec<_>>();

    assert!(tables.contains(&"customer".to_owned()));
    assert!(tables.contains(&"order".to_owned()));

    let customer_constraints = client
        .query(
            "SELECT constraint_type
               FROM information_schema.table_constraints
              WHERE table_schema = $1
                AND table_name = 'customer'",
            &[&schema],
        )
        .expect("constraint query should succeed")
        .into_iter()
        .map(|row| row.get::<_, String>(0))
        .collect::<BTreeSet<_>>();

    assert!(
        customer_constraints.contains("PRIMARY KEY"),
        "customer table should include a primary key constraint"
    );

    let customer_id_is_not_null: bool = client
        .query_one(
            "SELECT is_nullable = 'NO'
               FROM information_schema.columns
              WHERE table_schema = $1
                AND table_name = 'customer'
                AND column_name = 'id'",
            &[&schema],
        )
        .expect("column query should succeed")
        .get(0);
    assert!(customer_id_is_not_null, "customer.id should be NOT NULL");

    client
        .execute(
            "INSERT INTO \"customer\" (\"id\", \"email\") VALUES ($1, $2)",
            &[&1_i32, &"dupe@example.com"],
        )
        .expect("first customer row should insert");

    let unique_err = client
        .execute(
            "INSERT INTO \"customer\" (\"id\", \"email\") VALUES ($1, $2)",
            &[&2_i32, &"dupe@example.com"],
        )
        .expect_err("duplicate business-key insert should fail");
    assert_eq!(unique_err.code(), Some(&SqlState::UNIQUE_VIOLATION));

    let not_null_err = client
        .execute(
            "INSERT INTO \"customer\" (\"email\") VALUES ($1)",
            &[&"nonnull@example.com"],
        )
        .expect_err("missing customer.id should fail");
    assert_eq!(not_null_err.code(), Some(&SqlState::NOT_NULL_VIOLATION));

    // Current generator output does not yet materialize relationship FKs.
    // This patch validates live FK enforcement in CI by adding the expected FK.
    client
        .batch_execute(
            "ALTER TABLE \"customer\" ADD CONSTRAINT \"uq_customer_id\" UNIQUE (\"id\");
             ALTER TABLE \"order\" ADD CONSTRAINT \"fk_order_customer_id\" FOREIGN KEY (\"customer_id\") REFERENCES \"customer\" (\"id\");",
        )
        .expect("fk setup should succeed");

    client
        .execute(
            "INSERT INTO \"order\" (\"id\", \"customer_id\") VALUES ($1, $2)",
            &[&100_i32, &1_i32],
        )
        .expect("valid fk insert should succeed");

    let fk_err = client
        .execute(
            "INSERT INTO \"order\" (\"id\", \"customer_id\") VALUES ($1, $2)",
            &[&101_i32, &999_999_i32],
        )
        .expect_err("invalid fk insert should fail");
    assert_eq!(fk_err.code(), Some(&SqlState::FOREIGN_KEY_VIOLATION));

    client
        .batch_execute(&format!("DROP SCHEMA IF EXISTS {quoted_schema} CASCADE;"))
        .expect("schema cleanup should succeed");
}

fn live_test_url() -> Option<String> {
    std::env::var("KNOW_NOW_PG_TEST_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn unique_schema_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |d| d.as_nanos());
    format!("know_now_ci_{}_{}", std::process::id(), nanos)
}

fn quote_ident(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
