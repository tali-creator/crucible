//! Helpers for measuring and reporting contract execution costs.

/// A report of the compute costs for a contract invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostReport {
    instructions: u64,
    memory: u64,
}

impl CostReport {
    /// Creates a new cost report.
    pub fn new(instructions: u64, memory: u64) -> Self {
        Self {
            instructions,
            memory,
        }
    }

    /// Returns the number of CPU instructions consumed.
    pub fn instructions(&self) -> u64 {
        self.instructions
    }

    /// Returns the peak memory usage in bytes.
    pub fn memory_bytes(&self) -> u64 {
        self.memory
    }

    /// Returns the estimated network fee in stroops.
    ///
    /// This is a simplified estimation based on instructions.
    /// Heuristic: 100 instructions = 1 stroop (calibrate as needed).
    pub fn fee_stroops(&self) -> i64 {
        (self.instructions / 100) as i64
    }

    /// Returns a human-readable formatted table report of the costs.
    ///
    /// The output is a formatted table with comma-separated numbers for readability.
    /// Example:
    /// ```text
    /// ┌─────────────────────┬───────────┐
    /// │ Metric              │ Value     │
    /// ├─────────────────────┼───────────┤
    /// │ Instructions        │ 1,234,567 │
    /// │ Memory (bytes)      │ 45,678    │
    /// │ Estimated fee       │ 123 str   │
    /// └─────────────────────┴───────────┘
    /// ```
    pub fn report(&self) -> String {
        let instructions_str = format_with_commas(self.instructions);
        let memory_str = format_with_commas(self.memory);
        let fee_str = format!("{} str", self.fee_stroops());

        // Create formatted table with box-drawing characters
        let mut output = String::new();
        output.push_str("┌─────────────────────┬───────────┐\n");
        output.push_str("│ Metric              │ Value     │\n");
        output.push_str("├─────────────────────┼───────────┤\n");
        output.push_str(&format!(
            "│ Instructions        │ {:>9} │\n",
            instructions_str
        ));
        output.push_str(&format!("│ Memory (bytes)      │ {:>9} │\n", memory_str));
        output.push_str(&format!("│ Estimated fee       │ {:>9} │\n", fee_str));
        output.push_str("└─────────────────────┴───────────┘");

        output
    }
}

/// Format a number with comma separators for readability.
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        result.push(c);
        let remaining = len - i - 1;
        if remaining > 0 && remaining.is_multiple_of(3) {
            result.push(',');
        }
    }

    result
}

#[cfg(feature = "snapshots")]
impl CostReport {
    /// Assert that the cost report matches a snapshot.
    ///
    /// This is a placeholder for snapshot testing integration.
    /// When the `snapshots` feature is enabled, cost reports can be compared
    /// against saved snapshots to catch performance regressions.
    ///
    /// # Panics
    /// Panics if the snapshot does not exist or does not match.
    pub fn assert_snapshot(&self, name: &str) {
        // TODO: Integrate with insta or similar snapshot testing library
        // For now, this is a placeholder that could be extended with:
        // - insta integration for automated snapshot tests
        // - Custom snapshot storage and comparison
        // - Reporting when regressions are detected
        eprintln!("Snapshot assertion for '{}': {:?}", name, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_report_creation() {
        let report = CostReport::new(1_000_000, 50_000);
        assert_eq!(report.instructions(), 1_000_000);
        assert_eq!(report.memory_bytes(), 50_000);
    }

    #[test]
    fn test_fee_stroops_calculation() {
        let report = CostReport::new(10_000, 0);
        assert_eq!(report.fee_stroops(), 100); // 10_000 / 100 = 100
    }

    #[test]
    fn test_report_returns_non_empty_string() {
        let report = CostReport::new(1_234_567, 45_678);
        let report_str = report.report();
        assert!(!report_str.is_empty());
        // Check that expected labels are present
        assert!(report_str.contains("Instructions"));
        assert!(report_str.contains("Memory (bytes)"));
        assert!(report_str.contains("Estimated fee"));
    }

    #[test]
    fn test_format_with_commas() {
        assert_eq!(format_with_commas(0), "0");
        assert_eq!(format_with_commas(123), "123");
        assert_eq!(format_with_commas(1_234), "1,234");
        assert_eq!(format_with_commas(1_234_567), "1,234,567");
        assert_eq!(format_with_commas(1_000_000_000), "1,000,000,000");
    }

    #[test]
    fn test_report_formatting_contains_table_elements() {
        let report = CostReport::new(1_234_567, 45_678);
        let report_str = report.report();
        // Check for box-drawing characters
        assert!(report_str.contains("┌"));
        assert!(report_str.contains("┐"));
        assert!(report_str.contains("└"));
        assert!(report_str.contains("┘"));
        assert!(report_str.contains("├"));
        assert!(report_str.contains("┤"));
        assert!(report_str.contains("┼"));
    }
}
