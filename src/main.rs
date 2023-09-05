use std::fmt::{format, Display};

use im::hashmap::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Closure { param: Box<Type>, body: Box<Type> },
    Forall { param: String, body: Box<Type> },
    Var(String),
    Int,
}

impl Type {
    fn name(&self) -> String {
        match self {
            Type::Closure { param, body } => {
                let param = (*param).name();
                let body = (*body).name();

                format!("closure<{} -> {}>", param, body)
            }
            Type::Forall { param, body } => {
                let body = (*body).name();

                format!("closure<{} -> {}>", param, body)
            }
            Type::Var(string) => string.clone(),
            Type::Int => String::from("int"),
        }
    }
}

#[derive(Debug, Clone)]
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
            } => format!("({}:{}.{})", param, param_type.name(), body),
            TypeAbs { param, body } => format!("(Λ{}.{})", param, body),
            TypeApp { arg, abs } => format!("(∀{}.{}", arg.name(), abs),
            App { arg, abs } => format!("{} {}", arg, abs),
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
                    false => panic!("expecting type {}. {} given", (*param).name(), arg.name()),
                },
                typ => panic!("type {} cannot be used as a closure", typ.name()),
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
            type_ => panic!("cannot apply type {} to {}", abs, type_.name()),
        },
        Expr::Int(_int) => Type::Int,
    }
}

#[derive(Clone, Debug)]
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
    String(String),
    Native(fn(Box<Value>) -> Box<Value>),
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
                Value::String(_value) => panic!(),
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

fn main() {}
