use crate::env::{Compiled, Page, TokenInserter};
use crate::interpret::scope::PageScope;
use crate::interpret::traits::{DataProvider, Variable};
use crate::interpret::var::{LocalVar, SiteVar};
use crate::{InterpretError, InterpretResult};
use crate::{ast::*, magic};
use allay_base::data::AllayData;
use allay_base::data::{AllayDataError, AllayList};
use itertools::Itertools;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn converse_error(err: String) -> InterpretError {
    InterpretError::DataError(AllayDataError::TypeConversion(err))
}

macro_rules! interpret_unreachable {
    () => {
        unreachable!(
            "This is a bug of interpreter, please report it to the developers on https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."
        )
    }
}

macro_rules! interpret_unwrap {
    ($expr: expr) => {
        $expr.unwrap_or_else(|_| interpret_unreachable!())
    };
}

/// The global Allay interpreter context
#[derive(Debug)]
pub struct Interpreter {
    include_dir: PathBuf,
    shortcode_dir: PathBuf,
}

impl Interpreter {
    /// Create a new interpreter with the given include and shortcode directories
    pub fn new(include_dir: PathBuf, shortcode_dir: PathBuf) -> Interpreter {
        Interpreter {
            include_dir,
            shortcode_dir,
        }
    }
}

/// The main trait for interpreting AST nodes
pub trait Interpretable {
    /// The return of the interpretation
    type Output;

    /// The main interpretation function
    ///
    /// # Parameters
    /// - `ctx`: The global interpreter context
    /// - `page`: The page to interpret on
    ///
    /// # Returns
    /// The interpretation result
    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Self::Output>;
}

impl Interpretable for Template {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        self.controls.iter().try_for_each(|c| c.interpret(ctx, page))
    }
}

impl Interpretable for Control {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        match self {
            Control::Text(text) => {
                page.insert_text(text.clone());
                Ok(())
            }
            Control::Command(cmd) => cmd.interpret(ctx, page),
            Control::Substitution(sub) => sub.interpret(ctx, page),
            Control::Shortcode(sc) => sc.interpret(ctx, page),
        }
    }
}

impl Interpretable for Command {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        match self {
            Command::Set(cmd) => cmd.interpret(ctx, page),
            Command::For(cmd) => cmd.interpret(ctx, page),
            Command::With(cmd) => cmd.interpret(ctx, page),
            Command::If(cmd) => cmd.interpret(ctx, page),
            Command::Include(cmd) => cmd.interpret(ctx, page),
        }
    }
}

impl Interpretable for SetCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let value = self.value.interpret(ctx, page)?;
        interpret_unwrap!(page.lock())
            .scope_mut()
            .cur_scope_mut()
            .create_local(self.name.clone(), value);
        Ok(())
    }
}

impl Interpretable for ForCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let list = self.list.interpret(ctx, page)?;
        let list = list.as_list()?;
        for (index, item) in list.iter().enumerate() {
            interpret_unwrap!(page.lock())
                .scope_mut()
                .cur_scope_mut()
                .create_local(self.item_name.clone(), item.clone());
            if let Some(index_name) = &self.index_name {
                let index = Arc::new((index as i32).into());
                interpret_unwrap!(page.lock())
                    .scope_mut()
                    .cur_scope_mut()
                    .create_local(index_name.clone(), index);
            }
            self.inner.interpret(ctx, page)?;
        }
        Ok(())
    }
}

impl Interpretable for WithCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let scope_data = self.scope.interpret(ctx, page)?;
        let var = LocalVar::create(scope_data);
        {
            interpret_unwrap!(page.lock()).scope_mut().create_sub_scope(var);
        }
        self.inner.interpret(ctx, page)?;
        interpret_unwrap!(page.lock()).scope_mut().exit_sub_scope();
        Ok(())
    }
}

impl Interpretable for IfCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let cond = self.condition.interpret(ctx, page)?.as_bool()?;
        if cond {
            self.inner.interpret(ctx, page)
        } else if let Some(else_branch) = &self.else_inner {
            else_branch.interpret(ctx, page)
        } else {
            Ok(())
        }
    }
}

mod file_finder {
    use crate::{InterpretError, InterpretResult};
    use allay_base::template::TemplateKind;
    use std::path::{Path, PathBuf};

