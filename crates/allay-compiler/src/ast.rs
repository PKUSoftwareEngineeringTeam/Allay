#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File(pub Template);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub controls: Vec<Control>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    Text(String),
    ShortCode(ShortCode),
    Command(Command),
    Substitution(Substitution),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortCode {
    Single(SingleShortCode),
    Block(BlockShortCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
    pub inner: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Set(SetCommand),
    For(ForCommand),
    With(WithCommand),
    If(IfCommand),
    Include(IncludeCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCommand {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForCommand {
    pub item_name: String,
    pub index_name: Option<String>,
    pub list: Expression,
    pub inner: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithCommand {
    pub scope: Expression,
    pub inner: Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfCommand {
    pub condition: Expression,
    pub inner: Template,
    pub else_inner: Option<Template>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncludeCommand {
    pub path: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Substitution {
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression(pub Or);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Or(pub Vec<And>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And(pub Vec<Comparison>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    pub left: AddSub,
    pub right: Option<(ComparisonOp, AddSub)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddSub {
    pub left: MulDiv,
    pub rights: Vec<(AddSubOp, MulDiv)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddSubOp {
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MulDiv {
    pub left: Unary,
    pub rights: Vec<(MulDivOp, Unary)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MulDivOp {
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unary {
    Unary((UnaryOp, Primary)),
    Primary(Primary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primary {
    Field(Field),
    TopLevel(TopLevel),
    Number(i32),
    String(String),
    Boolean(bool),
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopLevel {
    This,
    Param,
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetField {
    Index(usize),
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub top_level: Option<TopLevel>,
    pub parts: Vec<GetField>, // at least one part
}
