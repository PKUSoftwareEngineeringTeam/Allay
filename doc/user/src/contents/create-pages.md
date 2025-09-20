## Create Pages

You can create custom pages in Allay by adding markdown files to the `content` directory. Each markdown file will be converted into a static HTML page when you run the `allay build` or `allay server` command.

All markdown files should include front-matter metadata at the top of the file. See [Front-matter](./front-matter.md) for more details on the available metadata fields.

## File Organization

Unless the "url" field is specified in the front-matter, the url of the page will be naturally derived from the file path. Note that the file name `index.md` is special, as it will be treated as the root of the directory.

For example, given the following file structure:

```
content
├── about.md
├── blog
│   ├── index.md
│   └── first-post.md
└── projects
    └── project1.md
```

The generated HTML in `public` will be:

```
public
├── about.html          # from about.md
├── blog
│   ├── index.html      # from blog/index.md
│   └── first-post.html # from blog/first-post.md
└── projects
    └── project1.html   # from projects/project1.md
```
