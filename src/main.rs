use std::fmt::Display;

use im::hashmap::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Type {
    Closure { param: Box<Type>, body: Box<Type> },
    Int,
    String,
}

impl Type {
    fn name(&self) -> String {
        match self {
            Type::Closure { param, body } => {
                let param = (*param).name();
                let body = (*body).name();

                format!("closure<{} -> {}>", param, body)
            }
            Type::Int => String::from("int"),
            Type::String => String::from("string"),
        }
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    String(String),
    Var(String),
    Abs {
        param: String,
        param_type: Type,
        body: Box<Expr>,
    },
    App {
        arg: Box<Expr>,
        abs: Box<Expr>,
    },
}

type TypeContext = HashMap<String, Type>;

fn infer(expr: Expr, context: TypeContext) -> Type {
    match expr {
        Expr::Int(_int) => Type::Int,
        Expr::String(_str) => Type::String,
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
    }
}

#[derive(Clone, Debug)]
enum Value {
    Closure {
        param: String,
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
            }
        }
        Expr::Int(int) => Value::Int(int),
        Expr::String(string) => Value::String(string),
    }
}

fn main() {
    let type_context = TypeContext::new();

    let t_print = Type::Closure {
        param: Box::new(Type::Int),
        body: Box::new(Type::Int),
    };

    let type_context = type_context.update(String::from("print"), t_print);

    let v_print = Value::Native(|value: Box<Value>| {
        match *value.clone() {
            Value::Int(value) => println!("{}", value),
            _ => panic!(),
        };

        value
    });

    let value_context = ValueContext::new().update(String::from("print"), v_print);

    let ast = Expr::App {
        abs: Box::new(Expr::Var(String::from("print"))),
        arg: Box::new(Expr::String("Hello, World!".to_owned())),
    };

    let _result_type = infer(ast.clone(), type_context);

    let _result_value = eval(ast, value_context);
}
