## File Structure

Your theme should be organized in a specific directory structure to ensure that the Allay Engine can properly locate and apply your custom styles. Below is the required structure for your theme files:

```
.
├── allay.toml   # Optional configuration file for the theme
├── assets       # Directory for images, fonts, CSS files, JavaScript files, etc.
├── i18n         # Directory for localization files
├── pages        # Directory for the pages
│   ├── 404.html        # The 404 error page template
│   ├── index.html      # The main page template
│   └── page.html       # The template for regular pages
├── templates   # Directory for HTML templates
└── theme-meta.toml  # Metadata for the theme
```

The `allay.toml` file is optional, but recommended if you want to provide default configuration parameters for users of your theme.

The `theme-meta.toml` file is essential as it contains metadata about your theme and let it be recognized by the Allay Engine. Here is an example of what the `theme-meta.toml` file might look like:

```toml
# theme-meta template for an Allay theme

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
