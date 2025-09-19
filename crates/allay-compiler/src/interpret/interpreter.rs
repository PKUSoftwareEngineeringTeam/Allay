#![allow(dead_code)] // TODO: remove this line when the interpreter is complete

use crate::ast::*;
use crate::interpret::scope::PageScope;
use crate::interpret::traits::Variable;
use crate::interpret::var::LocalVar;
use crate::{InterpretError, InterpretResult};
use allay_base::data::AllayDataError;
use allay_base::data::{AllayData, AllayObject};
use std::path::PathBuf;
use std::sync::Arc;

fn converse_error(err: String) -> InterpretError {
    InterpretError::DataError(AllayDataError::TypeConversion(err))
}

macro_rules! interpret_unreachable {
    () => {
        unreachable!(
            "This is a bug of AST interpreter, please report it to the developers on https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."
        )
    }
}

macro_rules! interpret_unwrap {
    ($expr: expr) => {
        $expr.unwrap_or_else(|| interpret_unreachable!())
    };
}

/// The global Allay interpreter context
#[derive(Debug)]
pub(super) struct Interpreter {
    stack: Vec<PageScope>,

    include_dir: PathBuf,
    shortcode_dir: PathBuf,
}

impl Interpreter {
    /// Create a new interpreter with the given include and shortcode directories
    pub fn new(include_dir: PathBuf, shortcode_dir: PathBuf) -> Interpreter {
        Interpreter {
            stack: Vec::new(),
            include_dir,
            shortcode_dir,
        }
    }

    /// Create a news page subscope
    fn new_page(&mut self, page: PageScope) {
        self.stack.push(page);
    }

    /// Get the current page scope
    fn page(&self) -> &PageScope {
        interpret_unwrap!(self.stack.last())
    }

    /// Get the current page scope mutably
    fn page_mut(&mut self) -> &mut PageScope {
        interpret_unwrap!(self.stack.last_mut())
    }

    /// Exit the current page scope
    fn exit_page(&mut self) -> Option<PageScope> {
        self.stack.pop()
    }
}

/// The main trait for interpreting AST nodes
pub(super) trait Interpretable {
    /// The output type of the interpretation
    type Output;

    /// The main interpretation function
    ///
    /// # Parameters
    /// - `ctx`: The global interpreter context
    /// - `res`: The string builder to accumulate the rendered result
    ///
    /// # Returns
    /// The interpretation result
    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Self::Output>;
}

impl Interpretable for File {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        // TODO: pass the real page data here
        let page = PageScope::new(AllayObject::default());

        ctx.new_page(page);
        self.0.interpret(ctx, res)?;
        interpret_unwrap!(ctx.exit_page());
        Ok(())
    }
}

impl Interpretable for Template {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        self.controls.iter().try_for_each(|c| c.interpret(ctx, res))
    }
}

impl Interpretable for Control {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        match self {
            Control::Text(text) => {
                res.push(text.clone());
                Ok(())
            }
            Control::Command(cmd) => cmd.interpret(ctx, res),
            Control::Substitution(sub) => sub.interpret(ctx, res),
            Control::Shortcode(sc) => sc.interpret(ctx, res),
        }
    }
}

impl Interpretable for Command {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        match self {
            Command::Set(cmd) => cmd.interpret(ctx, res),
            Command::For(cmd) => cmd.interpret(ctx, res),
            Command::With(cmd) => cmd.interpret(ctx, res),
            Command::If(cmd) => cmd.interpret(ctx, res),
            Command::Include(cmd) => cmd.interpret(ctx, res),
        }
    }
}

impl Interpretable for SetCommand {
    type Output = ();

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Self::Output> {
        let value = self.value.interpret(ctx, res)?;
        ctx.page_mut().cur_scope_mut().create_local(self.name.clone(), value);
        Ok(())
    }
}

impl Interpretable for ForCommand {
    type Output = ();

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Self::Output> {
        let list = self.list.interpret(ctx, res)?;
        let list = list.as_list()?;
        for (index, item) in list.iter().enumerate() {
            ctx.page_mut()
                .cur_scope_mut()
                .create_local(self.item_name.clone(), item.clone());
            if let Some(index_name) = &self.index_name {
                let index = Arc::new((index as i32).into());
                ctx.page_mut().cur_scope_mut().create_local(index_name.clone(), index);
            }
            self.inner.interpret(ctx, res)?;
        }
        Ok(())
    }
}

impl Interpretable for WithCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        let scope_data = self.scope.interpret(ctx, res)?;
        let var = LocalVar::create(scope_data);
        ctx.page_mut().create_sub_scope(var);
        self.inner.interpret(ctx, res)?;
        interpret_unwrap!(ctx.page_mut().exit_sub_scope());
        Ok(())
    }
}

impl Interpretable for IfCommand {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        let cond = self.condition.interpret(ctx, res)?.as_bool()?;
        if cond {
            self.inner.interpret(ctx, res)
        } else if let Some(else_branch) = &self.else_inner {
            else_branch.interpret(ctx, res)
        } else {
            Ok(())
        }
    }
}

impl Interpretable for IncludeCommand {
    type Output = ();

    fn interpret(&self, _: &mut Interpreter, _: &mut Vec<String>) -> InterpretResult<()> {
        todo!()
    }
}

impl Interpretable for Shortcode {
    type Output = ();

