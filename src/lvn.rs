use std::collections::HashMap;

use crate::{
    cfg::{Block, BrilCFG},
    parser::{Instr, Literal, Opcode},
};

type VarName = String;
type VarNum = usize;

pub struct LVN {
    table: HashMap<LVNTuple, (VarNum, VarName)>,
    var2num: HashMap<VarName, VarNum>,
    num2tuple: HashMap<VarNum, LVNTuple>,
    cur_num: VarNum,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct LVNTuple {
    op: LVNOpcode,
    args: Vec<VarNum>,
}

#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum LVNOpcode {
    add,
    mul,
    sub,
    div,
    eq,
    lt,
    gt,
    le,
    ge,
    not,
    and,
    or,
    jmp,
    br,
    call,
    ret,
    id,
    print,
    nop,
    cst(Literal)
}

impl LVNOpcode {
    pub fn from_instr(instr: &Instr) -> Self {
        if let Instr::Instruction { op, value, ..}  = instr {
            let mut vals = vec![];
            match op {
                Opcode::cst => {
                    if let Some(value) = value {
                        vals.push(value.clone());
                    } else {
                        panic!("const without value");
                    }
                }
                _ => {}
            }
            Self::from_opcode(op.clone(), &vals)
        } else {
            panic!("unexpected label {instr:?}");
        }
    }

    fn from_opcode(op: Opcode, val: &Vec<Literal>) -> Self {
        match op {
            Opcode::add => LVNOpcode::add,
            Opcode::mul => LVNOpcode::mul,
            Opcode::sub => LVNOpcode::sub,
            Opcode::div => LVNOpcode::div,
            Opcode::eq => LVNOpcode::eq,
            Opcode::lt => LVNOpcode::lt,
            Opcode::gt => LVNOpcode::gt,
            Opcode::le => LVNOpcode::le,
            Opcode::ge => LVNOpcode::ge,
            Opcode::not => LVNOpcode::not,
            Opcode::and => LVNOpcode::and,
            Opcode::or => LVNOpcode::or,
            Opcode::jmp => LVNOpcode::jmp,
            Opcode::br => LVNOpcode::br,
            Opcode::call => LVNOpcode::call,
            Opcode::ret => LVNOpcode::ret,
            Opcode::id => LVNOpcode::id,
            Opcode::print => LVNOpcode::print,
            Opcode::nop => LVNOpcode::nop,
            Opcode::cst => {
                assert!(val.len() == 1);
                LVNOpcode::cst(val[0].clone())
            },
        }
    }
}

impl BrilCFG {
    pub fn lvn(&mut self) {
        for block in self.blocks.iter_mut() {
            block.lvn();
        }
    }
}

impl Block {
    pub fn lvn(&mut self) {
        assert!(self.lvn.is_none(), "calling lvn multiple times");
        let mut lvn = LVN::new();
        let mut rewrite = HashMap::new();
        for instr in &self.instrs {
            let tuple = lvn.tuple_from_instr(instr);
            if lvn.table.contains_key(&tuple) {
                let (num, var) = &lvn.table[&tuple];
                if let Instr::Instruction { dest, typ, .. } = instr {
                    if let Some(dest) = dest {
                        let typ = typ.as_ref().expect("instr {instr} does't have type");
                        // replace instr with copy of var
                        let new_instr = Instr::new_id_instr(dest, var, typ.clone());
                        rewrite.insert(instr.clone(), new_instr);
                        lvn.var2num.insert(dest.clone(), *num);
                    }
                }
            } else {
                if let Instr::Instruction { op, dest, args,  .. } = instr {
                    if let Some(dest) = dest {
                        let num = lvn.next_var_num();
                        // FIXME: what if the dest is duplicated afterwards
                        if op == &Opcode::id {
                            let args = args.as_ref().unwrap();
                            lvn.table.insert(tuple.clone(), (num, args[0].clone()));
                        } else {
                            lvn.table.insert(tuple.clone(), (num, dest.clone()));
                        }
                        lvn.num2tuple.insert(num, tuple);
                        lvn.var2num.insert(dest.clone(), num);
                    } else {
                        // don't do anything
                    }

                    let new_instr = lvn.rewrite_instr_args(instr);
                    rewrite.insert(instr.clone(), new_instr);
                }
            }
        }

        let new_instr = self
            .instrs
            .iter()
            .map(|x| {
                if let Some(instr) = rewrite.get(x) {
                    instr
                } else {
                    x
                }
            })
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        self.instrs = new_instr;
    }
}

