/// A fully-qualified Daikon variable name, usually built up by recursive descent
/// through a compound type.
///
/// This type is intentionally simple and public so it can also be used
/// from outside this crate when constructing variable names that must
/// match the ones produced by the decls generator
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarName(String);

impl VarName {
    /// Wrap an existing string as the root of a variable name
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Append a struct/tuple field access: `{self}.{field}`.
    pub fn project_field(&self, field: &str) -> Self {
        Self(format!("{}.{}", self.0, field))
    }

    /// Append an enum variant resolution: `{self}::{variant}`.
    pub fn project_variant(&self, variant: &str) -> Self {
        Self(format!("{}::{}", self.0, variant))
    }

    /// Append a concrete index into a sequence: `{self}[{i}]`.
    pub fn project_index(&self, i: usize) -> Self {
        Self(format!("{}[{}]", self.0, i))
    }

    /// Append the collapsed-sequence marker: `{self}[..]`.
    pub fn project_slice(&self) -> Self {
        Self(format!("{}[..]", self.0))
    }

    /// Prefix with a pointer dereference: `*{self}`
    pub fn project_deref(&self) -> Self {
        Self(format!("*{}", self.0))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for VarName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for VarName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for VarName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<VarName> for String {
    fn from(v: VarName) -> String {
        v.0
    }
}

/// Replaces necessary chars in string with corresponding escape
/// sequences.
/// "In the declaration file, blanks must be replaced by \_, 
/// and backslashes must be escaped as \\."
pub fn escape_str(s: String) -> String {
    s.replace("\\", "\\\\").replace(" ", "\\_")
}