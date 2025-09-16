use crate::CompileResult;
use crate::ast::*;
use crate::error::CompileError;
use itertools::Itertools;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

/// The template parser using Pest
#[derive(Parser)]
#[grammar = "allay.pest"]
struct TemplateParser;

pub(crate) fn parse_template(source: &str) -> Result<File, CompileError> {
    let tokens = TemplateParser::parse(Rule::file, source).map_err(Box::new)?.next().unwrap();
    File::build(tokens)
}

macro_rules! parser_unreachable {
    () => {
        unreachable!(
            "This is a bug of AST parser, please report it to the developers on \
            https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."
        )
    };
}

macro_rules! parser_unwarp {
    ($expr: expr) => {
        $expr.unwrap_or_else(|| {
            unreachable!(
                "This is a bug of AST parser, please report it to the developers on \
                https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."
            )
        })
    };
}

fn single_inner(pair: Pair<Rule>) -> Pair<Rule> {
    parser_unwarp!(pair.into_inner().next())
}

trait ASTBuilder {
    type Output;

    fn build(pair: Pair<Rule>) -> CompileResult<Self::Output>;
}

impl ASTBuilder for File {
    type Output = File;

    fn build(pair: Pair<Rule>) -> CompileResult<File> {
        Ok(File(Template::build(single_inner(pair))?))
    }
}

impl ASTBuilder for Template {
    type Output = Template;

    fn build(pair: Pair<Rule>) -> CompileResult<Template> {
        let controls = pair
            .into_inner()
            .filter_map(|item| match item.as_rule() {
                Rule::control => Some(Control::build(item)),
                Rule::EOI => None,
                _ => parser_unreachable!(),
            })
            .collect::<Result<Vec<Control>, CompileError>>()?;
        Ok(Template { controls })
    }
}

impl ASTBuilder for Control {
    type Output = Control;

    fn build(pair: Pair<Rule>) -> CompileResult<Control> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::text => Ok(Control::Text(inner.as_str().to_string())),
            Rule::short_code => Ok(Control::ShortCode(ShortCode::build(inner)?)),
            Rule::command => Ok(Control::Command(Command::build(inner)?)),
            Rule::substitution => Ok(Control::Substitution(Substitution::build(inner)?)),
            _ => parser_unreachable!(),
        }
    }
}

impl ASTBuilder for ShortCode {
    type Output = ShortCode;

    fn build(pair: Pair<Rule>) -> CompileResult<ShortCode> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::single_short_code => Ok(ShortCode::Single(SingleShortCode::build(inner)?)),
            Rule::block_short_code => Ok(ShortCode::Block(BlockShortCode::build(inner)?)),
            _ => parser_unreachable!(),
        }
    }
}

fn get_inner_str(pair: Pair<Rule>) -> String {
    single_inner(pair).as_str().to_string()
}

impl ASTBuilder for SingleShortCode {
    type Output = SingleShortCode;

    fn build(pair: Pair<Rule>) -> CompileResult<SingleShortCode> {
        let mut name = String::new();
        let mut parameters = vec![];
        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::short_code_pattern => {
                    name = get_inner_str(inner);
                }
                Rule::expression => {
                    parameters.push(Expression::build(inner)?);
                }
                _ => parser_unreachable!(),
            }
        }
        Ok(SingleShortCode { name, parameters })
    }
}

impl ASTBuilder for BlockShortCode {
    type Output = BlockShortCode;

    fn build(pair: Pair<Rule>) -> CompileResult<BlockShortCode> {
        let mut name = String::new();
        let mut end_name = String::new();
        let mut parameters = vec![];
        let mut inner_template = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::short_code_pattern => {
                    name = get_inner_str(inner);
                }
                Rule::expression => {
                    parameters.push(Expression::build(inner)?);
                }
                Rule::template => {
                    inner_template = Some(Template::build(inner)?);
                }
                Rule::identifier => {
                    end_name = inner.as_str().to_string();
                }
                _ => parser_unreachable!(),
            }
        }

        if name != end_name {
            return Err(CompileError::ShortCodeInconsistent(name));
        }

        Ok(BlockShortCode {
            name,
            parameters,
            inner: parser_unwarp!(inner_template),
        })
    }
}

impl ASTBuilder for Command {
    type Output = Command;

