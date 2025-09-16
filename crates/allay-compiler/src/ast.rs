#[derive(Debug, PartialEq)]
pub struct File(pub Template);

#[derive(Debug, PartialEq)]
pub struct Template {
    pub controls: Vec<Control>,
}

#[derive(Debug, PartialEq)]
pub enum Control {
    Text(String),
    ShortCode(ShortCode),
    Command(Command),
    Substitution(Substitution),
}

#[derive(Debug, PartialEq)]
pub enum ShortCode {
    Single(SingleShortCode),
    Block(BlockShortCode),
}

#[derive(Debug, PartialEq)]
pub struct SingleShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct BlockShortCode {
    pub name: String,
    pub parameters: Vec<Expression>,
    pub inner: Template,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Set(SetCommand),
    For(ForCommand),
    With(WithCommand),
    If(IfCommand),
    Include(IncludeCommand),
}

#[derive(Debug, PartialEq)]
pub struct SetCommand {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct ForCommand {
    pub item_name: String,
    pub index_name: Option<String>,
    pub list: Expression,
    pub inner: Template,
}

#[derive(Debug, PartialEq)]
pub struct WithCommand {
    pub scope: Expression,
    pub inner: Template,
}

#[derive(Debug, PartialEq)]
pub struct IfCommand {
    pub condition: Expression,
    pub inner: Template,
    pub else_inner: Option<Template>,
}

#[derive(Debug, PartialEq)]
pub struct IncludeCommand {
    pub path: String,
    pub parameters: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct Substitution {
    pub expr: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Expression(pub Or);

#[derive(Debug, PartialEq)]
pub struct Or(pub Vec<And>);

#[derive(Debug, PartialEq)]
pub struct And(pub Vec<Comparison>);

#[derive(Debug, PartialEq)]
pub struct Comparison {
    pub left: AddSub,
    pub right: Option<(ComparisonOp, AddSub)>,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug, PartialEq)]
pub struct AddSub {
    pub left: MulDiv,
    pub rights: Vec<(AddSubOp, MulDiv)>,
}

#[derive(Debug, PartialEq)]
pub enum AddSubOp {
    Add,
    Subtract,
}

#[derive(Debug, PartialEq)]
pub struct MulDiv {
    pub left: Unary,
    pub rights: Vec<(MulDivOp, Unary)>,
}

#[derive(Debug, PartialEq)]
pub enum MulDivOp {
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Positive,
    Negative,
}

#[derive(Debug, PartialEq)]
pub enum Unary {
    Unary((UnaryOp, Primary)),
    Primary(Primary),
}

#[derive(Debug, PartialEq)]
pub enum Primary {
    Field(Field),
    TopLevel(TopLevel),
    Number(i32),
    String(String),
    Boolean(bool),
    Expression(Expression),
}

#[derive(Debug, PartialEq)]
pub enum TopLevel {
    This,
    Param,
    Variable(String),
}

#[derive(Debug, PartialEq)]
pub enum GetField {
    Index(i32),
    Name(String),
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub top_level: Option<TopLevel>,
    pub parts: Vec<GetField>,
}
