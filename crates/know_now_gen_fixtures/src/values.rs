use know_now_contract::contract::ContractAttribute;

use crate::prng::SeededRng;

const ISO_COUNTRY_CODES: &[&str] = &[
    "US", "GB", "DE", "FR", "NL", "JP", "AU", "CA", "BR", "IN",
    "SE", "NO", "DK", "FI", "ES", "IT", "CH", "AT", "BE", "PT",
];

pub fn generate_pk(attr: &ContractAttribute, rng: &mut SeededRng, row_idx: usize) -> String {
    match attr.logical_type.as_deref() {
        Some("uuid") => synthetic_uuid(rng, row_idx),
        _ => format!("{}", row_idx + 1),
    }
}

pub fn generate_semantic(semantic: &str, rng: &mut SeededRng, row_idx: usize) -> String {
    match semantic {
        "email" => format!("user_{:04}@example.test", row_idx + 1),
        "phone" => format!("+1{:010}", 1_000_000_000 + row_idx as u64),
        "url" => format!("https://example.test/resource/{:04}", row_idx + 1),
        "country_code" => ISO_COUNTRY_CODES[rng.next_usize(ISO_COUNTRY_CODES.len())].to_owned(),
        "ip_address" | "ipv4" => format!("10.0.{}.{}", (row_idx / 256) % 256, row_idx % 256),
        "currency" => {
            let currencies = ["USD", "EUR", "GBP", "JPY", "CHF"];
            currencies[rng.next_usize(currencies.len())].to_owned()
        }
        _ => format!("synthetic_{semantic}_{:04}", row_idx + 1),
    }
}

pub fn generate_logical(logical: &str, rng: &mut SeededRng, row_idx: usize) -> String {
    match logical {
        "integer" | "bigint" | "smallint" => format!("{}", rng.next_u64() % 10_000),
        "decimal" | "numeric" => {
            let whole = rng.next_u64() % 1000;
            let frac = rng.next_u64() % 100;
            format!("{whole}.{frac:02}")
        }
        "boolean" => if rng.next_bool() { "true" } else { "false" }.to_owned(),
        "date" => {
            let year = 2020 + (row_idx % 5);
            let month = 1 + rng.next_usize(12);
            let day = 1 + rng.next_usize(28);
            format!("{year:04}-{month:02}-{day:02}")
        }
        "timestamp" => {
            let year = 2020 + (row_idx % 5);
            let month = 1 + rng.next_usize(12);
            let day = 1 + rng.next_usize(28);
            let hour = rng.next_usize(24);
            let minute = rng.next_usize(60);
            let second = rng.next_usize(60);
            format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
        }
        "time" => {
            let hour = rng.next_usize(24);
            let minute = rng.next_usize(60);
            let second = rng.next_usize(60);
            format!("{hour:02}:{minute:02}:{second:02}")
        }
        "uuid" => synthetic_uuid(rng, row_idx),
        "json" | "jsonb" => format!("\"{{\"\"idx\"\":{row_idx}}}\""),
        "binary" => format!("0x{:016x}", rng.next_u64()),
        _ => csv_safe_string(rng, row_idx),
    }
}

fn synthetic_uuid(rng: &mut SeededRng, row_idx: usize) -> String {
    let a = rng.next_u64();
    let _ = row_idx;
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (a >> 32) as u32,
        (a >> 16) as u16,
        a as u16 & 0x0FFF,
        0x8000 | (rng.next_u64() as u16 & 0x3FFF),
        rng.next_u64() & 0xFFFF_FFFF_FFFF
    )
}

fn csv_safe_string(rng: &mut SeededRng, row_idx: usize) -> String {
    let prefixes = [
        "sample", "test", "demo", "synth", "mock", "placeholder", "example",
    ];
    let prefix = prefixes[rng.next_usize(prefixes.len())];
    format!("{prefix}_{:04}", row_idx + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use know_now_contract::contract::ContractAttribute;

    fn make_attr(logical: &str) -> ContractAttribute {
        ContractAttribute {
            id: "test".into(),
            name: "test".into(),
            logical_type: Some(logical.into()),
            semantic_type: None,
            sensitivity: None,
            pii: None,
            required: Some(true),
            is_unique: Some(true),
            constraints: vec![],
            description: None,
            attr_type: None,
        }
    }

    #[test]
    fn pk_integer_is_sequential() {
        let mut rng = SeededRng::new(0);
        assert_eq!(generate_pk(&make_attr("integer"), &mut rng, 0), "1");
        assert_eq!(generate_pk(&make_attr("integer"), &mut rng, 9), "10");
    }

    #[test]
    fn pk_uuid_is_valid_format() {
        let mut rng = SeededRng::new(42);
        let uuid = generate_pk(&make_attr("uuid"), &mut rng, 0);
        assert_eq!(uuid.len(), 36);
        assert!(uuid.contains("-4"));
    }

    #[test]
    fn email_semantic() {
        let mut rng = SeededRng::new(0);
        let email = generate_semantic("email", &mut rng, 0);
        assert!(email.contains("@example.test"));
        assert!(email.starts_with("user_"));
    }

    #[test]
    fn phone_semantic() {
        let mut rng = SeededRng::new(0);
        let phone = generate_semantic("phone", &mut rng, 0);
        assert!(phone.starts_with("+1"));
    }

    #[test]
    fn country_code_from_iso_list() {
        let mut rng = SeededRng::new(0);
        for _ in 0..50 {
            let code = generate_semantic("country_code", &mut rng, 0);
            assert!(
                ISO_COUNTRY_CODES.contains(&code.as_str()),
                "unexpected country code: {code}"
            );
        }
    }

    #[test]
    fn boolean_values() {
        let mut rng = SeededRng::new(0);
        let mut saw_true = false;
        let mut saw_false = false;
        for i in 0..100 {
            match generate_logical("boolean", &mut rng, i).as_str() {
                "true" => saw_true = true,
                "false" => saw_false = true,
                other => panic!("unexpected boolean: {other}"),
            }
        }
        assert!(saw_true && saw_false, "should produce both true and false");
    }

    #[test]
    fn decimal_has_two_fraction_digits() {
        let mut rng = SeededRng::new(0);
        let val = generate_logical("decimal", &mut rng, 0);
        let parts: Vec<&str> = val.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 2);
    }

    #[test]
    fn date_format() {
        let mut rng = SeededRng::new(0);
        let date = generate_logical("date", &mut rng, 0);
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn timestamp_format() {
        let mut rng = SeededRng::new(0);
        let ts = generate_logical("timestamp", &mut rng, 0);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }

    #[test]
    fn string_is_csv_safe() {
        let mut rng = SeededRng::new(0);
        for i in 0..50 {
            let val = generate_logical("string", &mut rng, i);
            assert!(!val.contains(','), "CSV-unsafe comma in: {val}");
            assert!(!val.contains('\n'), "CSV-unsafe newline in: {val}");
        }
    }
}