    fn build(pair: Pair<Rule>) -> CompileResult<Command> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::set_command => Ok(Command::Set(SetCommand::build(inner)?)),
            Rule::for_command => Ok(Command::For(ForCommand::build(inner)?)),
            Rule::with_command => Ok(Command::With(WithCommand::build(inner)?)),
            Rule::if_command => Ok(Command::If(IfCommand::build(inner)?)),
            Rule::include_command => Ok(Command::Include(IncludeCommand::build(inner)?)),
            _ => parser_unreachable!(),
        }
    }
}

impl ASTBuilder for SetCommand {
    type Output = SetCommand;

    fn build(pair: Pair<Rule>) -> CompileResult<SetCommand> {
        let mut name = String::new();
        let mut value = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::set_pattern => continue,
                Rule::variable => {
                    name = get_inner_str(inner);
                }
                Rule::expression => {
                    value = Some(Expression::build(inner)?);
                }
                _ => parser_unreachable!(),
            }
        }

        Ok(SetCommand {
            name,
            value: parser_unwarp!(value),
        })
    }
}

impl ASTBuilder for ForCommand {
    type Output = ForCommand;

    fn build(pair: Pair<Rule>) -> CompileResult<ForCommand> {
        let mut item_name = String::new();
        let mut index_name = None;
        let mut list = None;
        let mut inner_template = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::start_for_command => {
                    for item in inner.into_inner() {
                        match item.as_rule() {
                            Rule::for_pattern => continue,
                            Rule::variable => {
                                if item_name.is_empty() {
                                    item_name = get_inner_str(item);
                                } else {
                                    index_name = Some(get_inner_str(item));
                                }
                            }
                            Rule::expression => {
                                list = Some(Expression::build(item)?);
                            }
                            _ => parser_unreachable!(),
                        }
                    }
                }
                Rule::template => {
                    inner_template = Some(Template::build(inner)?);
                }
                Rule::end_command => continue,
                _ => parser_unreachable!(),
            }
        }

        Ok(ForCommand {
            item_name,
            index_name,
            list: parser_unwarp!(list),
            inner: parser_unwarp!(inner_template),
        })
    }
}

impl ASTBuilder for WithCommand {
    type Output = WithCommand;

    fn build(pair: Pair<Rule>) -> CompileResult<WithCommand> {
        let mut scope = None;
        let mut inner_template = None;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::start_with_command => {
                    for item in inner.into_inner() {
                        match item.as_rule() {
                            Rule::with_pattern => continue,
                            Rule::expression => {
                                scope = Some(Expression::build(item)?);
                            }
                            _ => parser_unreachable!(),
                        }
                    }
                }
                Rule::template => {
                    inner_template = Some(Template::build(inner)?);
                }
                Rule::end_command => continue,
                _ => parser_unreachable!(),
            }
        }

        Ok(WithCommand {
            scope: parser_unwarp!(scope),
            inner: parser_unwarp!(inner_template),
        })
    }
}

impl ASTBuilder for IfCommand {
    type Output = IfCommand;

    fn build(pair: Pair<Rule>) -> CompileResult<IfCommand> {
        let mut condition = None;
        let mut inner_template = None;
        let mut else_inner_template = None;
        let mut in_else = false;

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::start_if_command => {
                    for item in inner.into_inner() {
                        match item.as_rule() {
                            Rule::if_pattern => continue,
                            Rule::expression => {
                                condition = Some(Expression::build(item)?);
                            }
                            _ => parser_unreachable!(),
                        }
                    }
                }
                Rule::template => {
                    if in_else {
                        else_inner_template = Some(Template::build(inner)?);
                    } else {
                        inner_template = Some(Template::build(inner)?);
                    }
                }
                Rule::else_command => {
                    in_else = true;
                }
                Rule::end_command => continue,
                _ => parser_unreachable!(),
            }
        }

        Ok(IfCommand {
            condition: parser_unwarp!(condition),
            inner: parser_unwarp!(inner_template),
            else_inner: else_inner_template,
        })
    }
}

impl ASTBuilder for IncludeCommand {
    type Output = IncludeCommand;

    fn build(pair: Pair<Rule>) -> CompileResult<IncludeCommand> {
        let mut path = String::new();
        let mut parameters = vec![];

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::include_pattern => continue,
                Rule::string => {
                    path = inner.as_str().to_string();
                }
                Rule::expression => {
                    parameters.push(Expression::build(inner)?);
                }
                _ => parser_unreachable!(),
            }
        }

        Ok(IncludeCommand { path, parameters })
    }
}

