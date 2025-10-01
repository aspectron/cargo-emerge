use std::collections::HashMap;

/// Template processor for resolving $VARIABLE format variables
pub struct Tpl {
    variables: HashMap<String, String>,
}

impl Tpl {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Register a variable with its value
    pub fn register<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.variables.insert(key.into(), value.into());
    }

    /// Parse a string and resolve all $VARIABLE references
    pub fn parse(&self, input: &str) -> String {
        let mut result = input.to_string();

        for (key, value) in &self.variables {
            let pattern = format!("${}", key);
            result = result.replace(&pattern, value);
        }

        result
    }

    /// Parse a Vec of strings
    pub fn parse_vec(&self, input: &[String]) -> Vec<String> {
        input.iter().map(|s| self.parse(s)).collect()
    }
}

impl Default for Tpl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_parsing() {
        let mut tpl = Tpl::new();
        tpl.register("VERSION", "1.0.0");
        tpl.register("PLATFORM", "macos");

        let result = tpl.parse("app-$VERSION-$PLATFORM.dmg");
        assert_eq!(result, "app-1.0.0-macos.dmg");
    }

    #[test]
    fn test_multiple_occurrences() {
        let mut tpl = Tpl::new();
        tpl.register("NAME", "test");

        let result = tpl.parse("$NAME-$NAME");
        assert_eq!(result, "test-test");
    }
}
