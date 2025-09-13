## Shortcode

Shortcodes are a powerful feature in Allay that allows you to embed dynamic content within your markdown files. They are similar to macros or functions in programming languages, enabling you to reuse code snippets and pass parameters to customize their behavior.

### Usage

You should define all your shortcodes in the `templates/shortcodes` directory. Each shortcode should be a separate HTML file. You can then use these shortcodes in your markdown files by using the following syntax:

```
{< shortcode_name params... />}
```

- For example, let's define a `note` shortcode to display a note icon:

    `templates/shortcodes/note.html`:

    ```html
    <img src="note.png" alt="note" />
    ```

    You can then use this shortcode in your markdown files like this:

    ```md
    Here is a note: {< note />}
    ```

- You may need a block shortcode that wraps around some inner content. For example, a `closure` shortcode to create a styled div:

    `templates/shortcodes/closure.html`:

    ```html
    <div class="closure">{: .inner :}</div>
    ```

    You can use this shortcode with inner content like this:

    ```md
    {< closure >}
    This is some important content.
    {</ closure >}
    ```

    The inner content will be placed where `{: .inner :}` is specified in the template.

- Parameters are also supported. For example, a `say` shortcode that takes a parameter and displays it:

    `templates/shortcodes/say.html`:

    ```html
    <div class="say">{- param 0 -}</div>
    ```

    You can use this shortcode with a parameter like this:

    ```md
    {< say "Hello, World!" >}
    ```

    or more easily without quotes if the parameter is a single word:

    ```md
    {< say Hello >}
    ```
