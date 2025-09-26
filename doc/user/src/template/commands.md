## Commands

For code in Allay templates, we use `{- -}` to denote a command block, and `{: :}` to denote an expression block. All expression blocks will be evaluated and replaced by their result.

Allay templates support several commands to control the flow of the template rendering process.

### `set`

`set` directive is used to create a variable in the current scope.

```html
{- set $var = .something -}
{: $var :} <!-- use the variable -->
```

The `set` directive assigns the value of `something` field of current scope variable, i.e. `this`, to the variable `$var`, which can then be used later in the template.

### `get`

`get` directive is used to return a variable from the current scope. However, it can usually be omitted, as variables are automatically resolved.

```html
{: get $var :}
{: $var :} <!-- equivalent to above -->
```

### `for`

`for` directive is used to iterate over a list. It throws an error if the variable is not a list or does not exist.

```html
<ul>
    {- for $player: .players -}
    <!-- create a variable $player for each item in .players -->
    <li>
    {: $player.name :}
    </li>
    {- end -}
</ul>
```

You can also access the index of the current item by providing a second variable:

```html
<ul>
    {- for $player, $index: .players -}
    <!-- support index, start from 0 -->
    <li>
    {: $index + 1 :}: {: $player.name :}
    </li>
    {- end -}
</ul>
```

### `with`

`with` directive is used to enter a child [scope](./scope.md) by the object. If the object does not exist, it will be skipped.

```html
{- with .author -}
<p>{: .name :}</p>
{- end -}
```

### `if` and `else`

`if` directive is used to conditionally render content. If the condition is not null and true, the content will be rendered; otherwise, it will be skipped. You can also use `else` to provide alternative content.

```html
{- if .is_admin -}
<p>Admin</p>
{- else -}
<p>User</p>
{- end -}
```

### `end`

`end` directive is used to end a command block, such as `for`, `with`, or `if`.

### `param`

`param` directive is used to return the parameter passed to the scope, starting from 0. If the parameter does not exist, it will be skipped.

```html
<div class="say">{: param.0 :}</div>
```

### `include`

`include` directive is used to include other template files.

- The first parameter is the file path
- The second parameter is the scope to pass to the included template (if not provided, the current scope is used)
- Other parameters can be passed to the included template and accessed by `param`.

```html
{- include "header.html" -}
<!-- use current scope -->
{- include "article/post.html" .post "My Post" -}
<!-- pass .post as scope, "My Post" as param 0 -->
```
