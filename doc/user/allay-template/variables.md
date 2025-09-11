## Variables

All variables in allay template should be started with `$`. To define a variable, use the following syntax:

```html
{- set $my_variable "Hello, World" -}
<p>{: $my_variable :}</p>
```

The `set` directive assigns a value to a variable, and the `:` syntax is used to output the value of the variable. In this example, the output will be:

```html
<p>Hello, World</p>
```

All variable types in Allay templates include:

| Type    | Description                                      | Example                     |
|---------|--------------------------------------------------|-----------------------------|
| String  | A sequence of characters.                        | `"Hello, World!"`           |
| Integer | A whole number.                                  | `42`                        |
| Boolean | A true or false value.                           | `true` or `false`           |
| Array   | An ordered list of values.                       | `[1, 2, 3]`                 |
| Map     | A collection of key-value pairs.                 | `{"key": "value"}`         |
