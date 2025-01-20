use std::collections::HashMap;

use crate::{
    cfg::Block,
    parser::{Instr, Opcode},
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
    op: Opcode,
    args: Vec<VarNum>,
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
                if let Instr::Instruction { dest, .. } = instr {
                    if let Some(dest) = dest {
                        // replace instr with copy of var
                        let new_instr = Instr::new_id_instr(dest, var);
                        rewrite.insert(instr.clone(), new_instr);
                        lvn.var2num.insert(dest.clone(), *num);
                    }
                }
            } else {
                if let Instr::Instruction { dest, args, .. } = instr {
                    if let Some(dest) = dest {
                        let num = lvn.next_var_num();
                        // FIXME: what if the dest is duplicated afterwards
                        lvn.table.insert(tuple.clone(), (num, dest.clone()));
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
                        self.var2num
                            .get(arg)
                            .expect("cannot find variable {arg} in lvn")
                            .clone()
                    })
                    .collect()
            } else {
                vec![]
            };
            LVNTuple {
                op: op.clone(),
                args,
            }
        } else {
            panic!("try to convert from label {instr:?} to LVNTuple");
        }
    }
}


