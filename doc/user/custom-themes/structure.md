## File Structure

Your theme should be organized in a specific directory structure to ensure that the Allay Engine can properly locate and apply your custom styles. Below is the required structure for your theme files:

```
my_custom_theme/
├── assets/   # Directory for images, fonts, css files, javascript files, etc.
├── i18n/   # Directory for localization files
├── pages/   # Directory for the pages
|      ├ index.html # The main page template
|      ├ 404.html   # The 404 error page template
|      ├ page.html  # The template for regular pages
|      └ ...   # Other custom pages
├── templates/   # Directory for HTML templates
|      ├ shortcodes  # Subdirectory for shortcodes
|      └ ...   # Other template files
├── allay.toml   # Configuration file for the theme (Optional)
└── theme.toml   # Metadata for the theme
```

The `allay.toml` file is optional, but recommended if you want to provide default configuration parameters for users of your theme.

The `theme.toml` file is essential as it contains metadata about your theme and let it be recognized by the Allay Engine. Here is an example of what the `theme.toml` file might look like:

```toml
# theme.toml template for an Allay theme

name = "Axolotl"
license = "GPL-3.0-only"
licenselink = "https://www.gnu.org/licenses/gpl-3.0-standalone.html"
description = "Official theme of Allay"
homepage = "https://github.com/PKUSoftwareEngineeringTeam/Axolotl"
demosite = "https://github.com/PKUSoftwareEngineeringTeam/Axolotl-Wiki"

tags = ["clean", "light", "personal"]
min_version = "0.1.0"

[author]
name = "LeoDreamer"
homepage = "https://leodreamer2004.github.io"
```
