use super::lexer::{Token, TokenValue}; // assuming Token is defined in your lexer module

#[derive(Debug, Clone)]
pub enum ASTNode {
    Literal {
        token: Token,
    },
    Binary {
        left: Box<ASTNode>,
        op: Token,
        right: Box<ASTNode>,
    },
    Variable {
        name: Token,
    },
    Grouping {
        expression: Box<ASTNode>,
    },
    Call {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    PropertyAccess {
        object: Box<ASTNode>,
        property: Token,
    },
    LetStatement {
        name: Token,
        initializer: Box<ASTNode>,
    },
    LetBangStatement {
        name: Token,
        initializer: Box<ASTNode>,
    },
    ExpressionStatement {
        expression: Box<ASTNode>,
    },
    IfExpression {
        condition: Box<ASTNode>,
        then_branch: Vec<ASTNode>,
        else_branch: Option<Vec<ASTNode>>,
    },
    FunctionStatement {
        name: Token,
        params: Vec<Token>,
        body: Vec<ASTNode>,
    },
    LambdaExpression {
        params: Vec<Token>,
        body: Vec<ASTNode>,
    },
    MatchStatement {
        value: Box<ASTNode>,
        arms: Vec<MatchArm>,
    },
    Pipeline {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ImportStatement {
        path: Token,
    },
    ListLiteral {
        elements: Vec<ASTNode>,
    },
    StructLiteral {
        keys: Vec<Token>,
        values: Vec<ASTNode>,
    },
    StructUpdate {
        base: Box<ASTNode>,
        keys: Vec<Token>,
        values: Vec<ASTNode>,
    },
    ArrayAppend {
        base: Box<ASTNode>,
        elements: Vec<ASTNode>,
    },
    BoolLiteral {
        value: bool,
    },
    EnumStatement {
        name: Token,
        variant_names: Vec<Token>,
        field_names: Vec<Vec<Token>>,
        field_counts: Vec<usize>,
    },
    EnumConstructor {
        enum_name: Token,
        variant_name: Token,
        field_names: Vec<Token>,
        values: Vec<ASTNode>,
    },
    EnumDeconstructPattern {
        enum_name: Token,        // e.g., "Shape"
        variant_name: Token,     // e.g., "Circle"
        field_names: Vec<Token>, // e.g., ["radius"] (could be empty for unit variants)
    },
    AsyncFunctionStatement {
        name: Token,
        params: Vec<Token>,
        body: Vec<ASTNode>,
    },
    AsyncLambdaExpression {
        params: Vec<Token>,
        body: Vec<ASTNode>,
    },
    AwaitExpression {
        expression: Box<ASTNode>,
    },
    StringInterpolation {
        parts: Vec<ASTNode>,
    },
    IndexAccess {
        object: Box<ASTNode>,
        index: Box<ASTNode>,
    },
    StructDeconstructPattern {
        field_names: Vec<Token>,
    },
    WildcardPattern,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub patterns: Vec<ASTNode>,
    pub expression: Vec<ASTNode>,
}

#[derive(Debug, Clone)]
pub struct ASTProgram {
    pub nodes: Vec<ASTNode>,
}

impl ASTProgram {
    pub fn print(&self) {
        for node in &self.nodes {
            node.print();
            println!();
        }
    }
}

impl ASTNode {
    pub fn print(&self) {
        use ASTNode::*;
        match self {
            AsyncLambdaExpression { params, body } => {
                print!("async fn(");
                for (i, param) in params.iter().enumerate() {
                    print!("{:?}", param.value);
                    if i < params.len() - 1 {
                        print!(", ");
                    }
                }
                print!(") -> {{ ");
                for (i, stmt) in body.iter().enumerate() {
                    stmt.print();
                    if i < body.len() - 1 {
                        print!("; ");
                    }
                }
                print!(" }}");
            }
            AsyncFunctionStatement { name, params, body } => {
                print!("async func {:?}(", name.value);
                for (i, param) in params.iter().enumerate() {
                    print!("{:?}", param.value);
                    if i < params.len() - 1 {
                        print!(", ");
                    }
                }
                print!(") {{ ");
                for (i, stmt) in body.iter().enumerate() {
                    stmt.print();
                    if i < body.len() - 1 {
                        print!("; ");
                    }
                }
                print!(" }}");
            }
            AwaitExpression { expression } => {
                print!("await ");
                expression.print();
            }
            Literal { token } => print!("{:?}", token.value), // customize as needed
            Binary { left, op, right } => {
                print!("(");
                left.print();
                print!(" {:?} ", op.value);
                right.print();
                print!(")");
            }
            Variable { name } => print!("{:?}", name.value),
            Grouping { expression } => {
                print!("(");
                expression.print();
                print!(")");
            }
            Call { callee, arguments } => {
                callee.print();
                print!("(");
                for (i, arg) in arguments.iter().enumerate() {
                    arg.print();
                    if i < arguments.len() - 1 {
                        print!(", ");
                    }
                }
                print!(")");
            }
            PropertyAccess { object, property } => {
                object.print();
                print!(".");
                print!("{:?}", property.value);
            }
            LetStatement { name, initializer } => {
                print!("let {:?} = ", name.value);
                initializer.print();
            }
            LetBangStatement { name, initializer } => {
                print!("let! {:?} = ", name.value);
                initializer.print();
            }
            ExpressionStatement { expression } => expression.print(),
            FunctionStatement { name, params, body } => {
                print!("func {:?}(", name);
                for (i, param) in params.iter().enumerate() {
                    print!("{:?}", param);
                    if i < params.len() - 1 {
                        print!(", ");
                    }
                }
                print!(") {{ ");
                for (i, stmt) in body.iter().enumerate() {
                    stmt.print();
                    if i < body.len() - 1 {
                        print!("; ");
                    }
                }
                print!(" }}");
            }
            LambdaExpression { params, body } => {
                print!("fn(");
                for (i, param) in params.iter().enumerate() {
                    print!("{:?}", param);
                    if i < params.len() - 1 {
                        print!(", ");
                    }
                }
                print!(") -> {{ ");
                for (i, stmt) in body.iter().enumerate() {
                    stmt.print();
                    if i < body.len() - 1 {
                        print!("; ");
                    }
                }
                print!(" }}");
            }
            MatchStatement { value, arms } => {
                print!("match ");
                value.print();
                println!(" {{");
                for arm in arms {
                    print!("  ");
                    arm.patterns.clone().into_iter().for_each(|p| p.print());
                    print!(" -> ");
                    for expression in &arm.expression {
                        expression.print();
                    }
                    println!(",");
                }
                print!("}}");
            }
            Pipeline { left, right } => {
                print!("(");
                left.print();
                print!(" |> ");
                right.print();
                print!(")");
            }
            ImportStatement { path } => {
                print!("import {:?}", path);
            }
            IfExpression {
                condition,
                then_branch,
                else_branch,
            } => {
                print!("if ");
                condition.print();
                print!(" {{ ");
                for (i, stmt) in then_branch.iter().enumerate() {
                    stmt.print();
                    if i < then_branch.len() - 1 {
                        print!("; ");
                    }
                }
                print!(" }}");
                if let Some(else_branch) = else_branch {
                    print!(" else {{ ");
                    for (i, stmt) in else_branch.iter().enumerate() {
                        stmt.print();
                        if i < else_branch.len() - 1 {
                            print!("; ");
                        }
                    }
                    print!(" }}");
                }
            }
            ListLiteral { elements } => {
                print!("[");
                for (i, el) in elements.iter().enumerate() {
                    el.print();
                    if i < elements.len() - 1 {
                        print!(", ");
                    }
                }
                print!("]");
            }
            StructLiteral { keys, values } => {
                print!("{{ ");
                for (i, (k, v)) in keys.iter().zip(values).enumerate() {
                    print!("{:?} = ", k);
                    v.print();
                    if i < keys.len() - 1 {
                        print!(", ");
                    }
                }
                print!(" }}");
            }
            StructUpdate { base, keys, values } => {
                base.print();
                print!(" <- {{ ");
                for (i, (k, v)) in keys.iter().zip(values).enumerate() {
                    print!("{:?} = ", k);
                    v.print();
                    if i < keys.len() - 1 {
                        print!(", ");
                    }
                }
                print!(" }}");
            }
            ArrayAppend { base, elements } => {
                base.print();
                print!(" <- [");
                for (i, element) in elements.iter().enumerate() {
                    element.print();
                    if i < elements.len() - 1 {
                        print!(", ");
                    }
                }
                print!("]");
            }
            EnumStatement {
                name,
                variant_names,
                field_names,
                field_counts: _,
            } => {
                print!("enum {:?} {{\n", name);
                for (i, (variant, fields)) in variant_names.iter().zip(field_names).enumerate() {
                    print!("  {:?}", variant);
                    if !fields.is_empty() {
                        print!(" {{ ");
                        for (j, field) in fields.iter().enumerate() {
                            print!("{:?}", field);
                            if j < fields.len() - 1 {
                                print!(", ");
                            }
                        }
                        print!(" }}");
                    }
                    if i < variant_names.len() - 1 {
                        print!(",");
                    }
                    println!();
                }
                print!("}}");
            }
            EnumConstructor {
                enum_name,
                variant_name,
                field_names,
                values,
            } => {
                print!("{:?}::{:?}(", enum_name, variant_name);
                for (i, (name, value)) in field_names.iter().zip(values).enumerate() {
                    print!("{:?} = ", name);
                    value.print();
                    if i < field_names.len() - 1 {
                        print!(", ");
                    }
                }
                print!(")");
            }
            BoolLiteral { value } => print!("{}", value),
            EnumDeconstructPattern {
                field_names,
                enum_name: _,
                variant_name: _,
            } => {
                print!("(");
                for (i, b) in field_names.iter().enumerate() {
                    print!("{:?}", b);
                    if i < field_names.len() - 1 {
                        print!(", ");
                    }
                }
                print!(")");
            }
            StringInterpolation { parts } => {
                print!("\"");
                for part in parts {
                    match part {
                        ASTNode::Literal { token } => {
                            if let TokenValue::String(s) = &token.value {
                                print!("{}", s);
                            }
                        }
                        _ => {
                            print!("${{");
                            part.print();
                            print!("}}");
                        }
                    }
                }
                print!("\"");
            }
            IndexAccess { object, index } => {
                object.print();
                print!("[");
                index.print();
                print!("]");
            }
            StructDeconstructPattern { field_names } => {
                print!("{{ ");
                for (i, field) in field_names.iter().enumerate() {
                    print!("{:?}", field.value);
                    if i < field_names.len() - 1 {
                        print!(", ");
                    }
                }
                print!(" }}");
            }
            WildcardPattern => print!("_"),
        }
    }
}
