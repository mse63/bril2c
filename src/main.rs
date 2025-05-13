use bril_rs::{
    Argument, Code, EffectOps, Function, Instruction, Program, Type, ValueOps, load_program,
};
use serde_json::ser::Formatter;

#[derive(PartialEq, Eq, Debug)]
struct StaticVar {
    var_type: Type,
    name: String,
}

fn find_static_vars(function: &Function) -> Vec<StaticVar> {
    let mut input_vars = Vec::new();
    for arg in function.args.iter() {
        input_vars.push(arg.name.to_string());
    }
    let mut static_vars = Vec::new();
    for code in function.instrs.iter() {
        if let Code::Instruction(instr) = code {
            match instr {
                Instruction::Constant {
                    dest,
                    op,
                    const_type,
                    value,
                } => {
                    let static_var = StaticVar {
                        var_type: const_type.clone(),
                        name: dest.to_string(),
                    };
                    if !static_vars.contains(&static_var) && !input_vars.contains(&static_var.name)
                    {
                        static_vars.push(static_var);
                    }
                }
                Instruction::Value {
                    args,
                    dest,
                    funcs,
                    labels,
                    op,
                    op_type,
                } => {
                    let static_var = StaticVar {
                        var_type: op_type.clone(),
                        name: dest.to_string(),
                    };
                    if !static_vars.contains(&static_var) && !input_vars.contains(&static_var.name)
                    {
                        static_vars.push(static_var);
                    }
                }
                Instruction::Effect {
                    args,
                    funcs,
                    labels,
                    op,
                } => {}
            }
        }
    }
    static_vars
}

trait Crep {
    fn crep(self) -> String;
}

fn func_rep(func: String) -> String {
    format!("{func}_f")
}

impl Crep for String {
    fn crep(self) -> String {
        let ans = self.replace(".", "_");
        format!("{ans}_")
    }
}

impl Crep for StaticVar {
    fn crep(self) -> String {
        let type_crep = self.var_type.crep();
        let var_name = self.name.crep();
        format!("{type_crep} {var_name};")
    }
}

impl Crep for Type {
    fn crep(self) -> String {
        match self {
            Type::Int => "int64_t".to_string(),
            Type::Bool => "uint8_t".to_string(),
            Type::Float => "double".to_string(),
            Type::Pointer(pointed) => {
                let pointed_crep = pointed.crep();
                format!("{pointed_crep}*")
            }
        }
    }
}

impl Crep for Option<Type> {
    fn crep(self) -> String {
        match self {
            Option::None => "void".to_string(),
            Option::Some(t) => t.crep(),
        }
    }
}

impl Crep for Argument {
    fn crep(self) -> String {
        let arg_type_crep = self.arg_type.crep();
        let name_crep = self.name.crep();
        format!("{arg_type_crep} {name_crep}")
    }
}

impl Crep for ValueOps {
    fn crep(self) -> String {
        match self {
            ValueOps::Add | ValueOps::Fadd | ValueOps::PtrAdd => "+".to_string(),
            ValueOps::Sub | ValueOps::Fsub => "-".to_string(),
            ValueOps::Mul | ValueOps::Fmul => "*".to_string(),
            ValueOps::Div | ValueOps::Fdiv => "/".to_string(),
            ValueOps::Eq | ValueOps::Feq => "==".to_string(),
            ValueOps::Lt | ValueOps::Flt => "<".to_string(),
            ValueOps::Gt | ValueOps::Fgt => ">".to_string(),
            ValueOps::Le | ValueOps::Fle => "<=".to_string(),
            ValueOps::Ge | ValueOps::Fge => ">=".to_string(),
            ValueOps::Not => "!".to_string(),
            ValueOps::And => "&".to_string(),
            ValueOps::Or => "|".to_string(),

            ValueOps::Id => panic!("No crep for ValueOps::Id!"),
            ValueOps::Call => panic!("No crep for ValueOps::Call!"),
            ValueOps::Alloc | ValueOps::Load => panic!("No crep for memory ops!"),
        }
    }
}

