use std::fmt::{self, Display};

use crate::parser::{Bril, Function, Instr, Opcode};

pub struct BrilCFG {
    bril: Bril,
    unamed_block_cnt: usize,
    cur_name: Option<String>,
    pub blocks: Vec<Block>,
}

pub struct Block {
    name: String,
    pub(crate) instrs: Vec<Instr>,
    succ: Option<Vec<String>>,
    func: String,
}

const TERMINATOR: [Opcode; 3] = [Opcode::jmp, Opcode::ret, Opcode::br];

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.name).unwrap();
        for instr in &self.instrs {
            writeln!(f, "\t{:?}:", instr).unwrap();
        }
        writeln!(f, "succ: {:?}", self.succ).unwrap();
        Ok(())
    }
}

impl BrilCFG {
    pub fn new(bril: Bril) -> Self {
        Self {
            bril,
            unamed_block_cnt: 0,
            cur_name: None,
            blocks: vec![],
        }
    }
    pub fn resolve_cfg(&mut self) {
        for (cnt, block) in self.blocks.iter().enumerate() {
            if let Some(last) = block.instrs.last() {
                use crate::parser::Instr::*;
                let mut succ = None;
                if let Instruction { op, labels, .. } = last {
                    if [Opcode::jmp, Opcode::br].contains(op) {
                        let labels = labels.as_ref().unwrap();
                        succ = Some(labels.clone())
                    } else if op == &Opcode::ret {
                        succ = None
                    } else {
                        // get next block
                        if let Some(next_block) = self.blocks.get(cnt + 1) {
                            succ = Some(vec![next_block.name.clone()]);
                        } else {
                            succ = None;
                        }
                    }
                    let ptr = self.blocks.as_ptr() as *mut Block;
                    unsafe {
                        // cur block will never be used afterwards, so it's safe to
                        // change current block
                        // so unsafe code is applied to bypass the dumb checking system
                        (*ptr.add(cnt)).succ = succ.clone();
                    }
                } else {
                    panic!("unexpected label instruction");
                }
            } else {
                panic!("unexpected empty block");
            }
        }
    }
    pub fn parse_blocks(&mut self) {
        let mut instrs = vec![];
        let mut cur_func_name = "".to_string();
        for func in self.bril.functions.clone() {
            cur_func_name = func.name.clone();
            self.set_cur_block_name(&cur_func_name);
            for instr in &func.instrs {
                use crate::parser::Instr::*;
                match instr {
                    Instruction { op, .. } => {
                        instrs.push(instr.clone());
                        if TERMINATOR.contains(op) {
                            let block = Block {
                                name: self.cur_block_name(),
                                instrs: instrs.clone(),
                                succ: None,
                                func: cur_func_name.clone(),
                            };
                            self.blocks.push(block);
                            instrs.clear();
                        }
                    }
                    Label { label } => {
                        let block = Block {
                            name: self.cur_block_name(),
                            instrs: instrs.clone(),
                            succ: None,
                            func: cur_func_name.clone(),
                        };
                        self.blocks.push(block);
                        instrs.clear();
                        self.set_cur_block_name(label);
                    }
                }
            }
        }
        if !instrs.is_empty() {
            let block = Block {
                name: self.cur_block_name(),
                instrs: instrs.clone(),
                succ: None,
                func: cur_func_name,
            };
            self.blocks.push(block);
        }
        self.resolve_cfg();
    }

    // TODO: transform from cfg to original bril
    pub fn to_bril(&self) -> Bril {
        let mut cur_name = String::new();
        let mut cur_func = None;
        let mut funcs = vec![];
        for block in &self.blocks {
            if cur_name != block.func {
                if let Some(func) = cur_func.take() {
                    funcs.push(func);
                }
                cur_name = block.func.clone();
                cur_func = self.get_func_by_name(&cur_name);
            }
            cur_func = cur_func.map(|mut func| {
                if block.name != cur_name {
                    // add label
                    let label = Instr::Label {
                        label: block.name.clone(),
                    };
                    func.instrs.push(label);
                }
                func.instrs.append(&mut block.instrs.clone());
                func
            });
        }
        if let Some(cur_func) = cur_func {
            funcs.push(cur_func);
        }
        Bril { functions: funcs }
    }

    // the return function have empty instrs
    fn get_func_by_name(&self, name: &str) -> Option<Function> {
        for func in &self.bril.functions {
            if func.name == name {
                let mut res = func.clone();
                res.instrs.clear();
                return Some(res);
            }
        }
        None
    }

    fn set_cur_block_name(&mut self, name: &str) {
        self.cur_name = Some(name.to_string())
    }
    fn cur_block_name(&mut self) -> String {
        if let Some(name) = self.cur_name.take() {
            name
        } else {
            // FIXME: what if some label is start with tmp?
            let tmp_name = format!("tmp{}", self.unamed_block_cnt);
            self.unamed_block_cnt += 1;
            tmp_name
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::bril2json;

    use super::*;
    // TODO: test on bril's test directory

    #[test]
    fn cfg_jmp() {
        let bril_text = r#"@main{
        v: int = const 4;
        jmp .somewhere;
        v: int = const 2;
        .somewhere:
        print v;
}"#;

        let bril_json = bril2json(bril_text);
        println!("bril_json: {bril_json}");

        let bril: Bril = serde_json::from_str(&bril_json).unwrap();
        let mut cfg = BrilCFG::new(bril);
        cfg.parse_blocks();
        for block in &cfg.blocks {
            println!("{block}");
        }

        let bril = cfg.to_bril();
        let res = serde_json::to_string(&bril).expect("cannot convert bril {bril:?}");
        assert_eq!(
            res,
            r#"{"functions":[{"name":"main","instrs":[{"op":"const","dest":"v","type":"int","value":4},{"op":"jmp","labels":["somewhere"]},{"label":"tmp0"},{"op":"const","dest":"v","type":"int","value":2},{"label":"somewhere"},{"op":"print","args":["v"]}]}]}"#
        );
    }
}
