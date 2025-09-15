use std::collections::HashMap;

/// The type of template variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateVariable {
    String(String), // should support string concat
    Integer(i64),   // should support integer operations (+, -, *, /, %)
    Boolean(bool),  // should support boolean operations (&&, ||, !)
    List(Vec<TemplateVariable>),
    Map(HashMap<String, TemplateVariable>),
    Null,
}

impl TemplateVariable {
    /// Render the variable as a string in HTML
    pub fn render(&self) -> String {
        todo!()
    }
}

/// The variable scope for template, organized as a tree like json object
///
/// # Example
/// Current scope:
/// ```json
/// {
///  "title": "Hello, world!",
///  "author": {
///    "name": "John Doe",
///    "age": 30
///  },
///  "tags": ["test", "markdown"]
/// }
/// ```
///
/// Then the template can be like:
/// ```html
/// <!-- visit variables by dot notation -->
/// <h1>{: .title :}</h1>
///
/// <!-- use "for" to iterate a list -->
/// {- for $tag: .tags -}
/// <span>{: $tag :}</span>
/// {- end -}
///
/// <!-- use "with" to visit a child scope -->
/// {- with .author -}
/// <p>Author: {: .name :}, Age: {: .age :}</p>
/// {- end -}
/// ```
#[derive(Debug, Clone)]
pub struct TemplateScope {}