    pub(super) fn find_file<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        for ext in [
            TemplateKind::Markdown.extension(),
            TemplateKind::Html.extension(),
        ] {
            let p = path.as_ref().with_extension(ext);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    pub(super) fn try_find_file<P: AsRef<Path>>(path: P) -> InterpretResult<PathBuf> {
        let path = path.as_ref();
        find_file(path).ok_or(InterpretError::IncludePathNotFound(
            path.to_path_buf().to_string_lossy().to_string(),
        ))
    }
}

impl Interpretable for IncludeCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let inherited = match self.parameters.first() {
            Some(exp) => exp.interpret(ctx, page)?,
            None => interpret_unwrap!(page.lock()).scope().cur_scope().create_this().get_data(),
        };

        // from 1...n are params
        let params = if self.parameters.len() > 1 {
            self.parameters[1..].iter().map(|e| e.interpret(ctx, page)).try_collect()?
        } else {
            AllayList::default()
        };

        let scope = PageScope::new_from(Arc::new(AllayData::arc_to_obj(inherited)?), params);
        let path = file_finder::try_find_file(ctx.include_dir.join(&self.path))?;
        page.insert_subpage(path, scope);
        Ok(())
    }
}

impl Interpretable for Shortcode {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        match self {
            Shortcode::Single(sc) => sc.interpret(ctx, page),
            Shortcode::Block(sc) => sc.interpret(ctx, page),
        }
    }
}

impl Interpretable for SingleShortcode {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let params = self.parameters.iter().map(|e| e.interpret(ctx, page)).try_collect()?;
        let inherited = interpret_unwrap!(page.lock()).scope().cur_scope().create_this().get_data();

        let scope = PageScope::new_from(Arc::new(AllayData::arc_to_obj(inherited)?), params);
        let path = file_finder::try_find_file(ctx.shortcode_dir.join(&self.name))?;
        page.insert_subpage(path, scope);

        Ok(())
    }
}

impl Interpretable for BlockShortcode {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let params = self.parameters.iter().map(|e| e.interpret(ctx, page)).try_collect()?;
        let inherited = interpret_unwrap!(page.lock()).scope().cur_scope().create_this().get_data();

        let mut scope = PageScope::new_from(Arc::new(AllayData::arc_to_obj(inherited)?), params);

        // add the "inner" key to the shortcode page
        // Do not use the lazy evaluation here, because the inner text may be modified later
        // FIXME: If the inner text of the shortcode is modified after this point (e.g., during hot reload), those changes will not be reflected,
        // because the "inner" key is set to the current compiled value. This may cause stale content to appear after hot reloads.
        let inner_page = interpret_unwrap!(page.lock()).clone_detached();
        let inner_page = Arc::new(Mutex::new(inner_page));
        let inner = inner_page
            .compile_on(&self.inner, ctx)
            .map_err(|e| InterpretError::IncludeError(Box::new(e)))?;
        scope.add_key(magic::INNER.into(), Arc::new(AllayData::from(inner)));

        let path = file_finder::try_find_file(ctx.shortcode_dir.join(&self.name))?;
        page.insert_subpage(path, scope);
        Ok(())
    }
}

impl Interpretable for Substitution {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, page: &Arc<Mutex<Page>>) -> InterpretResult<()> {
        let value = self.expr.interpret(ctx, page)?;
        if !value.is_null() {
            page.insert_text(value.to_string());
        }
        Ok(())
    }
}

impl Interpretable for Expression {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        self.0.interpret(ctx, page)
    }
}

impl Interpretable for Or {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        let ands = &self.0;
        if ands.is_empty() {
            return Ok(Arc::new(AllayData::default()));
        }
        let value = ands[0].interpret(ctx, page);
        if ands.len() == 1 {
            return value;
        }

        // short-circuit evaluation
        let bool = value?.as_bool()?;
        if bool {
            return Ok(Arc::new(true.into()));
        }
        for and in &ands[1..] {
            let v = and.interpret(ctx, page)?.as_bool()?;
            if v {
                return Ok(Arc::new(true.into()));
            }
        }
        Ok(Arc::new(false.into()))
    }
}

impl Interpretable for And {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        let comps = &self.0;
        if comps.is_empty() {
            return Ok(Arc::new(AllayData::default()));
        }
        let value = comps[0].interpret(ctx, page);
        if comps.len() == 1 {
            return value;
        }

        // short-circuit evaluation
        let bool = value?.as_bool()?;
        if !bool {
            return Ok(Arc::new(false.into()));
        }
        for comp in &comps[1..] {
            let v = comp.interpret(ctx, page)?.as_bool()?;
            if !v {
                return Ok(Arc::new(false.into()));
            }
        }
        Ok(Arc::new(true.into()))
    }
}

impl Interpretable for Comparison {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.right.is_none() {
            return self.left.interpret(ctx, page);
        }

        let left = self.left.interpret(ctx, page)?;
        let (op, right) = self.right.as_ref().unwrap();
        let right = right.interpret(ctx, page)?;

