use std::fmt::{format, Display};

use im::hashmap::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Closure { param: Box<Type>, body: Box<Type> },
    Forall { param: String, body: Box<Type> },
    Var(String),
    Int,
    Str,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Type::*;

        let type_ = match self {
            Closure { param, body } => format!("{} -> {}", param, body),
            Forall { param, body } => format!("{} -> {}", param, body),
            Var(var) => format!("Var({})", var),
            Int => String::from("Int"),
            Str => String::from("Str"),
        };

        f.write_str(&type_)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Int(i64),
    Var(String),
    Abs {
        param: String,
        param_type: Type,
        body: Box<Expr>,
    },
    TypeAbs {
        param: String,
        body: Box<Expr>,
    },
    TypeApp {
        arg: Type,
        abs: Box<Expr>,
    },
    App {
        arg: Box<Expr>,
        abs: Box<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expr::*;

        let expr = match self {
            Int(int) => int.to_string(),
            Var(name) => name.clone(),
            Abs {
                param,
                param_type,
                body,
            } => format!("(λ{}:{}.{})", param, param_type, body),
            TypeAbs { param, body } => format!("(Λ{}.{})", param, body),
            TypeApp { arg, abs } => format!("({} {})", abs, arg),
            App { arg, abs } => format!("({} {})", abs, arg),
        };

        write!(f, "{}", expr)
    }
}

type TypeContext = HashMap<String, Type>;

fn replace_type(type_: &Type, from: String, to: Type) -> Type {
    use Type::*;

    match type_ {
        Closure { param, body } => {
            let param = replace_type(param, from.clone(), to.clone());
            let body = replace_type(body, from, to);

            Type::Closure {
                param: Box::new(param),
                body: Box::new(body),
            }
        }
        Forall { param, body } => match param == &from {
            true => Forall {
                param: param.clone(),
                body: body.clone(),
            },
            false => Forall {
                param: param.clone(),
                body: Box::new(replace_type(&*body, from, to)),
            },
        },
        Var(var) => match var == &from {
            true => to,
            false => type_.clone(),
        },
        Int => Int,
        Str => Str,
    }
}

