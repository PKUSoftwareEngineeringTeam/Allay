## Layouts

All your Allay templates should be placed in the `templates` directory. You should at least have these default templates:

- `index.html`: The main index page template.
- `404.html`: The 404 error page template.
- `page.html`: The default template for regular pages.

When the url field of the markdown [front matter](../custom-contents/front-matter.md) is not specified, Allay will use the `page.html` template to render the page.

### Custom Template Layouts

However, not all pages should use the same layout. You can specify a different layout template for each page by using the `template` field in the front matter. For example:

`content/about/index.md`:

```md
---
title: About Me
template: about.html
---

Hello, this is the about page.
```

`templates/about.html`:

```html
<body>
    <h1>About Me</h1>
    <p>{: .content :}</p>
</body>
```

This way, you can create custom layouts for different types of pages in your blog. Just make sure that the specified template file exists in the `templates` directory.
