// TODO: Remove this when everything is finished
#![allow(dead_code)]

pub struct File(pub Template);

pub struct Template {
    pub controls: Vec<Control>,
}

pub enum Control {
    Text(String),
    ShortCode(ShortCode),
    Command(Command),
    Substitution(Substitution),
}

pub enum ShortCode {
    Single(SingleShortCode),
    Block(BlockShortCode),
}

pub struct SingleShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
}

pub struct BlockShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
    pub inner: Template,
}

pub enum Command {
    Set(SetCommand),
    For(ForCommand),
    With(WithCommand),
    If(IfCommand),
    Include(IncludeCommand),
}

pub struct SetCommand {
    pub name: String,
    pub value: Expression,
}

pub struct ForCommand {
    pub item_name: String,
    pub index_name: Option<String>,
    pub list: Expression,
    pub inner: Template,
}

pub struct WithCommand {
    pub scope: Expression,
    pub inner: Template,
}

pub struct IfCommand {
    pub condition: Expression,
    pub inner: Template,
    pub else_inner: Option<Template>,
}

pub struct IncludeCommand {
    pub path: String,
    pub parameters: Vec<Expression>,
}

pub enum Substitution {
    Expression(Expression),
    Parameter(i32),
}

pub struct Expression(Or);

pub struct Or(Vec<And>);

pub struct And(Vec<Comparison>);

pub struct Comparison {
    pub left: AddSub,
    pub right: Option<(ComparisonOp, AddSub)>,
}

pub enum ComparisonOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

pub struct AddSub {
    pub left: MulDiv,
    pub right: Vec<(AddSubOp, MulDiv)>,
}

pub enum AddSubOp {
    Add,
    Subtract,
}

pub struct MulDiv {
    pub left: Unary,
    pub right: Vec<(MulDivOp, Unary)>,
}

pub enum MulDivOp {
    Multiply,
    Divide,
    Modulo,
}

pub enum Unary {
    Not(Primary),
    Negate((AddSubOp, Primary)),
    Primary(Primary),
}

pub enum Primary {
    Field(Field),
    TopLevel(TopLevel),
    Number(i32),
    String(String),
    Boolean(bool),
    Expression(Expression),
}

pub enum TopLevel {
    This,
    Variable(String),
}

pub struct Field {
    pub top_level: Option<TopLevel>,
    pub parts: Vec<String>,
}