impl ASTBuilder for Substitution {
    type Output = Substitution;

    fn build(pair: Pair<Rule>) -> CompileResult<Substitution> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::get_substitution => {
                for item in inner.into_inner() {
                    match item.as_rule() {
                        Rule::get_pattern => continue,
                        Rule::variable => {
                            return Ok(Substitution {
                                expr: Expression::build(item)?,
                            });
                        }
                        _ => parser_unreachable!(),
                    }
                }
                parser_unreachable!()
            }
            Rule::expr_substitution => {
                let inner = single_inner(inner);
                Ok(Substitution {
                    expr: Expression::build(inner)?,
                })
            }
            _ => parser_unreachable!(),
        }
    }
}

impl ASTBuilder for Expression {
    type Output = Expression;

    fn build(pair: Pair<Rule>) -> CompileResult<Expression> {
        Ok(Expression(Or::build(single_inner(pair))?))
    }
}

impl ASTBuilder for Or {
    type Output = Or;

    fn build(pair: Pair<Rule>) -> CompileResult<Or> {
        let ands = pair
            .into_inner()
            .filter_map(|item| match item.as_rule() {
                Rule::logic_or => None,
                Rule::logic_and => Some(And::build(item)),
                _ => parser_unreachable!(),
            })
            .collect::<Result<Vec<And>, CompileError>>()?;
        Ok(Or(ands))
    }
}

impl ASTBuilder for And {
    type Output = And;

    fn build(pair: Pair<Rule>) -> CompileResult<And> {
        let cmps = pair
            .into_inner()
            .filter_map(|item| match item.as_rule() {
                Rule::comparison => Some(Comparison::build(item)),
                Rule::logic_and => None,
                _ => parser_unreachable!(),
            })
            .collect::<Result<Vec<Comparison>, CompileError>>()?;
        Ok(And(cmps))
    }
}

impl ASTBuilder for Comparison {
    type Output = Comparison;

    fn build(pair: Pair<Rule>) -> CompileResult<Comparison> {
        let mut left = None;
        let mut operator = None;
        let mut right = None;

        for item in pair.into_inner() {
            match item.as_rule() {
                Rule::addition => {
                    if left.is_none() {
                        left = Some(AddSub::build(item)?);
                    } else {
                        right = Some(AddSub::build(item)?);
                    }
                }
                Rule::comparison_op => {
                    operator = match item.as_str() {
                        "==" => Some(ComparisonOp::Equal),
                        "!=" => Some(ComparisonOp::NotEqual),
                        ">" => Some(ComparisonOp::Greater),
                        "<" => Some(ComparisonOp::Less),
                        ">=" => Some(ComparisonOp::GreaterEqual),
                        "<=" => Some(ComparisonOp::LessEqual),
                        _ => parser_unreachable!(),
                    };
                }
                _ => parser_unreachable!(),
            }
        }

        Ok(Comparison {
            left: parser_unwarp!(left),
            right: operator.and_then(|op| right.map(|r| (op, r))),
        })
    }
}

impl ASTBuilder for AddSub {
    type Output = AddSub;

