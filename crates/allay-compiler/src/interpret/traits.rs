#![allow(dead_code)] // TODO: remove this line when the module is complete

use crate::ast::GetField;
use crate::interpret::var::ThisVar;
use crate::{InterpretError, InterpretResult};
use allay_base::data::AllayData;

/// Utility function to get the field of the element once
pub(crate) fn get_field_once<'a>(
    cur: &'a AllayData,
    field: &GetField,
) -> InterpretResult<&'a AllayData> {
    match field {
        GetField::Index(i) => {
            let list = cur.as_list()?;
            list.get(*i).ok_or(InterpretError::IndexOutOfBounds(*i))
        }
        GetField::Name(name) => {
            let obj = cur.as_obj()?;
            obj.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))
        }
    }
}

/// A provider of data
pub(crate) trait DataProvider {
    /// Get the data of the element
    fn get_data(&self) -> &AllayData;

    /// Get the field of the element by a series of field names or indices
    fn get_field(&self, fields: &[GetField]) -> InterpretResult<&AllayData> {
        fields.iter().try_fold(self.get_data(), get_field_once)
    }
}

/// A variable in the template, which can provide data
pub(crate) trait Variable: DataProvider {
    /// What the element renders to in template
    fn render(&self) -> String {
        self.get_data().to_string()
    }
}

/// The variable scope for template, organized as a tree like json object
/// [`DataProvider`] here is the `this` variable
///
/// # Example
/// Current scope:
/// ```json
/// {
///  "title": "Hello, world!",
///  "author": {
///    "name": "John Doe",
///    "age": 30
///  },
///  "tags": ["test", "markdown"]
/// }
/// ```
///
/// Then the template can be like:
/// ```html
/// <!-- visit variables by dot notation -->
/// <h1>{: .title :}</h1>
///
/// <!-- use "for" to iterate a list -->
/// {- for $tag: .tags -}
/// <span>{: $tag :}</span>
/// {- end -}
///
/// <!-- use "with" to visit a child scope -->
/// {- with .author -}
/// <p>Author: {: .name :}, Age: {: .age :}</p>
/// {- end -}
/// ```
pub(crate) trait Scope: DataProvider {
    fn create_this(&self) -> ThisVar<'_>
    where
        Self: Sized,
    {
        ThisVar::create(self)
    }
}
