use pest_derive::Parser;
use pest::Parser;
use std::fs;
use std::io::stdin;
use std::collections::hash_map::*;
use enum_primitive::*;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

const NOP_MASK : u16 = 0xFFFF;
const INSTRUCTION_MASK : u16 = 0xF000;
const ARGUMENT_MASK : u16 = 0x0FFF;
const FIRST_NEGATIVE : u16 = 0x8000;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

enum_from_primitive! {
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
                let output = ((instruction as u16) << 12) + (address & ARGUMENT_MASK);
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
                                let local = far & ARGUMENT_MASK;
                                for &addr in addresses {
                                    let a = &mut binary[addr as usize];
                                    *a = (*a & INSTRUCTION_MASK) + if *a==0 {far} else {local};
                                }
                            }
                            Label::Linked(addr) => warn!("Label defined twice! label {:?}, defined at {:04X}, already defined at {:04X}", label_name, binary.len() as u16, addr) // DoubleLabel
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
                warn!("Undefined Label {:?}, referenced in byte(s) {:04X?}", name, list)
            }
        }
    }
    binary
}

fn run_program(mut memory: Vec<u16>, extended_mode: bool){
    let mut accumulator = 0;
    let mut program_counter : u16 = 0;
    let indirect_mask = if extended_mode {NOP_MASK} else {ARGUMENT_MASK};
    let full_address_length = if extended_mode {16} else {12};
    memory.resize(1<<full_address_length, 0);
    loop{
        let instruction = memory[program_counter as usize];
        let argument = instruction & ARGUMENT_MASK;
        trace!("PC: {:03X}, INST: {:04X}", program_counter, instruction);
        program_counter += 1;
        match Instruction::from_u16(instruction>>12).unwrap(){
            Instruction::JnS     =>{// Operand
                trace!("JnS to {:03X} from {:04X}", argument, program_counter);
                memory[argument as usize] = program_counter;
                program_counter = argument + 1;
            },
            Instruction::Load    =>{// Operand
                accumulator = memory[argument as usize];
            },
            Instruction::Store   =>{// Operand
                memory[argument as usize] = accumulator;
            },
            Instruction::Add     =>{// Operand
                accumulator = accumulator.wrapping_add(memory[argument as usize]);
            },
            Instruction::Subt    =>{// Operand
                accumulator = accumulator.wrapping_sub(memory[argument as usize]);
            },
            Instruction::Skipcond=>{// Operand
                let negative : bool = accumulator.wrapping_sub(1) > FIRST_NEGATIVE;
                let positive : bool = accumulator < FIRST_NEGATIVE;
                let arg = argument >> 10;
                const SKIPCOND_MAP : [[bool; 2];4] = [
                    [false, true],
                    [false, false],
                    [true,  false],
                    [true,  true]

                ];
                let skipcond = SKIPCOND_MAP[arg as usize];
                let result = (skipcond[0] != negative) && (skipcond[1] != positive);
                trace!("acc {}, n {}, p {}, arg {} {}, sc {:?}, res {}",
                    accumulator,
                    negative,
                    positive,
                    argument,
                    arg,
                    skipcond,
                    result
                );
                if result{
                    program_counter += 1
                }
            },
            Instruction::Jump    =>{// Operand
                trace!("Jump to {:03X} from {:04X}", argument, program_counter);
                program_counter = (program_counter & INSTRUCTION_MASK) + argument;
            },
            Instruction::AddI    =>{// Operand
                let addr = indirect_mask & memory[argument as usize];
                accumulator = accumulator.wrapping_add(memory[addr as usize]);
            },
            Instruction::JumpI   =>{// Operand
                let addr = indirect_mask & memory[argument as usize];
                trace!("JumpI to {:04X} from {:04X}", addr, program_counter);
                program_counter = addr;
            },
            Instruction::LoadI   =>{// Operand
                accumulator = memory[memory[argument as usize] as usize];
            },
            Instruction::StoreI  =>{// Operand
                trace!("StoreI {:04X} {:03X} {:04X}", accumulator, argument, memory[argument as usize]);
                let addr = indirect_mask & memory[argument as usize];
                memory[addr as usize] = accumulator;
            },
            Instruction::Input   =>{
                info!("input: ");
                let mut buffer = String::new();
                stdin().read_line(&mut buffer).expect("unable to read from stdin");
                accumulator = buffer.trim().parse().expect("invalid number from stdin");
            },
            Instruction::Output  =>{
                info!("Output: {:04X} {}", accumulator, accumulator);
            },
            Instruction::Halt    => return,
            Instruction::Clear   =>{
                accumulator = 0;
            },
            Instruction::Invalid => ()
        }
    }
}

fn main() {
    fern::Dispatch::new()
          .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        //.level_for("blah", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply().expect("Could not initialize fern logger");

    const FILENAMES: [&str; 3] = [
        "multiply.mas",
        "heap5.mas",
        "heap6.mas",
    ];
    for filename in FILENAMES.iter(){
        info!("Parsing and interpreting {:?}", filename);
        let unparsed_file = fs::read_to_string(filename).expect("cannot read file");
        let binary = assemble(&unparsed_file);
        debug!("{:04X?}", binary);
        run_program(binary, true);
    }
}