    fn build(pair: Pair<Rule>) -> CompileResult<AddSub> {
        let mut inner = pair.into_inner();
        let left = MulDiv::build(parser_unwarp!(inner.next()))?;
        let rights = inner
            .tuples()
            .map(|(op_pair, val_pair)| {
                let op = match op_pair.as_str() {
                    "+" => AddSubOp::Add,
                    "-" => AddSubOp::Subtract,
                    _ => parser_unreachable!(),
                };
                let val = MulDiv::build(val_pair)?;
                Ok((op, val))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;

        Ok(AddSub { left, rights })
    }
}

impl ASTBuilder for MulDiv {
    type Output = MulDiv;

    fn build(pair: Pair<Rule>) -> CompileResult<MulDiv> {
        let mut inner = pair.into_inner();
        let left = Unary::build(parser_unwarp!(inner.next()))?;
        let rights = inner
            .tuples()
            .map(|(op_pair, val_pair)| {
                let op = match op_pair.as_str() {
                    "*" => MulDivOp::Multiply,
                    "/" => MulDivOp::Divide,
                    "%" => MulDivOp::Modulo,
                    _ => parser_unreachable!(),
                };
                let val = Unary::build(val_pair)?;
                Ok((op, val))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;

        Ok(MulDiv { left, rights })
    }
}

impl ASTBuilder for Unary {
    type Output = Unary;

    fn build(pair: Pair<Rule>) -> CompileResult<Unary> {
        let mut inner = pair.into_inner();
        if inner.len() == 1 {
            Ok(Unary::Primary(Primary::build(inner.next().unwrap())?))
        } else if inner.len() == 2 {
            let op = match inner.next().unwrap().as_str() {
                "!" => UnaryOp::Not,
                "+" => UnaryOp::Positive,
                "-" => UnaryOp::Negative,
                _ => parser_unreachable!(),
            };
            let primary = Primary::build(inner.next().unwrap())?;
            Ok(Unary::Unary((op, primary)))
        } else {
            parser_unreachable!()
        }
    }
}

impl ASTBuilder for Primary {
    type Output = Primary;

    fn build(pair: Pair<Rule>) -> CompileResult<Primary> {
        let item = single_inner(pair);
        match item.as_rule() {
            Rule::field => Ok(Primary::Field(Field::build(item)?)),
            Rule::top_level => Ok(Primary::TopLevel(TopLevel::build(item)?)),
            Rule::number => {
                let num = item
                    .as_str()
                    .parse::<i32>()
                    .map_err(|e| CompileError::InvalidNumber(item.as_str().to_string(), e))?;
                Ok(Primary::Number(num))
            }
            Rule::string => Ok(Primary::String(item.as_str().to_string())),
            Rule::bool_literal => {
                let val = match item.as_str() {
                    "#t" => true,
                    "#f" => false,
                    _ => parser_unreachable!(),
                };
                Ok(Primary::Boolean(val))
            }
            Rule::expression => Ok(Primary::Expression(Expression::build(item)?)),
            _ => parser_unreachable!(),
        }
    }
}

impl ASTBuilder for Field {
    type Output = Field;

    fn build(pair: Pair<Rule>) -> CompileResult<Field> {
        let inner = pair.into_inner();
        let mut top_level = None;
        let mut get_fields = vec![];

        for item in inner {
            match item.as_rule() {
                Rule::top_level => {
                    top_level = Some(TopLevel::build(item)?);
                }
                Rule::get_field => {
                    get_fields.push(GetField::build(item)?);
                }
                _ => parser_unreachable!(),
            }
        }

        Ok(Field {
            top_level,
            parts: get_fields,
        })
    }
}

impl ASTBuilder for GetField {
    type Output = GetField;

    fn build(pair: Pair<Rule>) -> CompileResult<GetField> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::number => {
                let idx = inner
                    .as_str()
                    .parse::<i32>()
                    .map_err(|e| CompileError::InvalidNumber(inner.as_str().to_string(), e))?;
                Ok(GetField::Index(idx))
            }
            Rule::identifier => Ok(GetField::Name(inner.as_str().to_string())),
            _ => parser_unreachable!(),
        }
    }
}

impl ASTBuilder for TopLevel {
    type Output = TopLevel;

    fn build(pair: Pair<Rule>) -> CompileResult<TopLevel> {
        let inner = single_inner(pair);
        match inner.as_rule() {
            Rule::this => Ok(TopLevel::This),
            Rule::param => Ok(TopLevel::Param),
            Rule::variable => Ok(TopLevel::Variable(single_inner(inner).as_str().to_string())),
            _ => parser_unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_template;
    use crate::ast::*;
    use crate::error::CompileError;

    #[test]
    fn test_parse_only_text() {
        let source = "This is a simple text.";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![Control::Text(source.to_string())],
            })
        )
    }

    #[test]
    fn test_single_short_code() {
        let source = "This is a simple text. {< my_shortcode />}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![
                    Control::Text("This is a simple text.".to_string()),
                    Control::ShortCode(ShortCode::Single(SingleShortCode {
                        name: "my_shortcode".to_string(),
                        parameters: vec![],
                    })),
                ],
            })
        );
    }

    #[test]
    fn test_block_short_code() {
        let source = "This is a simple text. {< my_shortcode >}Inner content{</ my_shortcode >}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![
                    Control::Text("This is a simple text.".to_string()),
                    Control::ShortCode(ShortCode::Block(BlockShortCode {
                        name: "my_shortcode".to_string(),
                        parameters: vec![],
                        inner: Template {
                            controls: vec![Control::Text("Inner content".to_string()),],
                        },
                    })),
                ],
            })
        );
    }

    #[test]
    fn test_block_short_code_inconsistent() {
        let source = "{< my_shortcode >}Inner content{</ another_shortcode >}";
        let ast = parse_template(source);
        assert!(ast.is_err());

        let err = ast.err().unwrap();
        match err {
            CompileError::ShortCodeInconsistent(name) => {
                assert_eq!(name, "my_shortcode".to_string());
            }
            _ => panic!("Expected ShortCodeInconsistent error"),
        }
    }

    #[test]
    fn test_set_command() {
        let source = "{- set $my_var = 42 -}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![Control::Command(Command::Set(SetCommand {
                    name: "my_var".to_string(),
                    value: Expression(Or(vec![And(vec![Comparison {
                        left: AddSub {
                            left: MulDiv {
                                left: Unary::Primary(Primary::Number(42)),
                                rights: vec![],
                            },
                            rights: vec![],
                        },
                        right: None,
                    }])]))
                })),],
            })
        );
    }

    #[test]
    fn test_for_command() {
        let source = "{- for $item, $index : .ref -}Inner Text{- end -}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![Control::Command(Command::For(ForCommand {
                    item_name: "item".to_string(),
                    index_name: Some("index".to_string()),
                    list: Expression(Or(vec![And(vec![Comparison {
                        left: AddSub {
                            left: MulDiv {
                                left: Unary::Primary(Primary::Field(Field {
                                    top_level: None,
                                    parts: vec![GetField::Name("ref".to_string())],
                                })),
                                rights: vec![],
                            },
                            rights: vec![],
                        },
                        right: None,
                    }])])),
                    inner: Template {
                        controls: vec![Control::Text("Inner Text".to_string())],
                    },
                }))],
            })
        );
    }

    #[test]
    fn test_if_command_with_else() {
        let source = "{- if #t -}It's true!{- else -}It's false!{- end -}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![Control::Command(Command::If(IfCommand {
                    condition: Expression(Or(vec![And(vec![Comparison {
                        left: AddSub {
                            left: MulDiv {
                                left: Unary::Primary(Primary::Boolean(true)),
                                rights: vec![],
                            },
                            rights: vec![],
                        },
                        right: None,
                    }])])),
                    inner: Template {
                        controls: vec![Control::Text("It's true!".to_string())],
                    },
                    else_inner: Some(Template {
                        controls: vec![Control::Text("It's false!".to_string())],
                    }),
                }))],
            })
        );
    }

    #[test]
    fn test_substitution() {
        let source = "Value: {:$my_var.my_field + 1:}, Expression: {:(1 + 2) * 3:}";
        let ast = parse_template(source);
        assert!(ast.is_ok());
        let ast = ast.unwrap();

        assert_eq!(
            ast,
            File(Template {
                controls: vec![
                    Control::Text("Value:".to_string()),
                    Control::Substitution(Substitution {
                        expr: Expression(Or(vec![And(vec![Comparison {
                            left: AddSub {
                                left: MulDiv {
                                    left: Unary::Primary(Primary::Field(Field {
                                        top_level: Some(TopLevel::Variable("my_var".to_string())),
                                        parts: vec![GetField::Name("my_field".to_string())],
                                    })),
                                    rights: vec![],
                                },
                                rights: vec![(
                                    AddSubOp::Add,
                                    MulDiv {
                                        left: Unary::Primary(Primary::Number(1)),
                                        rights: vec![],
                                    },
                                )],
                            },
                            right: None,
                        }])])),
                    }),
                    Control::Text(", Expression:".to_string()),
                    Control::Substitution(Substitution {
                        expr: Expression(Or(vec![And(vec![Comparison {
                            left: AddSub {
                                left: MulDiv {
                                    left: Unary::Primary(Primary::Expression(Expression(Or(
                                        vec![And(vec![Comparison {
                                            left: AddSub {
                                                left: MulDiv {
                                                    left: Unary::Primary(Primary::Number(1)),
                                                    rights: vec![],
                                                },
                                                rights: vec![(
                                                    AddSubOp::Add,
                                                    MulDiv {
                                                        left: Unary::Primary(Primary::Number(2)),
                                                        rights: vec![],
                                                    }
                                                )],
                                            },
                                            right: None,
                                        }])]
                                    )))),
                                    rights: vec![(
                                        MulDivOp::Multiply,
                                        Unary::Primary(Primary::Number(3)),
                                    )],
                                },
                                rights: vec![],
                            },
                            right: None,
                        }])])),
                    }),
                ],
            })
        );
    }
}
