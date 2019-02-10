use pest_derive::Parser;
use pest::Parser;
use std::fs;
use std::collections::hash_map::*;

const LOCAL_ADDRESS_MASK : u16 = 0x0FFF;
const INSTRUCTION_MASK : u16 = 0xF000;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

#[derive(Copy, Clone, Debug)]
enum Instruction{
    JnS     = 0x0,// Operand
    Load    = 0x1,// Operand
    Store   = 0x2,// Operand
    Add     = 0x3,// Operand
    Subt    = 0x4,// Operand
    Input   = 0x5,
    Output  = 0x6,
    Halt    = 0x7,
    Skipcond= 0x8,// Operand
    Jump    = 0x9,// Operand
    Clear   = 0xA,
    AddI    = 0xB,// Operand
    JumpI   = 0xC,// Operand
    LoadI   = 0xD,// Operand
    StoreI  = 0xE,// Operand
    Invalid = 0xF
}

#[derive(Debug)]
enum Label{
    Unlinked(Vec<u16>),
    Linked(u16)
}

fn label_to_addr<'a>(labels: &mut HashMap<&'a str, Label>, name: &'a str, current_addr: u16, default: u16) -> u16{
    match labels.entry(name).or_insert_with(|| Label::Unlinked(vec![])){
        Label::Unlinked(list) => {
            list.push(current_addr);
            default
        },
        Label::Linked(address) => *address
    }
}

fn assemble(unparsed_file: &str) -> Vec<u16>{
    let file = ASMParser::parse(Rule::File, &unparsed_file)
        .expect("unsuccessful parse"); // unwrap the parse result
    let mut labels = HashMap::new();
    let mut binary : Vec<u16> = vec![];
    for line in file{
        match line.as_rule() {
            Rule::InstructionWithArg =>{
                let mut iter = line.into_inner();
                let instruction = match iter.next().unwrap().as_rule(){
                    Rule::JnS => Instruction::JnS,
                    Rule::Load => Instruction::Load,
                    Rule::Store => Instruction::Store,
                    Rule::Add => Instruction::Add,
                    Rule::Subt => Instruction::Subt,
                    Rule::Jump => Instruction::Jump,
                    Rule::Skipcond => Instruction::Skipcond,
                    Rule::AddI => Instruction::AddI,
                    Rule::JumpI => Instruction::JumpI,
                    Rule::LoadI => Instruction::LoadI,
                    Rule::StoreI => Instruction::StoreI,
                    _ => unreachable!()
                };
                let operand = iter.next().unwrap();
                let address : u16 = match operand.as_rule(){
                    Rule::Name => label_to_addr(&mut labels, operand.as_str(), binary.len() as u16, 1),
                    Rule::Address => u16::from_str_radix(operand.as_str(),16).unwrap(),
                    _ => unreachable!()
                };
                let output = ((instruction as u16) << 12) + (address & LOCAL_ADDRESS_MASK);
                binary.push(output);
            },
            Rule::InstructionNoArg =>{
                let instruction = match line.into_inner().next().unwrap().as_rule(){
                    Rule::Input => Instruction::Input,
                    Rule::Output => Instruction::Output,
                    Rule::Clear => Instruction::Clear,
                    Rule::Halt => Instruction::Halt,
                    Rule::Breakpoint => Instruction::Invalid,
                    _ => unreachable!()
                };
                let output = (instruction as u16) << 12;
                binary.push(output);
            },
            Rule::Literal =>{
                let literal = line.into_inner().next().unwrap();
                let value = match literal.as_rule(){
                    Rule::DecLiteral => literal
                        .as_str()
                        .parse()
                        .expect("Invalid Dec Literal"),
                    Rule::HexLiteral => u16::from_str_radix(literal.as_str(), 16)
                        .expect("Invalid Hex Literal"),
                    Rule::LblLiteral => label_to_addr(
                        &mut labels,
                        literal.as_str(),
                        binary.len() as u16,
                        0 // Zero will allow a wide address to be written
                    ),
                    _ => unreachable!()
                };
                binary.push(value);
            },
            Rule::Label =>{
                let label_name = line.into_inner().next().unwrap().as_str();
                match labels.entry(label_name){
                    Entry::Occupied(mut occupied) => {
                        match occupied.get(){
                            Label::Unlinked(addresses) => {
                                let far = binary.len() as u16;
                                let local = far & LOCAL_ADDRESS_MASK;
                                for &addr in addresses {
                                    let a = &mut binary[addr as usize];
                                    *a = (*a & INSTRUCTION_MASK) + if *a==0 {far} else {local};
                                }
                            }
                            Label::Linked(addr) => eprintln!("Warning: Label defined twice! label {:?}, defined at {:04X}, already defined at {:04X}", label_name, binary.len() as u16, addr) // DoubleLabel
                        }
                        occupied.insert(Label::Linked(binary.len() as u16));
                    },
                    Entry::Vacant(vacant) => {vacant.insert(Label::Linked(binary.len() as u16));},
                }
            },

            Rule::EOI => (),
            _ => unreachable!()
        }
    }
    for pair in labels.iter(){
        let (name, label) = pair;
        match label{
            Label::Linked(_) => (),
            Label::Unlinked(list) => {
                eprintln!("Warning: Undefined Label {:?}, referenced in byte(s) {:04X?}", name, list)
            }
        }
    }
    binary
}

fn main() {
    let unparsed_file = fs::read_to_string("multiply.mas").expect("cannot read file");
    let binary = assemble(&unparsed_file);
    println!("{:04X?}", binary)
}