        let bool = match op {
            ComparisonOp::Equal => left == right,
            ComparisonOp::NotEqual => left != right,
            ComparisonOp::Greater => left > right,
            ComparisonOp::GreaterEqual => left >= right,
            ComparisonOp::Less => left < right,
            ComparisonOp::LessEqual => left <= right,
        };
        Ok(Arc::new(bool.into()))
    }
}

impl Interpretable for AddSub {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.rights.is_empty() {
            return self.left.interpret(ctx, page);
        }

        let res = self.rights.iter().try_fold(
            self.left.interpret(ctx, page)?.as_int()?,
            |acc, (op, right)| {
                let v = right.interpret(ctx, page)?.as_int()?;
                match op {
                    AddSubOp::Add => Ok(acc + v),
                    AddSubOp::Subtract => Ok(acc - v),
                }
            },
        );

        res.map(|v| Arc::new(v.into()))
    }
}

impl Interpretable for MulDiv {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.rights.is_empty() {
            return self.left.interpret(ctx, page);
        }

        let res = self.rights.iter().try_fold(
            self.left.interpret(ctx, page)?.as_int()?,
            |acc, (op, right)| {
                let v = right.interpret(ctx, page)?.as_int()?;
                match op {
                    MulDivOp::Multiply => Ok(acc * v),
                    MulDivOp::Divide => Ok(acc / v),
                    MulDivOp::Modulo => Ok(acc % v),
                }
            },
        );

        res.map(|v| Arc::new(v.into()))
    }
}

impl Interpretable for Unary {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.ops.is_empty() {
            return self.exp.interpret(ctx, page);
        }

        let data = self.exp.interpret(ctx, page)?;
        if data.is_int() {
            self.ops
                .iter()
                .rev()
                .try_fold(data.as_int()?, |acc, op| match op {
                    UnaryOp::Positive => Ok(acc),
                    UnaryOp::Negative => Ok(-acc),
                    UnaryOp::Not => Err(converse_error("not a boolean".into())),
                })
                .map(|v| Arc::new(v.into()))
        } else if data.is_bool() {
            self.ops
                .iter()
                .rev()
                .try_fold(data.as_bool()?, |acc, op| match op {
                    UnaryOp::Not => Ok(!acc),
                    UnaryOp::Positive | UnaryOp::Negative => {
                        Err(converse_error("not an integer".into()))
                    }
                })
                .map(|v| Arc::new(v.into()))
        } else {
            Err(converse_error("not an integer or a boolean".into()))
        }
    }
}

impl Interpretable for Primary {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Arc<AllayData>> {
        match self {
            Primary::Number(num) => Ok(Arc::new((*num as i32).into())),
            Primary::Boolean(bool) => Ok(Arc::new((*bool).into())),
            Primary::String(str) => Ok(Arc::new(str.clone().into())),
            Primary::Expression(exp) => exp.interpret(ctx, page),
            Primary::Field(field) => field.interpret(ctx, page),
            Primary::TopLevel(top) => top.interpret(ctx, page),
            Primary::Null => Ok(Arc::new(AllayData::Null)),
        }
    }
}

impl Interpretable for Field {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        _: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Self::Output> {
        // check magic fields
        if self.top_level.is_none()
            && self.parts.len() == 1
            && let GetField::Name(name) = &self.parts[0]
            && page.insert_stash(name).is_some()
        {
            return Ok(Arc::new(AllayData::default()));
        }

        let page = interpret_unwrap!(page.lock());
        let scope = page.scope();
        let var: &dyn Variable = match &self.top_level {
            None | Some(TopLevel::This) => &scope.cur_scope().create_this(),
            Some(TopLevel::Site) => SiteVar::get_instance(),
            Some(TopLevel::Param) => scope.get_param(),
            Some(TopLevel::Variable(id)) => {
                scope.get_local(id).ok_or(InterpretError::VariableNotFound(id.clone()))?
            }
        };
        var.get_field(&self.parts)
    }
}

impl Interpretable for TopLevel {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        _: &mut Interpreter,
        page: &Arc<Mutex<Page>>,
    ) -> InterpretResult<Self::Output> {
        let page = interpret_unwrap!(page.lock());
        let scope = page.scope();
        let var: &dyn Variable = match self {
            TopLevel::This => &scope.cur_scope().create_this(),
            TopLevel::Site => SiteVar::get_instance(),
            TopLevel::Param => scope.get_param(),
            TopLevel::Variable(id) => {
                scope.get_local(id).ok_or(InterpretError::VariableNotFound(id.clone()))?
            }
        };
        Ok(var.get_data())
    }
}
