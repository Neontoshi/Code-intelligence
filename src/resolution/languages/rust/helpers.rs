use super::RustResolver;

impl RustResolver {
    pub(super) fn strip_generic_args(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        let chars: Vec<char> = name.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == ':' && chars[i + 1] == '<' {
                i += 2;
                let mut depth = 1usize;
                while i < chars.len() && depth > 0 {
                    match chars[i] {
                        '<' => depth += 1,
                        '>' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                continue;
            }

            result.push(chars[i]);
            i += 1;
        }

        result
    }

    pub(super) fn is_primitive_type(name: &str) -> bool {
        matches!(
            name,
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "f32"
                | "f64"
                | "bool"
                | "char"
                | "str"
        )
    }

    pub(super) fn looks_like_external_qualified_call(parts: &[String]) -> bool {
        if parts.is_empty() {
            return false;
        }

        let first = parts[0].as_str();

        if Self::is_primitive_type(first) {
            return true;
        }

        if matches!(
            first,
            "std"
                | "core"
                | "alloc"
                | "tokio"
                | "serde"
                | "axum"
                | "redis"
                | "dotenvy"
                | "solana_sdk"
                | "solana_keccak_hasher"
                | "utoipa_swagger_ui"
                | "axum_middleware"
                | "tracing_opentelemetry"
                | "opentelemetry"
                | "sentry"
                | "prometheus"
                | "base64"
                | "urlencoding"
        ) {
            return true;
        }

        if parts.len() >= 2 {
            let method = parts.last().map(|s| s.as_str()).unwrap_or("");
            if first
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && matches!(
                    method,
                    "new"
                        | "default"
                        | "from"
                        | "from_str"
                        | "from_static"
                        | "from_secret"
                        | "from_le_bytes"
                        | "new_from_array"
                        | "find_program_address"
                        | "new_with_bytes"
                        | "new_unsigned"
                        | "new_with_blockhash"
                        | "new_with_timeout_and_commitment"
                        | "confirmed"
                        | "disable"
                        | "builder"
                        | "build"
                        | "layer"
                        | "encode"
                        | "try_from_default_env"
                        | "set_compute_unit_limit"
                        | "new_v4"
                        | "default_registry"
                )
            {
                return true;
            }
        }

        false
    }

    pub(super) fn std_member_is_external(member: &str) -> bool {
        matches!(
            member,
            "iter"
                | "filter"
                | "map"
                | "collect"
                | "len"
                | "is_empty"
                | "contains"
                | "is_ok"
                | "to_vec"
                | "ok_or_else"
                | "is_some"
                | "edge_count"
        )
    }
}
