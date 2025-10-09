use crate::ast::GetField;
use crate::interpret::var::{LocalVar, ThisVar};
use crate::{InterpretError, InterpretResult};
use allay_base::data::AllayData;
use std::sync::Arc;

/// Utility function to get the field of the element once
pub(crate) fn get_field_once(
    cur: Arc<AllayData>,
    field: &GetField,
) -> InterpretResult<Arc<AllayData>> {
    match field {
        GetField::Index(i) => {
            let list = cur.as_list()?;
            list.get(*i).map(Arc::clone).ok_or(InterpretError::IndexOutOfBounds(*i))
        }
        GetField::Name(name) => {
            let obj = cur.as_obj()?;
            Ok(obj.get(name).map(Arc::clone).unwrap_or(Arc::new(AllayData::Null)))
        }
    }
}

/// A provider of data
pub(crate) trait DataProvider {
    /// Get the data of the element
    fn get_data(&self) -> Arc<AllayData>;

    /// Get the field of the element by a series of field names or indices
    fn get_field(&self, fields: &[GetField]) -> InterpretResult<Arc<AllayData>> {
        fields.iter().try_fold(self.get_data(), get_field_once)
    }
}

/// A variable in the template, which can provide data
pub(crate) trait Variable: DataProvider {}

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
    /// Create the special variable `this`, which is the current scope itself
    fn create_this(&self) -> ThisVar<'_>;

    /// Create a local variable defined in template, like `for $item: .items`
    fn create_local_var(&mut self, id: String, data: LocalVar);

    /// A utility function to create a local variable from an [`AllayData`]
    fn create_local(&mut self, id: String, data: Arc<AllayData>) {
        self.create_local_var(id, LocalVar::create(data));
    }
}
