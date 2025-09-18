# Allay Compiler

## Compiler Overview

The Allay compiler translates the Allay template language into HTML. It has two main components:

1. **Parser**: Converts the template source code into an Abstract Syntax Tree (AST). The EBNF grammar
   for the Allay template language is defined in [ast.md](ast.md) and the implementation of the parser
   in [pest](https://pest.rs/) DSL can be found in [allay.pest](../../crates/allay-compiler/src/allay.pest).
2. **Interpreter**: Traverses the AST and executes the template logic to produce the final HTML output.

## Compilation Pipeline

Since we allow both markdown and html in our templates, the compilation pipeline is as follows:

1. Convert all markdown to HTML using [pulldown-cmark](https://crates.io/crates/pulldown-cmark).
2. Parse the HTML into an AST using the Allay parser.
3. Interpret the AST to generate the final HTML output. Note that the shortcode and include commands
   will bring new templates into the current template, we should do step 2 and 3 recursively until all templates
   are fully interpreted.

## Incremental Compilation

Incremental compilation is important for performance of hot-reload, as we want to avoid recompiling
the entire site on every change. To implement incremental compilation, we need to build a dependency
tree of all templates and save all intermediate files during compilation. When a file changes, we can
traverse the dependency tree to find all affected templates and only recompile those.
