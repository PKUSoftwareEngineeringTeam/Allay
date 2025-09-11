## Scope

Each html file in the `templates` directory has its own variable scope. Use `{: . :}` to access the current scope.

### Child Scope

For simplicity, Allay provide `with` directive to create a child scope. You can use `with` to switch the current scope to a child object.

Take an example. If the current variable scope is like:

```toml
title = "Hello, world!"
[author]
name = "John Doe"
age = 30
```

Then the following two snippets are equivalent:

```html
<p>{: .author.name :}</p>
```

```html
{- with .author -}
<p>{: .name :}</p>
{- end -}
```

### Global Scope

Allay will parse your configuration file, i.e. `allay.toml`'s `Param`, and pass its content to the global scope. You can access any configuration variable in your templates by the `CONFIG` field of `GLOBAL` object:

`allay.toml`:

```toml
[Param]
footer = "Axolotl Theme, 2025"
```

`footer.html`:

```html
{: GLOBAL.CONFIG.footer :}
```

### Markdown Page Scope

For every markdown pages, we will parse its front-matter and pass it to the page scope.

In the [template](./layouts.md) of regular pages like `page.html`, you can directly access front-matter params by the current PAGE object `.`, and get the content of the markdown file by the special variable `.content`.

Here is an example of a basic usage:

`content/some-path/index.md`:

```md
---
title: Test
date: 2024-01-01
description: This is a test page.
tags: ["test", "example"]
---
Hello, this is a test page.
```

`templates/page.html`:

```html
<h1>{: .title :}</h1>
<p>{: .date :}</p>
<p>{: .description :}</p>
<ul>
    {- for tag in .tags -}
    <li>{: tag :}</li>
    {- end -}
</ul>
<div>
    {: .content :}
</div>
```

For other templates like `index.html`, you can access all pages by the `PAGES` field of `GLOBAL` object. `PAGES` is an array of all markdown pages, each of which has the same scope as described above.

Here is an example of listing all pages in `index.html`:

```html
<ul>
    {- for $page: GLOBAL.PAGES -}
        {- with $page -}
        <li>
            <a href="{: .url :}">{: .title :}</a>
            <p>{: .description :}</p>
        </li>
        {- end -}
    {- end -}
</ul>
```
