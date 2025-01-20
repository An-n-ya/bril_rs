use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bril {
    pub(crate) functions: Vec<Function>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) args: Option<Vec<Arg>>,
    #[serde(rename="type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) typ: Option<Type>,
    pub(crate) instrs: Vec<Instr>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Instr {
    Instruction {
        op: Opcode,
        #[serde(skip_serializing_if = "Option::is_none")]
        dest: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename="type")]
        typ: Option<Type>,
        #[serde(skip_serializing_if = "Option::is_none")]
        args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        funcs: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        labels: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<Literal>,
    },
    Label {
        label: String 
    }
}

impl Instr {
    pub fn new_id_instr(dest: &str, src: &str) -> Self {
        Instr::Instruction {
            op: Opcode::id,
            dest: Some(dest.to_string()),
            args: Some(vec![src.to_string()]),
            typ: None,
            funcs: None,
            labels: None,
            value: None,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Opcode {
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
    #[serde(rename="const")]
    cst
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arg {
    name: String,
    #[serde(rename="type")]
    typ: Type
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    int,
    bool
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Literal {
    Number(usize),
    Bool(bool)
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bril_json_parse() {
        let s = r#"
{
  "functions": [
    {
      "instrs": [
        {
          "dest": "v0",
          "op": "const",
          "type": "int",
          "value": 1
        }
      ],
      "name": "main"
    }
  ]
}
"#;

        // let inst = Instr::Instruction{op: Opcode::add, dest: Some("hello".to_string()), typ: Some(Type::int), args: None, funcs: None, labels: None };
        // let func = Function{name: "hello".to_string(), instrs: vec![inst], typ: None, args: None};
        // let bril = Bril{functions: vec![func]};
        // let bril_str = serde_json::to_string(&bril).unwrap();
        // println!("bri: {bril_str}");
        serde_json::from_str::<Bril>(&s).expect("cannot parse functions");
    }

    #[test]
    fn bril_json_parse_instruction() {
        let s = r#"
        {
          "dest": "v0",
          "op": "const",
          "type": "int",
          "value": 1
        }
"#;
        serde_json::from_str::<Instr>(&s).expect("cannot parse functions");
    }

    #[test]
    fn bril_json_parse_label() {
        let s = r#"
        {
          "label": "hello"
        }
"#;
        serde_json::from_str::<Instr>(&s).expect("cannot parse functions");
    }

}
