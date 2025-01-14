use std::fmt::{self, Display};

use crate::parser::{Bril, Instr, Opcode};

pub struct BrilCFG {
    bril: Bril,
    unamed_block_cnt: usize,
    cur_name: Option<String>,
    pub blocks: Vec<Block>
}

pub struct Block {
    name: String,
    instrs: Vec<Instr>,
    succ: Option<Vec<String>>
}

const TERMINATOR: [Opcode; 3] = [Opcode::jmp, Opcode::ret, Opcode::br];

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", self.name).unwrap();
        for instr in &self.instrs {
            writeln!(f, "\t{:?}:", instr).unwrap();
        }
        Ok(())
    }
}

impl BrilCFG {
    pub fn new(bril: Bril) -> Self {
        Self {
            bril,
            unamed_block_cnt: 0,
            cur_name: None,
            blocks: vec![]
        }
    }
    pub fn resolve_cfg(&mut self) {
        for (cnt, block) in self.blocks.iter().enumerate() {
            if let Some(last) = block.instrs.last() {
                use crate::parser::Instr::*;
                let mut succ = None;
                if let Instruction {op, labels, ..} = last {
                    if [Opcode::jmp, Opcode::br].contains(op) {
                        let labels  = labels.as_ref().unwrap();
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
                        (*ptr.add(cnt)).succ = succ;
                    }
                } else {
                    panic!("unexpected empty block");
                }
            } else {
                panic!("unexpected empty block");
            }
        }
    }
    pub fn parse_blocks(&mut self)  {
        let mut instrs = vec![];
        for func in self.bril.functions.clone() {
            self.set_cur_block_name(&func.name);
            for instr in &func.instrs {
                use crate::parser::Instr::*;
                match instr {
                    Instruction { op, .. } => {
                        instrs.push(instr.clone());
                        if TERMINATOR.contains(op) {
                            let block = Block {name: self.cur_block_name(), instrs: instrs.clone(), succ: None};
                            self.blocks.push(block);
                            instrs.clear();
                        }
                    },
                    Label { label } => {
                        let block = Block {name: self.cur_block_name(), instrs: instrs.clone(), succ: None};
                        self.blocks.push(block);
                        instrs.clear();
                        self.set_cur_block_name(label);
                    },
                }
            }
        }
    }

    fn set_cur_block_name(&mut self, name: &str) {
        self.cur_name = Some(name.to_string())
    }
    fn cur_block_name(&mut self) -> String {
        if let Some(name) = self.cur_name.take() {
            name
        } else {
            let tmp_name = format!("tmp{}", self.unamed_block_cnt);
            self.unamed_block_cnt += 1;
            tmp_name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_jmp() {

    }
}

