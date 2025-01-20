use std::collections::{HashMap, HashSet};

use crate::cfg::{Block, BrilCFG};

impl BrilCFG {
    pub fn trivial_dce(&mut self) {
        for block in self.blocks.iter_mut() {
            block.trivial_dce();
            block.trivial_dce2();
        }
    }
}

use crate::parser::Instr::{self, *};
impl Block {
    pub fn iterate_every_instr<F>(&self, mut f: F)
    where
        F: FnMut(&Instr) -> (),
    {
        for instr in &self.instrs {
            match instr {
                instr @ Instruction { .. } => {
                    f(instr);
                }
                Label { label } => {
                    panic!("Unexpected label: {label}");
                }
            }
        }
    }

    pub fn trivial_dce(&mut self) {
        loop {
            let mut flag = false;
            let mut to_be_deleted = vec![];
            let mut used = HashSet::new();

            self.iterate_every_instr(|instr| {
                if let Instruction { args, .. } = instr {
                    if let Some(args) = args {
                        for arg in args {
                            used.insert(arg.clone());
                        }
                    }
                }
            });
            self.iterate_every_instr(|instr| {
                if let Instruction { dest, .. } = instr {
                    if let Some(dest) = dest {
                        if !used.contains(dest) {
                            to_be_deleted.push(instr.clone());
                            flag = true;
                        }
                    }
                }
            });
            if !flag {
                break;
            }

            let new_instr = self
                .instrs
                .iter()
                .filter(|x| !to_be_deleted.contains(x))
                .map(|x| x.clone())
                .collect::<Vec<_>>();
            self.instrs = new_instr;
        }
    }

    pub fn trivial_dce2(&mut self) {
        loop {
            let mut flag = false;
            let mut to_be_deleted = vec![];
            let mut last_defs: HashMap<String, Instr> = HashMap::new();
            self.iterate_every_instr(|instr| {
                if let Instruction {args, dest, ..}  =instr {
                    // for each use
                    if let Some(args) = args {
                        for arg in args {
                            if last_defs.contains_key(arg) {
                                // def used
                                last_defs.remove(arg);
                            }
                        }
                    }
                    // for each defines
                    if let Some(dest) = dest {
                        if last_defs.contains_key(dest) {
                            // FIXME: what if the current instr is exact same as last_def instr
                            //          in this case, current instr will be deleted as well
                            //          i.e.  a: int = const 1;
                            //                a: int = const 1;
                            to_be_deleted.push(last_defs.get(dest).unwrap().clone());
                            flag = true;
                        } else {
                            last_defs.insert(dest.clone(), instr.clone());
                        }
                    }
                }
            });


            if !flag {
                break;
            }

            let new_instr = self
                .instrs
                .iter()
                .filter(|x| !to_be_deleted.contains(x))
                .map(|x| x.clone())
                .collect::<Vec<_>>();
            self.instrs = new_instr;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial_dce() {
        let bril_text = r#"@main{
        a: int = const 4;
        b: int = const 2;
        c: int = const 1;
        a: int = const 8;
        d: int = add a b;
        print d;
}"#;

        let mut cfg = BrilCFG::from_text(bril_text);


        cfg.trivial_dce();
        let bril_txt = cfg.to_text();

        println!("bril_txt: {bril_txt}");
        assert!(!bril_txt.contains("c: int = const 1;"));
        assert!(!bril_txt.contains("a: int = const 4;"));
    }
}
