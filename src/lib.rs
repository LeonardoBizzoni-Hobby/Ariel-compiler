use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ast_generator::ast::{
    datatypes::DataType, enums::Enum, expressions, function::Function,
    scopebound_statements::ScopeBoundStatement, structs::Struct, ASTs,
};
use ast_walker::{env::Environment, type_error::TypeError, value::Value};

use crate::ast_generator::parser;

mod ast_generator;
mod ast_walker;
mod test_util;
mod tokens;

#[macro_export]
macro_rules! measure {
    ($task:expr) => {{
        let start_timer = std::time::Instant::now();
        let res = $task;
        let elapsed = start_timer.elapsed();

        println!(
            "Task took: {}ns ({}ms)",
            elapsed.as_nanos(),
            elapsed.as_millis()
        );

        res
    }};
}

pub fn compile(source: &str) {
    let imported_files: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let ast: ASTs = measure!(parser::parse(source, imported_files));

    let mut global_env: Environment = HashMap::new();
    let env_ptr: *mut Environment = &mut global_env;

    macro_rules! check_and_insert {
        ($ast2check:expr, $value2insert:expr) => {
            unsafe {
                if (*env_ptr).contains_key($ast2check.name.lexeme.as_str()) {
                    eprintln!("`{}` is defined more then once.", $ast2check.name.lexeme);
                    return;
                }

                (*env_ptr).insert(&$ast2check.name.lexeme, $value2insert);
            }
        };
    }

    let _ = measure!({
        ast.fns.iter().for_each(|func| {
            check_and_insert!(
                func,
                Value::Function {
                    arity: func.args.len(),
                    returns: &func.ret_type,
                    closure_env: &global_env,
                    ast: func,
                }
            )
        });

        ast.enums.iter().for_each(|enumeration| {
            check_and_insert!(enumeration, Value::Enum { ast: enumeration })
        });

        ast.structs
            .iter()
            .for_each(|class| check_and_insert!(class, Value::Struct { ast: class }));
    });

    println!("{global_env:#?}");

    if !valid_structs(&global_env, &ast.structs)
        | !valid_enums(&global_env, &ast.enums)
        | !valid_fn(&global_env, &ast.fns)
    {
        return;
    }

    println!("All good!");
}

fn valid_enums(env: &Environment, enums: &[Enum]) -> bool {
    let mut res = true;

    for myenum in enums.iter() {
        for (variant_name, datatype) in myenum.variants.iter() {
            if let Some(dt) = datatype {
                if !valid_datatype(env, dt) {
                    res = false;
                    eprintln!(
                        "[{} {}:{}] Enum `{}` defines field `{}` of type `{}` which doesn't exists.",
                        myenum.name.found_in, variant_name.line, variant_name.column, myenum.name.lexeme, variant_name.lexeme, dt
                    );
                }
            }
        }
    }

    res
}

fn valid_structs(env: &Environment, structs: &[Struct]) -> bool {
    let mut res = true;

    for mystruct in structs.iter() {
        for (fieldname, datatype) in mystruct.fields.iter() {
            if !valid_datatype(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Struct `{}` defines field `{}` of type `{datatype}` which doesn't exists.",
                    mystruct.name.found_in, fieldname.line, fieldname.column, mystruct.name.lexeme, fieldname.lexeme
                );
            }
        }
    }

    res
}

fn valid_fn(env: &Environment, fns: &[Function]) -> bool {
    let mut res = true;

    for myfn in fns.iter() {
        for (arg, datatype) in myfn.args.iter() {
            if !valid_datatype(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Function `{}` expects argument `{}` of type `{datatype}` which doesn't exists.",
                    myfn.name.found_in, arg.line, arg.column, myfn.name.lexeme, arg.lexeme
                );
            }
        }

        if let Some(datatype) = &myfn.ret_type {
            if !valid_datatype(env, datatype) {
                res = false;
                eprintln!(
                    "[{} {}:{}] Function `{}` returns `{datatype}` but this type isn't defined.",
                    myfn.name.found_in, myfn.name.line, myfn.name.column, myfn.name.lexeme
                );
            }
        }

        if let Some(body) = &myfn.body {
            if let Err(evec) = validate_local_scope(body, &myfn.ret_type) {
                for e in evec {
                    res = false;
                    match e {
                        TypeError::InvalidReturnValue {
                            line,
                            column,
                            expected,
                            got,
                        } => {
                            eprintln!(
                        "[{} {}:{}] Invalid return type in function {}: expected `{expected}` but instead received `{got}`.",
                        myfn.name.found_in, line, column, myfn.name.lexeme
                    );
                        }
                        TypeError::UnexpectedType {
                            line,
                            column,
                            expected,
                            got,
                        } => {
                            eprintln!(
                        "[{} {}:{}] Unexpected type in function {}: expected `{expected}` but instead received `{got}`.",
                        myfn.name.found_in, line, column, myfn.name.lexeme
                    );
                        }
                        TypeError::InvalidStmt { line, column } => {
                            eprintln!(
                                "[{} {}:{}] Impossible statement in function {}.",
                                myfn.name.found_in, line, column, myfn.name.lexeme
                            );
                        }
                        TypeError::InvalidTypeConversion {
                            line,
                            column,
                            from,
                            to,
                        } => {
                            eprintln!(
                            "[{} {}:{}] Invalid type conversion in function {}: from value of type `{from}` to value of type `{to}`.",
                            myfn.name.found_in, line, column, myfn.name.lexeme
                        );
                        }
                        TypeError::InvalidArrayLiteral {
                            line,
                            column,
                            expected,
                            found,
                        } => {
                            eprintln!(
                            "[{} {}:{}] Invalid array literal in function {}: an array can contain only expression that result to the same type but there was a mix of `{expected}` and `{found}` types.",
                            myfn.name.found_in, line, column, myfn.name.lexeme
                        );
                        }
                    }
                }
            }
        }
    }

    res
}

