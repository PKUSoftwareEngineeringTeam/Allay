## File Structure

Your blog directory should look like this:

```
.
├── allay.toml      # The configuration file of your blog, usually comes from the theme you use
├── content         # Your markdown files go here
├── public          # Generated static files will be placed here
├── shortcodes      # Custom shortcodes can be defined here
├── static          # Static files which can be directly accessed
└── themes          # Themes directory
```

- `static/`: This directory contains static files such as images and PDF files. These files will be copied directly to the `public/` directory during the build process and can be accessed directly via URLs.
- `content/`: This directory contains your markdown files. Each markdown file represents a page on your blog. See [Create Pages](../custom-contents/create-pages.md) for more details on how to create and organize your markdown files.
- `public/`: This directory is where the generated static files will be placed after building your blog. You can deploy the contents of this directory to your web server.
- `shortcodes/`: This directory is for custom shortcodes that you can define to use in your markdown files. See [Shortcodes](../extra-functions/shortcode.md) for more details on how to create and use shortcodes.
- `themes/`: This directory contains themes for your blog. You can create subdirectories for each theme, and each theme can have its own templates and static files. See [Themes](../themes/index.md) for more details on how to custom themes.
- `allay.toml`: This is the main configuration file for your blog. You can set various parameters such as the site title, base URL, and theme in this file. See [Configuration](../configuration/index.md) for more details.
