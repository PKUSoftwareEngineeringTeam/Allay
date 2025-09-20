## Front Matter

Markdown supports front-matter metadata at the top of the file. The front-matter should be enclosed within `---` for
YAML or `+++` for TOML. Both are supported.

Here is an example of a markdown file front-matter:

```md
---
title: "My First Page"
date: 2024-01-01
description: "This is a description of my first page."
tags: ["introduction", "welcome"]
---
```

or

```md
+++
title = "My First Page"
date = 2024-01-01
description = "This is a description of my first page."
tags = ["introduction", "welcome"]
+++
```

### Default Metadata Fields

These are the default metadata fields supported by Allay, all of which are optional:

| Field      | Type   | Description                                                                       |
|------------|--------|-----------------------------------------------------------------------------------|
| `head`     | String | The title of this **web page** but not the article.                               |
| `template` | String | Specifies the [layout template](../template/layouts.md) to use.                   |
| `date`     | Date   | The publication date of the page. Usually like "2024-01-01"                       |
| `url`      | String | Custom URL for the page. If not specified, it will be derived from the file path. |

Note that fields like `title`, `description` and `tags` should actually be used in your theme templates but not Allay
itself.
