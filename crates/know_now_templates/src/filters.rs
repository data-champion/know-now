use minijinja::Environment;

pub fn register_builtin_filters(env: &mut Environment<'_>) {
    env.add_filter("snake_case", filter_snake_case);
    env.add_filter("kebab_case", filter_kebab_case);
    env.add_filter("pascal_case", filter_pascal_case);
    env.add_filter("upper_case", filter_upper_case);
    env.add_filter("lower_case", filter_lower_case);
    env.add_filter("markdown_escape", filter_markdown_escape);
    env.add_filter("html_escape", filter_html_escape);
}

fn filter_snake_case(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 4);
    for (i, ch) in value.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result.replace(['-', ' '], "_")
}

fn filter_kebab_case(value: &str) -> String {
    filter_snake_case(value).replace('_', "-")
}

fn filter_pascal_case(value: &str) -> String {
    value
        .split(['_', '-', ' '])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |c| {
                c.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect()
}

fn filter_upper_case(value: &str) -> String {
    value.to_uppercase()
}

fn filter_lower_case(value: &str) -> String {
    value.to_lowercase()
}

fn filter_markdown_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('|', "\\|")
}

fn filter_html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_conversion() {
        assert_eq!(filter_snake_case("CustomerOrder"), "customer_order");
        assert_eq!(filter_snake_case("my-name"), "my_name");
        assert_eq!(filter_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn kebab_case_conversion() {
        assert_eq!(filter_kebab_case("CustomerOrder"), "customer-order");
        assert_eq!(filter_kebab_case("already_snake"), "already-snake");
    }

    #[test]
    fn pascal_case_conversion() {
        assert_eq!(filter_pascal_case("customer_order"), "CustomerOrder");
        assert_eq!(filter_pascal_case("my-name"), "MyName");
    }

    #[test]
    fn upper_and_lower_case() {
        assert_eq!(filter_upper_case("hello"), "HELLO");
        assert_eq!(filter_lower_case("HELLO"), "hello");
    }

    #[test]
    fn markdown_escape_special_chars() {
        assert_eq!(filter_markdown_escape("*bold*"), "\\*bold\\*");
        assert_eq!(filter_markdown_escape("`code`"), "\\`code\\`");
    }

    #[test]
    fn html_escape_special_chars() {
        assert_eq!(filter_html_escape("<div>"), "&lt;div&gt;");
        assert_eq!(filter_html_escape("a&b"), "a&amp;b");
    }
}
