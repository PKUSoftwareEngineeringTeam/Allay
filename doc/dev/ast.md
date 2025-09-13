This file contains the AST for the Allay template language.

```
Template            ::= {Control};
Control             ::= Text | ShortCode | Command | Substitution;

Text                ::= /[^{}-]+/;

Identifier          ::= /[a-zA-Z_][a-zA-Z0-9_]*/;
Variable            ::= '$' Identifier;
GetField            ::= '.' Identifier;
this                ::= 'this';
TopLevel            ::= Variable | this;
Number              ::= /[0-9]+/;
String              ::= /"([^"\\]|\\.)*"/;
AddOp               ::= '+' | '-';
MulOp               ::= '*' | '/' | '%';
ComparisonOp        ::= '==' | '!=' | '<' | '<=' | '>' | '>=' ;
AndOp               ::= '&&';
OrOp                ::= '||';
NotOp               ::= '!';

Expression          ::= LogicOr;
LogicOr             ::= LogicAnd { OrOp LogicAnd };
LogicAnd            ::= Comparison { AndOp Comparison };
Comparison          ::= Addition [ ComparisonOp Addition ];
Addition            ::= Multiplication { AddOp Multiplication };
Multiplication      ::= Unary { MulOp Unary };
Unary               ::= [NotOp | AddOp] Primary;
Field               ::= TopLevel? GetField {GetField};
BoolLiteral         ::= 'true' | 'false';
Primary             ::= Field | TopLevel | Number | String | BoolLiteral | '(' Expression ')';

ShortCode           ::= SingleShortCode | BlockShortCode;
SingleShortCode     ::= '{<' Identifier {Expression} '/>}';
BlockShortCode      ::= '{<' Identifier {Expression} '>}' Template '{</' Identifier '>}';

Command             ::= SetCommand | ForCommand | WithCommand | IfCommand | IncludeCommand;

StartForCommand     ::= '{-' 'for' UserVariable [',' UserVariable] ':' Expression '-}';
StartWithCommand    ::= '{-' 'with' Expression '-}';
StartIfCommand      ::= '{-' 'if' Expression '-}';
ElseCommand         ::= '{-' 'else' '-}';
EndCommand          ::= '{-' 'end' '-}';

SetCommand          ::= '{-' 'set' UserVariable Expression '-}';
ForCommand          ::= StartForCommand Template EndCommand;
WithCommand         ::= StartWithCommand Template EndCommand;
IfCommand           ::= StartIfCommand Template {ElseCommand Template} EndCommand;
IncludeCommand      ::= '{-' 'include' String {Expression} '-}';

Substitution        ::= GetSubstitution | ExprSubstitution | ParamSubstitution;
GetSubstitution     ::= '{:' 'get' Expression ':}';
ExprSubstitution    ::= '{:' Expression ':}';
ParamSubstitution   ::= '{:' 'param' Number ':}';
```