fn validate_local_scope(
    stmts: &[ScopeBoundStatement],
    return_type: &Option<DataType>,
) -> Result<(), Vec<TypeError>> {
    let mut errvec = vec![];
    for stmt in stmts.iter() {
        match stmt {
            ScopeBoundStatement::Scope { body, .. } => {
                if let Err(mut sub_errvec) = validate_local_scope(&body, return_type) {
                    errvec.append(&mut sub_errvec);
                }
            }
            ScopeBoundStatement::VariableDeclaration { .. } => todo!(),
            ScopeBoundStatement::Return {
                value: maybe_value,
                line,
                column,
            } => match maybe_value {
                Some(value) => {
                    let maybe_found_type: Result<DataType, TypeError> = evaluate(value);
                    match maybe_found_type {
                        Ok(found_type) => match return_type {
                            Some(expected_type) => match (found_type.clone(), expected_type) {
                                (DataType::Bool, DataType::Bool) => {}
                                (
                                    DataType::Array(_) |
                                    DataType::Pointer(_),
                                    DataType::U8
                                    | DataType::U16
                                    | DataType::U32
                                    | DataType::U64
                                    | DataType::Usize
                                    | DataType::I8
                                    | DataType::I16
                                    | DataType::I32
                                    | DataType::I64
                                    | DataType::Isize
                                    | DataType::F32
                                    | DataType::F64
                                    | DataType::String
                                    | DataType::Bool
                                    | DataType::Compound { .. },
                                )
                                | (
                                    DataType::U8
                                    | DataType::U16
                                    | DataType::U32
                                    | DataType::U64
                                    | DataType::Usize
                                    | DataType::I8
                                    | DataType::I16
                                    | DataType::I32
                                    | DataType::I64
                                    | DataType::Isize
                                    | DataType::F32
                                    | DataType::F64,
                                    DataType::Array(_) | DataType::Pointer(_),
                                )
                                | (
                                    DataType::U8
                                    | DataType::U16
                                    | DataType::U32
                                    | DataType::U64
                                    | DataType::Usize
                                    | DataType::I8
                                    | DataType::I16
                                    | DataType::I32
                                    | DataType::I64
                                    | DataType::Isize
                                    | DataType::F32
                                    | DataType::F64,
                                    DataType::String
                                    | DataType::Bool
                                    | DataType::Void
                                    | DataType::Compound { .. },
                                )
                                | (DataType::Bool, _) => {
                                    errvec.push(TypeError::InvalidTypeConversion {
                                        line: stmt.line(),
                                        column: stmt.column(),
                                        from: found_type,
                                        to: expected_type.clone(),
                                    })
                                }
                                _ => {
                                    println!("\t\t{} - {} OK!", found_type.clone(), expected_type);
                                }
                            },
                            None => errvec.push(TypeError::InvalidReturnValue {
                                line: *line,
                                column: *column,
                                expected: DataType::Void,
                                got: found_type,
                            }),
                        },
                        Err(e) => errvec.push(e),
                    }
                }
                None => {
                    if let Some(return_type) = return_type {
                        if *return_type != DataType::Void {
                            errvec.push(TypeError::InvalidReturnValue {
                                line: *line,
                                column: *column,
                                expected: return_type.clone(),
                                got: DataType::Void,
                            });
                        }
                    }
                }
            },
            ScopeBoundStatement::ImplicitReturn { .. } => todo!(),
            ScopeBoundStatement::Expression { .. } => todo!(),
            ScopeBoundStatement::Defer { .. } => todo!(),
            ScopeBoundStatement::Conditional { .. } => todo!(),
            ScopeBoundStatement::Match { .. } => todo!(),
            ScopeBoundStatement::Loop {
                line: _,
                column: _,
                body: _,
            } => {
                // if let Some(body) = body {
                //     if !validate_loop_scope(&body) {
                //         return false;
                //     }
                // }
            }
            ScopeBoundStatement::While {
                condition,
                body: _,
                line,
                column,
            } => {
                let maybe_condition_type = evaluate_expr(condition);
                match maybe_condition_type {
                    Ok(condition_type) => {
                        if condition_type != DataType::Bool {
                            errvec.push(TypeError::UnexpectedType {
                                line: *line,
                                column: *column,
                                expected: DataType::Bool,
                                got: condition_type,
                            });
                        }
                    }
                    Err(e) => errvec.push(e),
                }
            }
            ScopeBoundStatement::For { .. } => todo!(),
            ScopeBoundStatement::Break { line, column }
            | ScopeBoundStatement::Continue { line, column } => {
                errvec.push(TypeError::InvalidStmt {
                    line: *line,
                    column: *column,
                });
            }
        }
    }

    if errvec.is_empty() {
        Ok(())
    } else {
        Err(errvec)
    }
}

