//! # Allay Markdown Parser
//! A lib for markdown parsing and html code generation

use std::{collections::HashMap, sync::LazyLock};

use pulldown_cmark::{Parser, html};
use regex::Regex;

/// The type of a template variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateVariable {
    String(String), // should support string concat
    Integer(i64),   // should support integer operations (+, -, *, /, %)
    Boolean(bool),  // should support boolean operations (&, |, !)
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
/// The regex pattern for commands
pub static COMMAND_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{-(\w+)-}").unwrap());
/// The regex pattern for rendering an expression
pub static RENDER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{:(\w+):}").unwrap());

/// Magic commands for template
/// 
/// These are keywords for template engine to process the template
/// Which means that user cannot use these keywords as variable names
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateCommand {
    /// Create a variable in current scope
    ///
    /// # Example:
    /// ```html
    /// {- set $var .value -}
    /// {: $var :} <!-- use the variable -->
    /// ```
    SET,

    /// Return a variable from current scope
    /// Can usually be omitted, as variables are automatically resolved
    /// 
    /// # Example:
    /// ```html
    /// {: get $var :}
    /// {: $var :} <!-- equivalent to above -->
    /// ```
    GET,

    /// For loop, to iterate a list
    /// Throw error if the variable is not a list or the variable does not exist
    ///
    /// Examples:
    /// ```html
    /// <ul>
    ///     {- for $player: .players -} <!-- create a variable $player for each item in .players -->
    ///     <li>
    ///     {: $player.name :}
    ///     </li>
    ///     {- end -}
    /// </ul>
    /// ```
    /// ```html
    /// <ul>
    ///     {- for $player, $index: .players -} <!-- support index, start from 0 -->
    ///     <li>
    ///     {: $index + 1 :}: {: $player.name :}
    ///     </li>
    ///     {- end -}
    /// </ul>
    /// ```
    FOR,

    /// With block, to enter a child scope
    /// If the scope does not exist, it will be skipped
    ///
    /// # Example:
    /// ```html
    /// {- with .author -}
    /// <p>{: .name :}</p>
    /// {- end -}
    /// ```
    WITH,

    /// If block, to conditionally render content
    /// If the condition is not null and true, the content will be rendered
    /// Otherwise, it will be skipped
    ///
    /// # Example:
    /// ```html
    /// {- if .is_admin -}
    /// <p>Admin</p>
    /// {- end -}
    /// ```
    IF,

    /// Else block, to provide an alternative content for if block
    ///
    /// # Example: 
    /// ```html
    /// {- if .is_admin -} 
    /// <p>Admin</p>
    /// {- else -}
    /// <p>User</p>
    /// {- end -}
    /// ```
    ELSE,

    /// End block, to end a command block
    END,

    /// Return the parameter passed to the scope, start from 0
    /// If the parameter does not exist, skip it
    /// 
    /// # Example:
    /// ```html
    /// <div class="say">{: param 0 :}</div>
    /// ```
    PARAM,

    /// Include other template files
    /// The first parameter is the file path
    /// The second parameter is the scope to pass to the included template
    /// If the scope is not provided, use the current scope
    /// Other parameters can be passed to the included template and accessed by PARAM
    /// 
    /// # Example:
    /// ```html
    /// {- include "header.html" -} <!-- use current scope -->
    /// {- include "article/post.html" .post "My Post" -} <!-- pass .post as scope, "My Post" as param 0 -->
    /// ```
    INCLUDE
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

/// The root scope for template
pub const GLOABL_SCOPE: TemplateScope = TemplateScope {};

/// The metadata for markdown, usually in YAML/TOML format
/// It should totally configurable by user
/// This content WILL BE RENDERED as HTML element if needed
///
/// # Example
///
/// Here is an example of YAML front matter:
/// ```md
/// ---
/// title: "Hello, world!"
/// date: 2024-01-01
/// description: "This is a test."
/// tags:  // a list
///   - test
///   - markdown
/// reference:  // a map
///   - url: "https://example.com"
///   - name: "Example Site"  
/// ---
/// The markdown content **starts** here...
/// ```
/// The TOML front matter is similar, but uses `+++` as delimiters
///
/// NOTE: Should be implemented as a template scope
/// In rendering, the template will be in markdown file scopes:
/// `<head> <title>{: .title :}</title></head>`
#[derive(Debug, Clone)]
pub struct MarkdownMeta {}

/// The shortcode pattern for commands
/// TODO: make it support parameters
pub static SHORTCODE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{<(\w+)>}").unwrap());

/// The inner content of a shortcode
pub const INNER: &str = "inner";
pub const CLOSURE_END: char = '/';

/// The shortcode for markdown
///
/// The pattern is provided in template
/// Use {: .0 :} for first parameter, {: .1 :} for second parameter, and so on.
///
/// # Examples
///
/// 1. Shortcode: note (self-closing)
///    - template: `<img src="note.png" alt="note" />`
///    - markdown: "{< note >}"
///    - rendered: `<img src="note.png" alt="note" />`
/// 2. Shortcode: closure (with inner content)
///    - template: `<div class="closure">{: .inner :}</div>`
///    - markdown: {< closure >} text {< /closure >}"
///    - rendered: `<div class="closure">text</div>`
/// 3. Shortcode: say (with parameter)
///    - template: `<div class="say">{- param 0 -}</div>`
///    - markdown: "{< say "hello" >}" or "{< say hello >}"
///    - rendered: `<div class="say">hello</div>`
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Shortcode {
    name: String,
    template: String,
}

impl Shortcode {
    /// Create a new shortcode
    pub fn new(name: String, template: String) -> Self {
        Self { name, template }
    }
}

/// The main struct for the Allay Markdown Parser
#[derive(Debug, Clone, Default)]
pub struct AllayMdParser {}

impl AllayMdParser {
    /// Parse the markdown string and return the HTML string
    pub fn parse(self, md: &str) -> String {
        let parser = Parser::new(md);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}