    fn interpret(&self, _: &mut Interpreter, _: &mut Vec<String>) -> InterpretResult<()> {
        todo!()
    }
}

impl Interpretable for Substitution {
    type Output = ();

    fn interpret(&self, ctx: &mut Interpreter, res: &mut Vec<String>) -> InterpretResult<()> {
        let value = self.expr.interpret(ctx, res)?;
        res.push(value.to_string());
        Ok(())
    }
}

impl Interpretable for Expression {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        self.0.interpret(ctx, res)
    }
}

impl Interpretable for Or {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        let ands = &self.0;
        if ands.is_empty() {
            return Ok(Arc::new(AllayData::default()));
        }
        let value = ands[0].interpret(ctx, res);
        if ands.len() == 1 {
            return value;
        }

        // short-circuit evaluation
        let bool = value?.as_bool()?;
        if bool {
            return Ok(Arc::new(true.into()));
        }
        for and in &ands[1..] {
            let v = and.interpret(ctx, res)?.as_bool()?;
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
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        let comps = &self.0;
        if comps.is_empty() {
            return Ok(Arc::new(AllayData::default()));
        }
        let value = comps[0].interpret(ctx, res);
        if comps.len() == 1 {
            return value;
        }

        // short-circuit evaluation
        let bool = value?.as_bool()?;
        if !bool {
            return Ok(Arc::new(false.into()));
        }
        for comp in &comps[1..] {
            let v = comp.interpret(ctx, res)?.as_bool()?;
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
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.right.is_none() {
            return self.left.interpret(ctx, res);
        }

        let left = self.left.interpret(ctx, res)?.as_int()?;
        let (op, right) = self.right.as_ref().unwrap();
        let right = right.interpret(ctx, res)?.as_int()?;

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
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.rights.is_empty() {
            return self.left.interpret(ctx, res);
        }

        let mut acc = self.left.interpret(ctx, res)?.as_int()?;
        for (op, right) in &self.rights {
            let v = right.interpret(ctx, res)?.as_int()?;
            match op {
                AddSubOp::Add => acc += v,
                AddSubOp::Subtract => acc -= v,
            }
        }
        Ok(Arc::new(acc.into()))
    }
}

impl Interpretable for MulDiv {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.rights.is_empty() {
            return self.left.interpret(ctx, res);
        }

        let mut acc = self.left.interpret(ctx, res)?.as_int()?;
        for (op, right) in &self.rights {
            let v = right.interpret(ctx, res)?.as_int()?;
            match op {
                MulDivOp::Multiply => acc *= v,
                MulDivOp::Divide => acc /= v,
                MulDivOp::Modulo => acc %= v,
            }
        }
        Ok(Arc::new(acc.into()))
    }
}

impl Interpretable for Unary {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        if self.ops.is_empty() {
            return self.exp.interpret(ctx, res);
        }

        let data = self.exp.interpret(ctx, res)?;
        if data.is_int() {
            let mut acc = data.as_int()?;
            for op in self.ops.iter().rev() {
                match op {
                    UnaryOp::Positive => {}
                    UnaryOp::Negative => acc = -acc,
                    UnaryOp::Not => {
                        return Err(converse_error("not a boolean".into()));
                    }
                }
            }
            Ok(Arc::new(acc.into()))
        } else if data.is_bool() {
            let mut bool = data.as_bool()?;
            for op in self.ops.iter().rev() {
                match op {
                    UnaryOp::Positive | UnaryOp::Negative => {
                        return Err(converse_error("not an integer".into()));
                    }
                    UnaryOp::Not => bool = !bool,
                }
            }
            Ok(Arc::new(bool.into()))
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
        res: &mut Vec<String>,
    ) -> InterpretResult<Arc<AllayData>> {
        match self {
            Primary::Number(num) => Ok(Arc::new((*num as i32).into())),
            Primary::Boolean(bool) => Ok(Arc::new((*bool).into())),
            Primary::String(str) => Ok(Arc::new(str.clone().into())),
            Primary::Expression(exp) => exp.interpret(ctx, res),
            Primary::Field(field) => field.interpret(ctx, res),
            Primary::TopLevel(top) => top.interpret(ctx, res),
        }
    }
}

impl Interpretable for Field {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        _: &mut Vec<String>,
    ) -> InterpretResult<Self::Output> {
        let var: &dyn Variable = match &self.top_level {
            None | Some(TopLevel::This) => &ctx.page().cur_scope().create_this(),
            Some(TopLevel::Param) => ctx.page().get_param(),
            Some(TopLevel::Variable(id)) => {
                ctx.page().get_local(id).ok_or(InterpretError::VariableNotFound(id.clone()))?
            }
        };
        var.get_field(&self.parts)
    }
}

impl Interpretable for TopLevel {
    type Output = Arc<AllayData>;

    fn interpret(
        &self,
        ctx: &mut Interpreter,
        _: &mut Vec<String>,
    ) -> InterpretResult<Self::Output> {
        let var: &dyn Variable = match self {
            TopLevel::This => &ctx.page().cur_scope().create_this(),
            TopLevel::Param => ctx.page().get_param(),
            TopLevel::Variable(id) => {
                ctx.page().get_local(id).ok_or(InterpretError::VariableNotFound(id.clone()))?
            }
        };
        Ok(var.get_data())
    }
}
