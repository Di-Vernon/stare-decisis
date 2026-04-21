//! 출력 포맷 helper — `--format text|json|yaml`.

use anyhow::Result;
use serde::Serialize;

use crate::args::OutputFormat;

pub fn format_output<T: Serialize + std::fmt::Display>(
    data: &T,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Text => Ok(format!("{data}")),
        OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::fmt;

    #[derive(Serialize)]
    struct Example {
        key: String,
    }

    impl fmt::Display for Example {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "key={}", self.key)
        }
    }

    #[test]
    fn text_uses_display() {
        let e = Example { key: "value".into() };
        assert_eq!(format_output(&e, OutputFormat::Text).unwrap(), "key=value");
    }

    #[test]
    fn json_pretty() {
        let e = Example { key: "value".into() };
        let out = format_output(&e, OutputFormat::Json).unwrap();
        assert!(out.contains("\"key\": \"value\""));
    }

    #[test]
    fn yaml_basic() {
        let e = Example { key: "value".into() };
        let out = format_output(&e, OutputFormat::Yaml).unwrap();
        assert!(out.contains("key: value"));
    }
}
