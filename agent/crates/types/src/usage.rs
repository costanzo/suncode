use serde::{Deserialize, Serialize};

/// Token usage. The optional breakdown fields are set only when a provider
/// reports them and are omitted from JSON when absent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_miss_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
}

impl Usage {
    /// Usage with only the input, output, and total counts.
    pub fn totals(input_tokens: u64, output_tokens: u64, total_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens,
            ..Self::default()
        }
    }

    /// The same counts without the optional breakdown.
    pub fn without_breakdown(&self) -> Self {
        Self::totals(self.input_tokens, self.output_tokens, self.total_tokens)
    }

    /// Accumulates the input, output, and total counts.
    pub fn add(&mut self, other: &Usage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn omits_absent_breakdown_fields() {
        assert_eq!(
            serde_json::to_value(Usage::totals(8, 1, 9)).unwrap(),
            json!({"input_tokens": 8, "output_tokens": 1, "total_tokens": 9})
        );
        let usage = Usage {
            cache_read_tokens: Some(6),
            ..Usage::totals(8, 1, 9)
        };
        let value = serde_json::to_value(&usage).unwrap();
        assert_eq!(value["cache_read_tokens"], 6);
        assert!(value.get("reasoning_tokens").is_none());
        assert_eq!(serde_json::from_value::<Usage>(value).unwrap(), usage);
    }
}