fn valid_datatype(env: &Environment, datatype: &DataType) -> bool {
    match datatype {
        DataType::Compound { name } => env.contains_key(name.lexeme.as_str()),
        DataType::Pointer(of) | DataType::Array(of) => valid_datatype(env, of),
        _ => true,
    }
}

fn evaluate(value: &ScopeBoundStatement) -> Result<DataType, TypeError> {
    let mut returns = DataType::Void;
    match value {
        ScopeBoundStatement::Scope { body, .. } => {
            for stmt in body.iter() {
                match stmt {
                    ScopeBoundStatement::Scope { .. } => todo!(),
                    ScopeBoundStatement::Return { value, .. } => match value {
                        Some(value) => returns = evaluate(&value)?,
                        None => returns = DataType::Void,
                    },
                    ScopeBoundStatement::ImplicitReturn { expr, .. }
                    | ScopeBoundStatement::Expression { expr, .. } => {
                        returns = evaluate_expr(&expr)?
                    }
                    ScopeBoundStatement::Defer { .. } => todo!(),
                    ScopeBoundStatement::Conditional { .. } => todo!(),
                    ScopeBoundStatement::Match { .. } => todo!(),
                    ScopeBoundStatement::Loop { .. } => todo!(),
                    ScopeBoundStatement::While { .. } => todo!(),
                    ScopeBoundStatement::For { .. } => todo!(),
                    _ => {}
                }
            }
        }
        ScopeBoundStatement::VariableDeclaration { .. } => todo!(),
        ScopeBoundStatement::Return { .. } => todo!(),
        ScopeBoundStatement::ImplicitReturn { .. } => todo!(),
        ScopeBoundStatement::Expression { expr, .. } => returns = evaluate_expr(expr)?,
        ScopeBoundStatement::Defer { .. } => todo!(),
        ScopeBoundStatement::Conditional { .. } => todo!(),
        ScopeBoundStatement::Match { .. } => todo!(),
        ScopeBoundStatement::Loop { .. } => todo!(),
        ScopeBoundStatement::While { .. } => todo!(),
        ScopeBoundStatement::For { .. } => todo!(),
        ScopeBoundStatement::Break { .. } => todo!(),
        ScopeBoundStatement::Continue { .. } => todo!(),
    }

    Ok(returns)
}

fn evaluate_expr(expr: &expressions::Expression) -> Result<DataType, TypeError> {
    match expr {
        expressions::Expression::Literal { literal, .. } => match literal.ttype {
            tokens::token_type::TokenType::Double => Ok(DataType::F32),
            tokens::token_type::TokenType::Integer => Ok(DataType::I32),
            tokens::token_type::TokenType::String => Ok(DataType::String),
            tokens::token_type::TokenType::True | tokens::token_type::TokenType::False => {
                Ok(DataType::Bool)
            }
            tokens::token_type::TokenType::Nil => Ok(DataType::Pointer(Box::new(DataType::Void))),
            _ => panic!("somehow a non-literal value is interpreted as a literal"),
        },
        expressions::Expression::ArrayLiteral { values, .. } => {
            if values.is_empty() {
                Ok(DataType::Void)
            } else {
                let res: DataType = evaluate_expr(&values[0])?;

                for x in 1..values.len() {
                    let curr_dt = evaluate_expr(&values[x])?;
                    if curr_dt != res {
                        return Err(TypeError::InvalidArrayLiteral {
                            line: values[x].line(),
                            column: values[x].column(),
                            expected: res,
                            found: curr_dt,
                        });
                    }
                }

                Ok(DataType::Array(Box::new(res)))
            }
        }
        _ => todo!(),
    }
}
