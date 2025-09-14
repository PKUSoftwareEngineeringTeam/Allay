## Variables

### Types

All variable types in Allay templates include:

| Type    | Description                                      | Example                     |
|---------|--------------------------------------------------|-----------------------------|
| String  | A sequence of characters.                        | `"Hello, World!"`           |
| Integer | A whole number.                                  | `42`                        |
| Boolean | A true or false value.                           | `true` or `false`           |
| Array   | An ordered list of values.                       | `[1, 2, 3]`                 |
| Map     | A collection of key-value pairs.                 | `{"key": "value"}`         |

### Preset Variables

The variable of the current scope can be accessed by `this`, which can usually be omitted.

The variable of the global scope can be accessed by `site`.

### Custom Variables

All variables in allay template should be started with `$`. To define a variable, use the following syntax:

```html
{- set $my_variable "Hello, World" -}
<p>{: $my_variable :}</p>
```

The `set` directive assigns a value to a variable, and the `:` syntax is used to output the value of the variable. In this example, the output will be:

```html
<p>Hello, World</p>
```

### Accessing Object Fields

You can access the fields of an object using `.` notation. For example, if the current scope variable is:

```json
{
    "name": "Alice",
    "age": 30,
    "address": {
        "city": "Wonderland",
        "zip": "12345"
    }
}
```

Then in template:

```html
<p>{: this.name :}</p>       <!-- Outputs "Alice" -->
<p>{: .name :}</p>       <!-- Omits "this" and outputs "Alice" -->
<p>{: .address.city :}</p> <!-- Outputs "Wonderland" -->
{- set $addr .address -}
<p>{: $addr.zip :}</p>      <!-- Outputs "12345" -->
```