impl Crep for Instruction {
    fn crep(self) -> String {
        match self {
            Instruction::Constant {
                dest,
                op,
                const_type,
                value,
            } => {
                let dest = dest.crep();
                let const_type_crep = const_type.crep();
                format!("{dest} = {value};")
            }
            Instruction::Value {
                args,
                dest,
                funcs,
                labels,
                op,
                op_type,
            } => match op {
                ValueOps::Add
                | ValueOps::PtrAdd
                | ValueOps::Fadd
                | ValueOps::Sub
                | ValueOps::Fsub
                | ValueOps::Mul
                | ValueOps::Fmul
                | ValueOps::Div
                | ValueOps::Fdiv
                | ValueOps::Eq
                | ValueOps::Feq
                | ValueOps::Lt
                | ValueOps::Flt
                | ValueOps::Gt
                | ValueOps::Fgt
                | ValueOps::Le
                | ValueOps::Fle
                | ValueOps::Ge
                | ValueOps::Fge
                | ValueOps::And
                | ValueOps::Or => {
                    let arg1 = &args[0].clone().crep();
                    let arg2 = &args[1].clone().crep();
                    let dest = dest.crep();
                    let op_crep = op.crep();
                    let op_type_crep = op_type.crep();
                    format!("{dest} = {arg1} {op_crep} {arg2};")
                }
                ValueOps::Not => {
                    let dest = dest.crep();
                    let arg1 = &args[0].clone().crep();
                    let op_crep = op.crep();
                    let op_type_crep = op_type.crep();
                    format!("{dest} = {op_crep}{arg1};")
                }
                ValueOps::Id => {
                    let dest = dest.crep();
                    let arg1 = &args[0].clone().crep();
                    let op_type_crep = op_type.crep();
                    format!("{dest} = {arg1};")
                }
                ValueOps::Call => {
                    let dest = dest.crep();
                    let func_name = func_rep(funcs[0].clone());
                    let op_type_crep = op_type.crep();
                    let args_crep = args
                        .into_iter()
                        .map(|x| x.clone().crep())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{dest} = {func_name}({args_crep});")
                }
                ValueOps::Alloc => {
                    let dest = dest.crep();
                    let arg1 = &args[0].clone().crep();
                    let op_type_crep = op_type.crep();
                    format!("{dest} = malloc({arg1}*sizeof({op_type_crep}));")
                }
                ValueOps::Load => {
                    let dest = dest.crep();
                    let arg1 = &args[0].clone().crep();
                    format!("{dest} = *({arg1});")
                }
            },
            Instruction::Effect {
                args,
                funcs,
                labels,
                op,
            } => match op {
                EffectOps::Jump => {
                    let label_name = &labels[0].clone().crep();
                    format!("goto {label_name};")
                }
                EffectOps::Branch => {
                    let cond = &args[0].clone().crep();
                    let tj = &labels[0].clone().crep();
                    let rj = &labels[1].clone().crep();
                    format!("if ({cond}) {{goto {tj};}} else {{goto {rj};}}")
                }
                EffectOps::Call => {
                    let func_name = func_rep(funcs[0].clone());
                    let args_crep = args
                        .into_iter()
                        .map(|x| x.clone().crep())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{func_name}({args_crep});")
                }
                EffectOps::Return => {
                    let args_crep = args
                        .into_iter()
                        .map(|x| x.clone().crep())
                        .collect::<Vec<_>>()
                        .join("");
                    format!("return {args_crep};")
                }
                EffectOps::Print => {
                    let mut ans = "".to_string();
                    for arg in args {
                        let arg = arg.clone().crep();
                        ans += &format!("print({arg});")
                    }
                    ans += r#"printf("\n");"#;
                    ans
                }
                EffectOps::Nop => ";".to_string(),
                EffectOps::Store => {
                    let loc = &args[0].clone().crep();
                    let val = &args[1].clone().crep();
                    format!("*{loc} = {val};")
                }
                EffectOps::Free => {
                    let loc = &args[0].clone().crep();
                    format!("free({loc});")
                }
            },
        }
    }
}

impl Crep for Code {
    fn crep(self) -> String {
        match self {
            Code::Label { label } => {
                let label_name = label.crep();
                format!("{label_name}:")
            }
            Code::Instruction(instr) => instr.crep(),
        }
    }
}

fn func_decl(func: &Function) -> String {
    let func = func.clone();
    let return_type_crep = func.return_type.crep();
    let name_crep = func_rep(func.name);
    let args_crep = &func
        .args
        .into_iter()
        .map(|f| f.crep())
        .collect::<Vec<_>>()
        .join(", ");

    format! {"{return_type_crep} {name_crep}({args_crep});\n"}
}

impl Crep for Function {
    fn crep(self) -> String {
        let return_type_crep = self.return_type.clone().crep();
        let name_crep = func_rep(self.name.clone());
        let args_crep = &self
            .args
            .iter()
            .map(|f| f.clone().crep())
            .collect::<Vec<_>>()
            .join(", ");
        let instrs_crep = &self
            .instrs
            .iter()
            .map(|f| f.clone().crep())
            .collect::<Vec<_>>()
            .join("\n");

        let static_vars_crep = find_static_vars(&self)
            .into_iter()
            .map(|f| f.crep())
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "{return_type_crep} {name_crep}({args_crep}){{\n{static_vars_crep}\n{instrs_crep}\n}}"
        )
    }
}

impl Crep for Program {
    fn crep(self) -> String {
        let functions_decl = &&self
            .functions
            .iter()
            .map(|f| func_decl(&f))
            .collect::<Vec<_>>()
            .join("\n");
        let functions_crep = &self
            .functions
            .iter()
            .map(|f| f.clone().crep())
            .collect::<Vec<_>>()
            .join("\n");
        let includes =
            "#include <stdint.h>\n#include <stdlib.h>\n#include <stdio.h>\n#include <inttypes.h>\n"
                .to_string();
        let print_macro = r#"#define print(x) _Generic((x), int64_t: printf("%" PRId64 " ", x), uint8_t: printf("%s ", (x) ? "true" : "false"), double: printf("%.17f ", x))"#.to_string();

        let mut main_func = "int main(int argc, char *argv[]){\n".to_string();
        let mut main_args: Vec<String> = Vec::new();

        for func in self.functions.iter() {
            if func.name == "main" {
                for (i, arg) in func.args.iter().enumerate() {
                    let i = i + 1;
                    let arg_type_crep = arg.arg_type.clone().crep();
                    let arg_name = &arg.name;
                    let conversion_func = match arg.arg_type {
                        Type::Int => "atoi",
                        Type::Bool => "atoi",
                        Type::Float => "atof",
                        Type::Pointer(_) => panic!("main can't take in a pointer!"),
                    };
                    let declaration_statement =
                        format!("{arg_type_crep} {arg_name} = {conversion_func}(argv[{i}]);\n");
                    main_args.push(arg_name.to_string());
                    main_func += &declaration_statement;
                }
            }
        }
        main_func += &format!("main_f({});\n", main_args.join(","));
        main_func += "return 0;\n}";

        let bool_defs = "uint8_t true = 1; uint8_t false = 0;";

        format!(
            "{includes}\n{bool_defs}\n{print_macro}\n\n{functions_decl}\n\n{functions_crep}\n{main_func}"
        )
    }
}
fn main() {
    let program = load_program();
    println!("{}", program.crep());
}
