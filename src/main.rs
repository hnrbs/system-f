use im::hashmap::HashMap;

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    String(String),
    Var(String),
    Abs { param: String, body: Box<Expr> },
    App { arg: Box<Expr>, abs: Box<Expr> },
}

#[derive(Clone, Debug)]
enum Value {
    VClosure {
        param: String,
        body: Expr,
        context: Context,
    },
    VInt(i64),
    VString(String),
    VNative(fn(Box<Value>) -> Box<Value>),
}

type Context = HashMap<String, Value>;

fn eval(expr: Expr, context: Context) -> Value {
    match expr {
        Expr::Var(var) => context
            .get(&var)
            .expect(&format!("unbound variable: {}", var))
            .clone(),
        Expr::Abs { param, body } => Value::VClosure {
            param,
            body: *body,
            context,
        },
        Expr::App { arg, abs } => {
            let arg = eval(*arg, context.clone());

            match eval(*abs, context) {
                Value::VClosure {
                    param,
                    body,
                    context,
                } => {
                    let context = context.update(param, arg);

                    eval(body, context)
                }
                Value::VNative(native) => *(native(Box::new(arg))),
                Value::VInt(_value) => panic!(),
                Value::VString(_value) => panic!(),
            }
        }
        Expr::Int(int) => Value::VInt(int),
        Expr::String(string) => Value::VString(string),
    }
}

fn main() {
    let print = Value::VNative(|value: Box<Value>| {
        match *value.clone() {
            Value::VInt(value) => println!("{}", value),
            Value::VString(value) => println!("{}", value),
            Value::VNative(_value) => panic!("natives cannot be printed"),
            Value::VClosure {
                param: _,
                body: _,
                context: _,
            } => panic!("closures cannot be printed"),
        };

        value
    });

    let context = Context::new().update(String::from("print"), print);

    let ast = Expr::App {
        abs: Box::new(Expr::Var(String::from("print"))),
        arg: Box::new(Expr::String("Hello, World!".to_owned())),
    };

    let _result = eval(ast, context);
}