fn infer(expr: Expr, context: TypeContext) -> Type {
    match expr {
        Expr::Var(var) => context
            .get(&var)
            .expect(&format! {"type error: unbound variable {var}"})
            .clone(),
        Expr::Abs {
            param,
            param_type,
            body,
        } => {
            let context = context.update(param, param_type.clone());

            let body = infer(*body, context);
            Type::Closure {
                param: Box::new(param_type),
                body: Box::new(body),
            }
        }
        Expr::App { arg, abs } => {
            let arg = infer(*arg, context.clone());

            match infer(*abs, context) {
                Type::Closure { param, body } => match *param == arg {
                    true => *body,
                    false => panic!("expecting type {}. {} given", (*param), arg),
                },
                typ => panic!("type {} cannot be used as a closure", typ),
            }
        }
        Expr::TypeAbs { param, body } => {
            let body = infer(*body, context);

            Type::Forall {
                param,
                body: Box::new(body),
            }
        }
        Expr::TypeApp { arg, abs } => match infer(*abs.clone(), context) {
            Type::Forall { param, body } => replace_type(&*body, param, arg),
            type_ => panic!("cannot apply type {} to {}", abs, type_),
        },
        Expr::Int(_int) => Type::Int,
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Closure {
        param: String,
        body: Expr,
        context: ValueContext,
    },
    Forall {
        body: Expr,
        context: ValueContext,
    },
    Int(i64),
    Native(fn(Box<Value>) -> Box<Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Value::*;

        let value = match self {
            Closure {
                param,
                body,
                context: _,
            } => format!("(Closure {} -> {} )", param, body),
            Forall { body, context: _ } => format!("(Forall {})", body),
            Int(int) => int.to_string(),
            Native(_) => "(Native)".to_string(),
        };

        f.write_str(&value)
    }
}

type ValueContext = HashMap<String, Value>;

fn eval(expr: Expr, context: ValueContext) -> Value {
    match expr {
        Expr::Var(var) => context
            .get(&var)
            .expect(&format!("unbound variable: {}", var))
            .clone(),
        Expr::Abs {
            param,
            param_type: _,
            body,
        } => Value::Closure {
            param,
            body: *body,
            context,
        },
        Expr::App { arg, abs } => {
            let arg = eval(*arg, context.clone());

            match eval(*abs, context) {
                Value::Closure {
                    param,
                    body,
                    context,
                } => {
                    let context = context.update(param, arg);

                    eval(body, context)
                }
                Value::Native(native) => *(native(Box::new(arg))),
                Value::Int(_value) => panic!(),
                Value::Forall {
                    body: _,
                    context: _,
                } => panic!(),
            }
        }
        // the forall value is evaluated later. this is just no-op
        Expr::TypeAbs { param: _, body } => Value::Forall {
            body: *body,
            context,
        },
        Expr::TypeApp { arg: _, abs } => match eval(*abs, context) {
            Value::Forall { body, context } => eval(body, context),
            _ => panic!("invalid type application"),
        },
        Expr::Int(int) => Value::Int(int),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_infers_identity_function() {
        let ast = Expr::Abs {
            param: String::from("x"),
            param_type: Type::Int,
            body: Box::new(Expr::Var(String::from("x"))),
        };

        let infered_type = infer(ast.clone(), TypeContext::new());
        let expected_type = Type::Closure {
            param: Box::new(Type::Int),
            body: Box::new(Type::Int),
        };

        assert_eq!(infered_type, expected_type);

        let ast = Expr::App {
            arg: Box::new(Expr::Int(4)),
            abs: Box::new(ast),
        };
        let infered_type = infer(ast, TypeContext::new());
        let expected_type = Type::Int;

        assert_eq!(infered_type, expected_type);
    }

    #[test]
    fn it_evals_identity_function() {
        let ast = Expr::Abs {
            param: String::from("x"),
            param_type: Type::Str,
            body: Box::new(Expr::Var(String::from("x"))),
        };

        let evaluated_value = eval(ast.clone(), ValueContext::new());
        let expected_value = Value::Closure {
            param: String::from("x"),
            body: Expr::Var(String::from("x")),
            context: ValueContext::new(),
        };

        assert_eq!(evaluated_value, expected_value);

        let ast = Expr::App {
            arg: Box::new(Expr::Int(4)),
            abs: Box::new(ast),
        };

        let evaluated_value = eval(ast, ValueContext::new());
        let expected_value = Value::Int(4);

        assert_eq!(evaluated_value, expected_value);
    }

    #[test]
    fn it_infers_polymorphic_identity_function() {
        let ast = Expr::TypeAbs {
            param: String::from("a"),
            body: Box::new(Expr::Abs {
                param: String::from("x"),
                param_type: Type::Var(String::from("a")),
                body: Box::new(Expr::Var(String::from("x"))),
            }),
        };

        let infered_type = infer(ast.clone(), TypeContext::new());
        let expected_type = Type::Forall {
            param: String::from("a"),
            body: Box::new(Type::Closure {
                param: Box::new(Type::Var(String::from("a"))),
                body: Box::new(Type::Var(String::from("a"))),
            }),
        };

        assert_eq!(infered_type, expected_type);

        let ast = Expr::TypeApp {
            arg: Type::Int,
            abs: Box::new(ast),
        };

        let infered_type = infer(ast, TypeContext::new());
        let expected_type = Type::Closure {
            param: Box::new(Type::Int),
            body: Box::new(Type::Int),
        };

        assert_eq!(infered_type, expected_type);
    }

    #[test]
    fn it_evaluates_polymorphic_identity_function() {
        let ast = Expr::TypeAbs {
            param: String::from("a"),
            body: Box::new(Expr::Abs {
                param: String::from("x"),
                param_type: Type::Var(String::from("a")),
                body: Box::new(Expr::Var(String::from("x"))),
            }),
        };

        let evaluated_value = eval(ast.clone(), ValueContext::new());
        let expected_value = Value::Forall {
            body: Expr::Abs {
                param: String::from("x"),
                param_type: Type::Var(String::from("a")),
                body: Box::new(Expr::Var(String::from("x"))),
            },
            context: ValueContext::new(),
        };

        assert_eq!(evaluated_value, expected_value);

        let ast = Expr::TypeApp {
            arg: Type::Int,
            abs: Box::new(ast),
        };

        let evaluated_value = eval(ast, ValueContext::new());
        let expected_value = Value::Closure {
            param: String::from("x"),
            body: Expr::Var(String::from("x")),
            context: ValueContext::new(),
        };

        assert_eq!(evaluated_value, expected_value);
    }
}

fn main() {
    println!("run `cargo test` to see if it works");
}
