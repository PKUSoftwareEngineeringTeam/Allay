## Shortcode

Shortcodes are a powerful feature in Allay that allows you to embed dynamic content within your markdown files. They are similar to macros or functions in programming languages, enabling you to reuse code snippets and pass parameters to customize their behavior.

### Definition

You should define all your shortcodes in the `shortcodes` directory. Each shortcode will be bound to the HTML/Markdown file with the same name. You can then use these shortcodes in your markdown files by using the following syntax:

```
{< shortcode_name params... >}
```

### Examples

#### Self-Closing Shortcodes

`shortcodes/note.md`:

```md
![Note](https://img.shields.io/badge/Note-Important-brightgreen)
```

In your markdown:

```md
Here is a note badge: {< note />}
```

#### Block Shortcodes

`shortcodes/closure.html`:

```html
<div class="closure">{: .inner :}</div>
```

In your markdown:

```md
{< closure >}
This is some important content.
{</ closure >}
```

The inner content will be placed where `{: .inner :}` is specified in the template.

#### Shortcodes with Parameters

`shortcodes/say.html`:

```html
<div class="say">{- param.0 -}</div>
```

In your markdown:

```md
{< say "Hello, World!" >}
```

### Recursive Shortcodes Template

`shortcodes/warning.html`:

```html
<div class="warning">
    {- include "warning-badge" -}
    <div class="content">{: .inner :}</div>
</div>
```

`shortcodes/warning-badge.md`:

```md
![Warning](https://img.shields.io/badge/Warning-Important-red)
```

In your markdown:

```md
{< warning >}
This is a warning message.
{</ warning >}
```

> Shortcode is actually a syntactic sugar for including templates. `{< note "Hello!" />}` is equivalent to `{- include "note" this "Hello" -}`. But note that they are not in the same folder.