impl LVN {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            var2num: HashMap::new(),
            num2tuple: HashMap::new(),
            cur_num: 0,
        }
    }
    fn debug(&self) {
        println!("{:?}", self.table);
        println!("{:?}", self.var2num);
        println!("{:?}", self.num2tuple);
    }
    pub fn next_var_num(&mut self) -> VarNum {
        let ret = self.cur_num;
        self.cur_num += 1;
        ret
    }
    pub fn rewrite_instr_args(&self, instr: &Instr) -> Instr {
        let mut new_args = vec![];
        if let Instr::Instruction { args, .. } = instr {
            if let Some(args) = args {
                for arg in args {
                    new_args.push(self.replace_var(arg));
                }
            }
        }

        let mut ret = instr.clone();
        if let Instr::Instruction { args, .. } = &mut ret {
            args.replace(new_args);
        }

        ret
    }
    fn replace_var(&self, var: &str) -> String {
        if let Some(num) = self.var2num.get(var) {
            if let Some(index) = self.num2tuple.get(num) {
                if let Some((_, var)) = self.table.get(index) {
                    var.clone()
                } else {
                    panic!("cannot find lvn entry of num {num}, index: {index:?}");
                }
            } else {
                panic!("cannot find lvn entry of num {num}");
            }
        } else {
            var.to_string()
        }
    }
    pub fn tuple_from_instr(&self, instr: &Instr) -> LVNTuple {
        if let Instr::Instruction { op, args, .. } = instr {
            let args = if let Some(args) = args {
                args.iter()
                    .map(|arg| {
                        self.resolve_arg(arg)
                    })
                    .collect()
            } else {
                vec![]
            };
            LVNTuple {
                op: LVNOpcode::from_instr(instr),
                args,
            }
        } else {
            panic!("try to convert from label {instr:?} to LVNTuple");
        }
    }

    fn resolve_arg(&self, arg: &str) -> VarNum {
        let num = self.var2num.get(arg).expect("cannot find variable {arg} in lvn");
        let tuple = &self.num2tuple[num];
        if tuple.op == LVNOpcode::id {
            return tuple.args[0]
        }
        *num
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg::BrilCFG;

    #[test]
    fn lvn() {
        let bril_text = r#"@main{
        a: int = const 4;
        b: int = const 2;
        sum1: int = add a b;
        sum2: int = add a b;
        prod: int = mul sum1 sum2;
        print prod;
}"#;
        let mut cfg = BrilCFG::from_text(bril_text);
        cfg.lvn();
        cfg.trivial_dce();
        let bril_txt = cfg.to_text();
        println!("out: {bril_txt}");
        assert!(!bril_txt.contains("sum2: int"));
    }

    #[test]
    fn copy_propagation() {
        let bril_text = r#"@main{
        x: int = const 4;
        copy1: int = id x;
        copy2: int = id copy1;
        copy3: int = id copy2;
        print copy3;
}"#;
        let mut cfg = BrilCFG::from_text(bril_text);
        cfg.lvn();
        cfg.trivial_dce();
        let bril_txt = cfg.to_text();
        println!("out: {bril_txt}");
        assert!(!bril_txt.contains("copy1: int"));
        assert!(!bril_txt.contains("copy2: int"));
        assert!(!bril_txt.contains("copy3: int"));
    }
}
