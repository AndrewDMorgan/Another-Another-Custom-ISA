use std::io::Write;
use std::io::Read;

#[derive(Clone, Debug)]
struct SudoInstruction {
    pub name: &'static str,
    // name, parameters, final instruction (* is a placeholder for where the parameters/operands are inserted)
    pub conversions: &'static [(&'static str, &'static [Param], &'static str)],
}

#[derive(Clone, Debug)]
struct Instruction {
    pub name: &'static str,
    pub params: &'static [Param],
    pub op_code: u8,
    pub cycle_cost: usize,
}

#[derive(Clone, Debug)]
enum Param {
    Reg,
    Const16,
    Const8,
    Addr16,
    Addr32,
    Ptr,
}

static SUDO_INSTRUCTIONS: &[SudoInstruction] = &[
    SudoInstruction { name: "Add"    , conversions: &[("Add", &[Param::Reg, Param::Reg, Param::Reg], "Add * * *"), ("AddImm", &[Param::Reg, Param::Const16, Param::Reg], "AddImm * * *")] },
    SudoInstruction { name: "Sub"    , conversions: &[("Sub", &[Param::Reg, Param::Reg, Param::Reg], "Sub * * *"), ("SubImm", &[Param::Reg, Param::Const16, Param::Reg], "SubImm * * *")] },
    SudoInstruction { name: "Mul"    , conversions: &[("Mul", &[Param::Reg, Param::Reg, Param::Reg], "Mul * * *"), ("MulImm", &[Param::Reg, Param::Const16, Param::Reg], "MulImm * * *")] },
    SudoInstruction { name: "Div"    , conversions: &[("Div", &[Param::Reg, Param::Reg, Param::Reg], "Div * * *"), ("DivImm", &[Param::Reg, Param::Const16, Param::Reg], "DivImm * * *")] },
    SudoInstruction { name: "Mod"    , conversions: &[("Mod", &[Param::Reg, Param::Reg, Param::Reg], "Mod * * *"), ("ModImm", &[Param::Reg, Param::Const16, Param::Reg], "ModImm * * *")] },
    SudoInstruction { name: "Mov"    , conversions: &[("Mov", &[Param::Reg, Param::Reg], "Mov * *"), ("MovR", &[Param::Addr16, Param::Addr16], "MovR * *")] },
    SudoInstruction { name: "Ldi"    , conversions: &[("Ldi", &[Param::Reg, Param::Const16], "Ldi * *"), ("LdiR", &[Param::Addr16, Param::Const16], "LdiR * *"), ("LdiPtr", &[Param::Ptr, Param::Const16], "LdiPtr * * *")] },
    SudoInstruction { name: "MemCpy" , conversions: &[("MemCpy", &[Param::Addr16, Param::Addr16, Param::Const8], "MemCpy * * *"), ("MemCpyPtr", &[Param::Ptr, Param::Ptr, Param::Reg], "MemCpyPtr * * *")] },
    SudoInstruction { name: "MemCmp" , conversions: &[("MemCmp", &[Param::Addr16, Param::Addr16, Param::Const8], "MemCmp * * *"), ("MemCmpPtr", &[Param::Ptr, Param::Ptr, Param::Reg], "MemCmpPtr * * *")] },
    SudoInstruction { name: "Sto"    , conversions: &[("Sto", &[Param::Addr16, Param::Reg], "Sto"), ("StoPtr", &[Param::Ptr, Param::Reg], "StoPtr * * *"), ("StoPtrOff", &[Param::Ptr, Param::Const16, Param::Reg], "StoPtrOff * * *")] },
    SudoInstruction { name: "Get"    , conversions: &[("Get", &[Param::Addr16, Param::Reg], "Get"), ("GetPtr", &[Param::Ptr, Param::Reg], "GetPtr * * *"), ("GetPtrOff", &[Param::Ptr, Param::Const16, Param::Reg], "GetPtrOff * * *")] },
    SudoInstruction { name: "Psh"    , conversions: &[("Psh", &[Param::Reg], "Psh"), ("PshCon", &[Param::Const16], "PshCon"), ("PshPtr", &[Param::Ptr], "PshPtr")] },
];

static INSTRUCTIONS: &[Instruction] = &[
    Instruction { name: "Nop"          , params: &[], op_code: 0b0000_0000, cycle_cost: 1 },
    Instruction { name: "Ldi"          , params: &[Param::Reg, Param::Const16], op_code: 0b0000_0001, cycle_cost: 1 },
    Instruction { name: "Mov"          , params: &[Param::Reg, Param::Reg], op_code: 0b0000_0010, cycle_cost: 1 },
    Instruction { name: "Swp"          , params: &[Param::Reg, Param::Reg], op_code: 0b0000_0011, cycle_cost: 1 },
    Instruction { name: "LdiR"         , params: &[Param::Addr16, Param::Const16], op_code: 0b0001_0000, cycle_cost: 1 },
    Instruction { name: "Sto"          , params: &[Param::Addr16, Param::Reg], op_code: 0b0001_0001, cycle_cost: 1 },
    Instruction { name: "Get"          , params: &[Param::Addr16, Param::Reg], op_code: 0b0001_0010, cycle_cost: 1 },
    Instruction { name: "LdiPtr"       , params: &[Param::Ptr, Param::Const16], op_code: 0b0001_0011, cycle_cost: 2 },
    Instruction { name: "StoPtr"       , params: &[Param::Ptr, Param::Reg], op_code: 0b0001_0100, cycle_cost: 2 },
    Instruction { name: "GetPtr"       , params: &[Param::Ptr, Param::Reg], op_code: 0b0001_0101, cycle_cost: 2 },
    Instruction { name: "MemCpy"       , params: &[Param::Addr16, Param::Addr32, Param::Const8], op_code: 0b0001_0110, cycle_cost: 5 },
    Instruction { name: "MemCpyPtr"    , params: &[Param::Ptr, Param::Ptr, Param::Reg], op_code: 0b0001_0111, cycle_cost: 6 },
    Instruction { name: "MemCmp"       , params: &[Param::Addr16, Param::Addr32, Param::Const8], op_code: 0b0001_1000, cycle_cost: 3 },
    Instruction { name: "MemCmpPtr"    , params: &[Param::Ptr, Param::Ptr, Param::Reg], op_code: 0b0001_1001, cycle_cost: 4 },
    Instruction { name: "StoPtrOff"    , params: &[Param::Ptr, Param::Const16, Param::Reg], op_code: 0b0001_1010, cycle_cost: 3 },
    Instruction { name: "GetPtrOff"    , params: &[Param::Ptr, Param::Const16, Param::Reg], op_code: 0b0001_1011, cycle_cost: 3 },
    Instruction { name: "MemFill"      , params: &[Param::Ptr, Param::Const16, Param::Const16], op_code: 0b0001_1100, cycle_cost: 5 },
    Instruction { name: "StoPtrOffPtr" , params: &[Param::Ptr, Param::Reg, Param::Reg], op_code: 0b0001_1101, cycle_cost: 3 },
    Instruction { name: "GetPtrOffPtr" , params: &[Param::Ptr, Param::Reg, Param::Reg], op_code: 0b0001_1110, cycle_cost: 3 },
    Instruction { name: "MovR"         , params: &[Param::Addr16, Param::Addr16], op_code: 0b0001_1111, cycle_cost: 2 },
    Instruction { name: "SetRamFrame"  , params: &[Param::Reg], op_code: 0b0010_0000, cycle_cost: 1 },
    Instruction { name: "SetStackFrame", params: &[Param::Reg], op_code: 0b0010_0001, cycle_cost: 1 },
    Instruction { name: "SaveRegisters", params: &[Param::Reg], op_code: 0b0010_0010, cycle_cost: 3 },
    Instruction { name: "LodRegisters" , params: &[Param::Reg], op_code: 0b0010_0011, cycle_cost: 3 },
    Instruction { name: "SetTimeout"   , params: &[Param::Reg], op_code: 0b0010_0100, cycle_cost: 1 },
    Instruction { name: "SetTimeoutAdd", params: &[Param::Addr16], op_code: 0b0010_0101, cycle_cost: 1 },
    Instruction { name: "SetIntAddr"   , params: &[Param::Addr16], op_code: 0b0010_0110, cycle_cost: 1 },
    Instruction { name: "Int"          , params: &[], op_code: 0b0010_0111, cycle_cost: 1 },
    Instruction { name: "CallPgrm"     , params: &[Param::Reg], op_code: 0b0010_1000, cycle_cost: 1 },
    Instruction { name: "RetInt"       , params: &[], op_code: 0b0010_1001, cycle_cost: 1 },
    Instruction { name: "SetPgrmStart" , params: &[Param::Reg], op_code: 0b0010_1010, cycle_cost: 1 },
    Instruction { name: "SetRamSize"   , params: &[Param::Reg], op_code: 0b0010_1011, cycle_cost: 1 },
    Instruction { name: "SetPgrmSize"  , params: &[Param::Reg], op_code: 0b0010_1100, cycle_cost: 1 },
    Instruction { name: "SetStackSize" , params: &[Param::Reg], op_code: 0b0010_1101, cycle_cost: 1 },
    Instruction { name: "SetFaultAddr" , params: &[Param::Addr16], op_code: 0b0010_1110, cycle_cost: 1 },
    Instruction { name: "Kill"         , params: &[], op_code: 0b0010_1111, cycle_cost: 1 },
    Instruction { name: "CpyRegion"    , params: &[Param::Addr16, Param::Addr16, Param::Const8], op_code: 0b0011_0000, cycle_cost: 4 },
    Instruction { name: "Plot"         , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0011_0001, cycle_cost: 2 },
    Instruction { name: "CpyRegionPtr" , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0011_0010, cycle_cost: 5 },
    Instruction { name: "VBlank"       , params: &[], op_code: 0b0011_0011, cycle_cost: 2 },
    Instruction { name: "SwapFrameBuf" , params: &[], op_code: 0b0011_0100, cycle_cost: 2 },
    Instruction { name: "ColorAt"      , params: &[Param::Const16, Param::Const16, Param::Reg], op_code: 0b0011_0101, cycle_cost: 2 },
    Instruction { name: "ColorPtr"     , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0011_0110, cycle_cost: 3 },
    Instruction { name: "Place"        , params: &[Param::Reg, Param::Reg, Param::Reg, Param::Reg, Param::Reg], op_code: 0b0011_0111, cycle_cost: 4 },
    Instruction { name: "CpyShown"     , params: &[], op_code: 0b0011_1000, cycle_cost: 6 },
    Instruction { name: "Solid"        , params: &[Param::Reg, Param::Reg, Param::Reg, Param::Reg, Param::Reg], op_code: 0b0011_1001, cycle_cost: 5 },
    Instruction { name: "Add"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0000, cycle_cost: 1 },
    Instruction { name: "Sub"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0001, cycle_cost: 1 },
    Instruction { name: "SubRev"       , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0010, cycle_cost: 1 },
    Instruction { name: "Mul"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0011, cycle_cost: 1 },
    Instruction { name: "Div"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0100, cycle_cost: 1 },
    Instruction { name: "Mod"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0101, cycle_cost: 1 },
    Instruction { name: "And"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0110, cycle_cost: 1 },
    Instruction { name: "Or"           , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_0111, cycle_cost: 1 },
    Instruction { name: "Not"          , params: &[Param::Reg, Param::Reg], op_code: 0b0100_1000, cycle_cost: 1 },
    Instruction { name: "Xor"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1001, cycle_cost: 1 },
    Instruction { name: "Pow"          , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1010, cycle_cost: 1 },
    Instruction { name: "Left"         , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1011, cycle_cost: 1 },
    Instruction { name: "Right"        , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1100, cycle_cost: 1 },
    Instruction { name: "RotLeft"      , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1101, cycle_cost: 1 },
    Instruction { name: "RotRight"     , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b0100_1110, cycle_cost: 1 },
    Instruction { name: "AddImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0000, cycle_cost: 1 },
    Instruction { name: "SubImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0001, cycle_cost: 1 },
    Instruction { name: "SubRevImm"    , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0010, cycle_cost: 1 },
    Instruction { name: "MulImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0011, cycle_cost: 1 },
    Instruction { name: "DivImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0100, cycle_cost: 1 },
    Instruction { name: "ModImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0101, cycle_cost: 1 },
    Instruction { name: "AndImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0110, cycle_cost: 1 },
    Instruction { name: "OrImm"        , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_0111, cycle_cost: 1 },
    Instruction { name: "XorImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1001, cycle_cost: 1 },
    Instruction { name: "PowImm"       , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1010, cycle_cost: 1 },
    Instruction { name: "LeftImm"      , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1011, cycle_cost: 1 },
    Instruction { name: "RightImm"     , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1100, cycle_cost: 1 },
    Instruction { name: "RotLeftImm"   , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1101, cycle_cost: 1 },
    Instruction { name: "RotRightImm"  , params: &[Param::Reg, Param::Const16, Param::Reg], op_code: 0b0101_1110, cycle_cost: 1 },
    Instruction { name: "Less"         , params: &[Param::Reg, Param::Reg], op_code: 0b0110_0000, cycle_cost: 1 },
    Instruction { name: "Grtr"         , params: &[Param::Reg, Param::Reg], op_code: 0b0110_0001, cycle_cost: 1 },
    Instruction { name: "Eq"           , params: &[Param::Reg, Param::Reg], op_code: 0b0110_0010, cycle_cost: 1 },
    Instruction { name: "LessImm"      , params: &[Param::Reg, Param::Const16], op_code: 0b0110_0011, cycle_cost: 1 },
    Instruction { name: "GrtrImm"      , params: &[Param::Reg, Param::Const16], op_code: 0b0110_0100, cycle_cost: 1 },
    Instruction { name: "EqImm"        , params: &[Param::Reg, Param::Const16], op_code: 0b0110_0101, cycle_cost: 1 },
    Instruction { name: "ClrFlags"     , params: &[], op_code: 0b0110_0110, cycle_cost: 1 },
    Instruction { name: "SaveFlags"    , params: &[Param::Reg], op_code: 0b0110_0111, cycle_cost: 1 },
    Instruction { name: "Zero"         , params: &[Param::Reg], op_code: 0b0110_1000, cycle_cost: 1 },
    Instruction { name: "loadFlags"    , params: &[Param::Reg], op_code: 0b0110_1001, cycle_cost: 1 },
    Instruction { name: "Jmp"          , params: &[Param::Addr16], op_code: 0b0111_0000, cycle_cost: 2 },
    Instruction { name: "Jic"          , params: &[Param::Addr16], op_code: 0b0111_0001, cycle_cost: 2 },
    Instruction { name: "Jnc"          , params: &[Param::Addr16], op_code: 0b0111_0010, cycle_cost: 2 },
    Instruction { name: "Jiz"          , params: &[Param::Addr16], op_code: 0b0111_0011, cycle_cost: 2 },
    Instruction { name: "Jnz"          , params: &[Param::Addr16], op_code: 0b0111_0100, cycle_cost: 2 },
    Instruction { name: "JiErr"        , params: &[Param::Addr16], op_code: 0b0111_0101, cycle_cost: 2 },
    Instruction { name: "JnErr"        , params: &[Param::Addr16], op_code: 0b0111_0110, cycle_cost: 2 },
    Instruction { name: "JiCry"        , params: &[Param::Addr16], op_code: 0b0111_0111, cycle_cost: 2 },
    Instruction { name: "JnCry"        , params: &[Param::Addr16], op_code: 0b0111_1000, cycle_cost: 2 },
    Instruction { name: "JmpPtr"       , params: &[Param::Reg], op_code: 0b0111_1001, cycle_cost: 2 },
    Instruction { name: "JicPtr"       , params: &[Param::Reg], op_code: 0b0111_1010, cycle_cost: 2 },
    Instruction { name: "JncPtr"       , params: &[Param::Reg], op_code: 0b0111_1011, cycle_cost: 2 },
    Instruction { name: "JizPtr"       , params: &[Param::Reg], op_code: 0b0111_1100, cycle_cost: 2 },
    Instruction { name: "JnzPtr"       , params: &[Param::Reg], op_code: 0b0111_1101, cycle_cost: 2 },
    Instruction { name: "JiCryPtr"     , params: &[Param::Reg], op_code: 0b0111_1110, cycle_cost: 2 },
    Instruction { name: "JnCryPtr"     , params: &[Param::Reg], op_code: 0b0111_1111, cycle_cost: 2 },
    Instruction { name: "Psh"          , params: &[Param::Reg], op_code: 0b1000_0000, cycle_cost: 2 },
    Instruction { name: "PshCon"       , params: &[Param::Const16], op_code: 0b1000_0001, cycle_cost: 1 },
    Instruction { name: "Pop"          , params: &[Param::Reg], op_code: 0b1000_0010, cycle_cost: 2 },
    Instruction { name: "Index"        , params: &[Param::Addr16, Param::Reg], op_code: 0b1000_0011, cycle_cost: 2 },
    Instruction { name: "Edit"         , params: &[Param::Addr16, Param::Reg], op_code: 0b1000_0100, cycle_cost: 2 },
    Instruction { name: "Call"         , params: &[Param::Addr16], op_code: 0b1000_0101, cycle_cost: 3 },
    Instruction { name: "Ret"          , params: &[], op_code: 0b1000_0110, cycle_cost: 3 },
    Instruction { name: "RetFramed"    , params: &[Param::Reg], op_code: 0b1000_0111, cycle_cost: 3 },
    Instruction { name: "SetStackPtr"  , params: &[Param::Reg], op_code: 0b1000_1000, cycle_cost: 3 },
    Instruction { name: "RetConst"     , params: &[Param::Const16], op_code: 0b1000_1001, cycle_cost: 3 },
    Instruction { name: "IndexPtr"     , params: &[Param::Reg, Param::Reg], op_code: 0b1000_1010, cycle_cost: 3 },
    Instruction { name: "IndexOff"     , params: &[Param::Reg, Param::Reg], op_code: 0b1000_1011, cycle_cost: 3 },
    Instruction { name: "IndexOffConst", params: &[Param::Const16, Param::Reg], op_code: 0b1000_1100, cycle_cost: 3 },
    Instruction { name: "EditPtr"      , params: &[Param::Reg, Param::Reg], op_code: 0b1000_1101, cycle_cost: 3 },
    Instruction { name: "PshPtr"       , params: &[Param::Ptr], op_code: 0b1000_1110, cycle_cost: 3 },
    Instruction { name: "Write"        , params: &[Param::Addr32, Param::Reg], op_code: 0b1001_0000, cycle_cost: 3 },
    Instruction { name: "Load"         , params: &[Param::Addr32, Param::Reg], op_code: 0b1001_0001, cycle_cost: 2 },
    Instruction { name: "WritePtr"     , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b1001_0010, cycle_cost: 3 },
    Instruction { name: "LoadPtr"      , params: &[Param::Reg, Param::Reg, Param::Reg], op_code: 0b1001_0011, cycle_cost: 2 },
    Instruction { name: "WriteSeg"     , params: &[Param::Reg, Param::Reg, Param::Reg, Param::Reg], op_code: 0b1001_0100, cycle_cost: 8 },
    Instruction { name: "LoadSeg"      , params: &[Param::Reg, Param::Reg, Param::Reg, Param::Reg], op_code: 0b1001_0101, cycle_cost: 8 },
    Instruction { name: "readIn"       , params: &[Param::Reg, Param::Const8], op_code: 0b1010_0000, cycle_cost: 1 },
    Instruction { name: "readInFlag"   , params: &[Param::Const8], op_code: 0b1010_0001, cycle_cost: 1 },
    Instruction { name: "writeOut"     , params: &[Param::Reg, Param::Const8], op_code: 0b1010_0010, cycle_cost: 1 },
    Instruction { name: "writeOutFlag" , params: &[Param::Reg, Param::Const8], op_code: 0b1010_0011, cycle_cost: 1 },
];

fn split_line(line: &mut Vec<&str>) {
    let mut i = 0;
    let mut char_index = 0;
    loop {
        if char_index >= line[i].len() {
            char_index = 0;
            i += 1;
            if i >= line.len() { break; }
        }
        
        if &line[i][char_index..next_valid_index(char_index + 1, line[i])] == "¡" {
            let next = next_valid_index(char_index + 1, line[i]);
            let text_seg = line.remove(i);
            line.insert(i, &text_seg[next..]);  // the end bit
            line.insert(i, &text_seg[char_index..next]);  // the end bit
            line.insert(i, &text_seg[..char_index]);  // the start
            i += 2;  // the previous token is completed
            char_index = 0;  // restart the check for this new token
        }
        else if BREAKS.contains(&&line[i][0..char_index]) && !part_of_large_token(line[i], last_valid_index(char_index.saturating_sub(1), line[i])) {
            let text_seg = line.remove(i);
            line.insert(i, &text_seg[char_index..]);
            line.insert(i, &text_seg[..char_index]);
            i += 1;  // the previous token is completed
            char_index = 0;  // restart the check for this new token
            continue;
        }
        else if BREAKS.contains(&&line[i][char_index..next_valid_index(char_index+1, line[i])]) {
            // making sure it's not part of a larger token, if so breaking it up
            if part_of_large_token(&line[i], last_valid_index(char_index.saturating_sub(1), line[i])) {
                if char_index == 0 || BREAKS.contains(&&line[i][0..char_index]) {
                    char_index += next_valid_index(char_index + 1, line[i]) - char_index;
                    continue;
                }
                let text_seg = line.remove(i);
                // adding the previous bit allowing the token to be looked at in its entirety
                line.insert(i, &text_seg[char_index..]);
                line.insert(i, &text_seg[..char_index]);
                i += 1;  // the previous token is completed
                char_index = 0;  // restart the check for this new token
                continue;
            }
            
            if part_of_large_token(&line[i][char_index..], 0) {
                let text_seg = line.remove(i);
                // adding the previous bit allowing the token to be looked at in its entirety
                line.insert(i, &text_seg[char_index..]);
                line.insert(i, &text_seg[..char_index]);
                i += 1;  // the previous token is completed
                char_index = 0;  // restart the check for this new token
                continue;
            }
            
            if char_index == 0 {
                let text_seg = line.remove(i);
                line.insert(i, &text_seg[1..]);
                line.insert(i, &text_seg[0..1]);  // the token
                i += 1;
                char_index = 0;
                continue;
            }
            
            let text_seg = line.remove(i);
            line.insert(i, &text_seg[char_index+1..]);  // the end bit
            line.insert(i, &text_seg[char_index..char_index+1]);  // the end bit
            line.insert(i, &text_seg[..char_index]);  // the start
            i += 2;  // the previous token is completed
            char_index = 0;  // restart the check for this new token
        }
        char_index += next_valid_index(char_index + 1, line[i]) - char_index;//line[i][].bytes().count();
    }
}

fn next_valid_index(mut index: usize, text: &str) -> usize {
    while index < text.len() && !text.is_char_boundary(index) {
        index += 1;
    } index
}

fn last_valid_index(index: usize, text: &str) -> usize {
    let mut index = index;
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    } index
}

pub fn part_of_large_token(text: &str, char_index: usize) -> bool {
    if text.len() == char_index + 1 {
        if text.len() == 1 { return false; }
        for breaker in BREAKS {
            if breaker.len() == 1 { continue; }
            if breaker.contains(&text) {
                return true;
            }
        } return false;
    }
    if BREAKS.contains(&&text[0..next_valid_index(1, text)]) {
        for breaker in BREAKS {
            if breaker.len() == 1 || breaker.len() - 1 <= char_index { continue; }
            if breaker.contains(&&text[0..next_valid_index(usize::min(breaker.len(), text.len()), &text)]) {
                return true;
            } } }
    for breaker in BREAKS {
        if breaker.len() == 1 { continue; }
        if breaker.contains(&&text[char_index..]) {
            return true;
        } } false
}

static BLANK_CHARS: &[&str] = &[
    " ", ",",
];

static BREAKS: &[&str] = &[
    " ", "\n", "\t", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "=", "+",
    "[", "]", "{", "}", ";", ":", "'", "\"", ",", "<", ".", ">", "/", "?", "\\", "|",
    "//",
];

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Parameter {
    Register (u8),
    Constant (u16),
    Address (u32),
    Pointer (u8),
}

static REGISTERS: &'static [&'static str] = &[
    "rda",
    "rdb",
    "rdc",
    "rdd",
    "rde",
    "rdf",
    "rdg",
    "rdh",
    "rdi",
    "rdj",
    "rdk",
    "rdl",
    "rdm",
    "rdn",
    "rdo",
    "rdp",
    "rdq",
    "rdr",
    "rds",
    "rdt",
    "rdu",
    "rdv",
    "rdw",
    "rdx",
    "rdy",
    "rdz",
    "acc",
];

fn resolve_number(number: &str, labels: Option<&Vec<Label>>) -> u32 {
    if let Some(label) = labels.unwrap_or(&vec![]).iter().find(|l| match l {
        Label::Trait(s,..) | Label::Const(s,..) | Label::Variable(s,..) | Label::Header(s,..) => {
            s == number
        },
        Label::Alloc(..) => false,
    }) {
        match label {
            Label::Header(..,l) => {
                return *l as u32;
            },
            Label::Const(..,l) | Label::Variable(..,l) => {
                return *l as u32
            },
            _ => {}
        }
    }
    if number.len() <= 2 { return number.parse::<u32>().unwrap(); }
    match &number[..2] {
        "0b" => { u32::from_str_radix(&number[2..], 2).unwrap() },
        "0x" => { u32::from_str_radix(&number[2..], 16).unwrap() },
        _ => { number.parse::<u32>().unwrap() },
    }
}

fn handle_instruction(errors: &mut Vec<String>, cont: &mut bool, line: &Vec<&str>, native_line_number: usize, parameters: Vec<Parameter>, instructions: &mut Vec<Union<(Instruction, Vec<Parameter>, usize), Label>>) {
    // parsing normal
    match INSTRUCTIONS.iter().position(|inst| inst.name.to_uppercase() == line[1].to_uppercase()) {
        Some(index) => {
            if INSTRUCTIONS[index].params.len() != parameters.len() {
                errors.push(format!(
                    "Invalid number of operands: found '{}' when '{}' were expected. Line: '{}'",
                    parameters.len(), INSTRUCTIONS[index].params.len(), native_line_number)
                );
                *cont = true;
            }
            for (i, param) in INSTRUCTIONS[index].params.iter().enumerate() {
                match_valid(errors, param, &parameters, i, native_line_number);
            }
            instructions.push(Union::A((INSTRUCTIONS[index].clone(), parameters, native_line_number)));
            *cont = true;
        },
        None => {},
    }
}

fn match_valid(errors: &mut Vec<String>, param: &Param, parameters: &Vec<Parameter>, i: usize, native_line_number: usize) -> bool {
    match param {
        Param::Const8 | Param::Const16 => {
            if !matches!(parameters[i], Parameter::Constant(_)) {
                errors.push(format!(
                    "Invalid operand type; found type {:?}, but expected a Constant. Line: '{}'",
                    parameters[i], native_line_number
                ));
                true
            } else { false }
        },
        Param::Addr16 | Param::Addr32 => {
            if !matches!(parameters[i], Parameter::Address(_)) {
                errors.push(format!(
                    "Invalid operand type; found type {:?}, but expected an Address. Line: '{}'",
                    parameters[i], native_line_number
                ));
                true
            } else { false }
        },
        Param::Reg => {
            if !matches!(parameters[i], Parameter::Register(_)) {
                errors.push(format!(
                    "Invalid operand type; found type {:?}, but expected a Register. Line: '{}'",
                    parameters[i], native_line_number
                ));
                true
            } else { false }
        },
        Param::Ptr => {
            if !matches!(parameters[i], Parameter::Pointer(_)) {
                errors.push(format!(
                    "Invalid operand type; found type {:?}, but expected a Pointer. Line: '{}'",
                    parameters[i], native_line_number
                ));
                true
            } else { false }
        },
    }
}

#[derive(Debug, Clone)]
enum Label {
    Header (String, usize),  // name, program addr
    Const (String, u16),  // name, value
    Variable (String, u16),  // name, addr
    Alloc (usize, Vec<u16>),  // ram addr, byte pairs
    Trait (String, u16),  // trait name, byte pair
    // macro expansion already happened before this
}

fn parse_macros_and_allocs(script: &mut Vec<(Vec<&str>, usize)>) {
    // todo!   collect and extract macros, than expand (the src line numbers should be the same as the extracted macro, so no changes are needed there)    for arg replacement, just look for <...> to find operands being used
    // todo! also expand any !alloc's (might have to be done later???? idk, &str's are just hard to work with in this context
}

fn parse_sudo(mut script: Vec<(Vec<&str>, usize)>) -> Result<(Vec<Union<(Instruction, Vec<Parameter>, usize), Label>>, Vec<Label>), Vec<String>> {
    let mut errors = vec![];
    parse_macros_and_allocs(&mut script);
    // generating the names and values of labels first
    let mut pg_line_number = 3;  // the first three byte pairs are reserved for the os header
    let mut labels: Vec<Label> = vec![];
    for (line, native_line_number) in &script {
        if line.is_empty() { continue; }  // shouldn't happen
        match line[0] {
            "!" => {
                if line.len() < 2 { errors.push(format!("Invalid !Label, no specified ending/name present; line: '{}'", native_line_number + 1)); }
                match line[1] {
                    "function" | "header" | "loop" | "end" | "condition" | "true" | "false" | "if" | "else" | "label" => {
                        if line.len() < 3 { errors.push(format!("No header name found for label declared on line: '{}'", native_line_number + 1)); }
                        if labels.iter().any(|l| match l {
                            Label::Trait(s,..) | Label::Const(s,..) | Label::Variable(s,..) | Label::Header(s,..) => {
                                s == line[2]
                            },
                            Label::Alloc(..) => false,
                        }) { errors.push(format!("Redefintion of label '{}' on line: '{}'", line[2], native_line_number + 1)); }
                        labels.push(Label::Header(line[2].to_string(), pg_line_number));
                    },
                    "define" => {
                        if line.len() < 4 { errors.push(format!("No operands and/or name found for label declared on line: '{}'", native_line_number + 1)); }
                        if labels.iter().any(|l| match l {
                            Label::Trait(s,..) | Label::Const(s,..) | Label::Variable(s,..) | Label::Header(s,..) => {
                                s == line[2]
                            },
                            Label::Alloc(..) => false,
                        }) { errors.push(format!("Redefintion of label '{}' on line: '{}'", line[2], native_line_number + 1)); }
                        labels.push(Label::Variable(line[2].to_string(), resolve_number(line[3], None) as u16));
                    },
                    "const" => {
                        if line.len() < 4 { errors.push(format!("No operands and/or name found for label declared on line: '{}'", native_line_number + 1)); }
                        if labels.iter().any(|l| match l {
                            Label::Trait(s,..) | Label::Const(s,..) | Label::Variable(s,..) | Label::Header(s,..) => {
                                s == line[2]
                            },
                            Label::Alloc(..) => false,
                        }) { errors.push(format!("Redefintion of label '{}' on line: '{}'", line[2], native_line_number + 1)); }
                        labels.push(Label::Const(line[2].to_string(), resolve_number(line[3], None) as u16));
                    },
                    // macros & allocs have been parsed out already
                    _ => {
                        errors.push(format!("Invalid label name: '{}' on line: '{}'", line[1], native_line_number + 1));
                    },
                }
            },
            "." => {
                if line.len() < 3 { errors.push(format!("Invalid .trait, no specified ending/name present, and no operands found; line: '{}'", native_line_number + 1)); }
                match line[1] {
                    "ram_size" | "name" | "program_size" | "page" => {
                        labels.push(Label::Trait(line[1].to_string(), resolve_number(line[2], None) as u16));
                        if line[1] == "page" {
                            pg_line_number = resolve_number(line[2], None) as usize;
                        }
                    },
                    _ => {
                        errors.push(format!("Invalid trait name: '{}' on line: '{}'", line[1], native_line_number + 1));
                    }
                }
            }
            _ => { pg_line_number += 3; },  // not a blank line, and not a special label line
        }
    }
    println!("Generated labels: {:?}", labels);
    if !errors.is_empty() { return Err(errors); }
    
    let mut instructions = vec![];
    // unless the instruction starts with *, first check sudo, than normal; with a * just check normal instructions
    for (mut line, native_line_number) in script {
        // identifying parameters
        let mut parameters = vec![];
        let mut index = 0;
        match line[0] {
            "!" | "." => {
                if line.len() < 3 { errors.push(format!("Invalid label declared on line: '{}'", native_line_number + 1)); }
                let offset = if line[0] == "." { 1 } else { 2 };
                instructions.push(Union::B(
                    labels.iter().find(|l| match l {
                        Label::Trait(s,..) | Label::Const(s,..) | Label::Variable(s,..) | Label::Header(s,..) => {
                            s == line[offset]
                        },
                        Label::Alloc(..) => false,
                    }).unwrap().clone()
                ));
                continue;
            },
            "*" => { index += 1; },
            _ => {}
        }
        
        while line.len() >= 2 && index < line.len() - 2 {
            index += 1;
            
            match line[index] {
                "[" => {
                    // either: '[' '%' 'reg' ']'   or '[' '%' 'reg' '+' '$' 'num' ']'
                    if line[index + 3] == "]" {
                        // just a pointer to the register mentioned
                        match REGISTERS.iter().position(|reg| reg.to_uppercase() == line[index + 2].to_uppercase()) {
                            Some(index) => parameters.push(Parameter::Pointer(index as u8)),
                            None => {
                                errors.push(
                                    format!("Invalid register name given for pointer, '{}', on line {}, token number '{}'",
                                            line[index + 2], native_line_number + 1, index + 2)
                                );
                            }
                        };
                        index += 3;
                    } else {
                        // needs to become an offset instruction (should be auto done if setup right)
                        // push an extra parameter onto it (should be as easy as that I think?)
                        match REGISTERS.iter().position(|reg| reg.to_uppercase() == line[index + 2].to_uppercase()) {
                            Some(index) => parameters.push(Parameter::Pointer(index as u8)),
                            None => {
                                errors.push(
                                    format!("Invalid register name given for pointer, '{}', on line {}, token number '{}'",
                                            line[index + 2], native_line_number + 1, index + 2)
                                );
                            }
                        };
                        parameters.push(Parameter::Constant(resolve_number(line[index + 5], Some(&labels)) as u16));
                        index += 6;
                    }
                },
                "{" => {
                    // const stuff, expr needs resolution todo!    should this be done before? or will pointers not using constants? but they could subtract a constant that had arithmatic? idk
                    index += 1;
                },
                "#" => {
                    // address
                    parameters.push(Parameter::Address(resolve_number(line[index + 1], Some(&labels))));
                    index += 1;
                },
                "$" | "@" => {
                    // constant of some sort
                    parameters.push(Parameter::Constant(resolve_number(line[index + 1], Some(&labels)) as u16));
                    index += 1;
                },
                "%" => {
                    // register of sorts
                    match REGISTERS.iter().position(|reg| reg.to_uppercase() == line[index + 1].to_uppercase()) {
                        Some(index) => parameters.push(Parameter::Register(index as u8)),
                        None => {
                            errors.push(
                                format!("Invalid register name, '{}', on line {}, token number '{}'",
                                line[index + 1], native_line_number + 1, index + 1)
                            );
                        }
                    };
                    index += 1;
                },
                _ => {
                    errors.push(
                        format!("Invalid operand type, '{}', on line {}, token number '{}'",
                                line[index], native_line_number + 1, index + 1)
                    );
                }
            }
        }
        
        if line[0] == "*" {
            let mut cont = false;
            handle_instruction(&mut errors, &mut cont, &line, native_line_number, parameters, &mut instructions);
            if cont { continue; }
        } else {
            // parsing sudo
            match SUDO_INSTRUCTIONS.iter().position(|inst| inst.name.to_uppercase() == line[0].to_uppercase()) {
                Some(index) => {
                    let sudo = &SUDO_INSTRUCTIONS[index];
                    // finding the best match for the instruction
                    match sudo.conversions.iter().position(|(_name, params, _replacement)| params.len() == parameters.len() && !params.iter().enumerate().any(|(i, p)| {
                        if i >= parameters.len() { return false; }
                        match_valid(&mut vec![], p, &parameters, i, native_line_number)
                    })) {
                        Some(index) => {
                            let (name, _params, _replacement) = sudo.conversions[index].clone();
                            let instruction = INSTRUCTIONS.iter().find(|inst| inst.name == name).unwrap();
                            instructions.push(Union::A((instruction.clone(), parameters, native_line_number)));
                            continue;
                        },
                        None => {},
                    }
                },
                None => {},
            }
        }
        errors.push(format!("Invalid Instruction on line '{}':\n> Tokens: {:?}", native_line_number + 1, line));
    }
    if !errors.is_empty() { return Err(errors); }
    Ok((instructions, labels))
}

#[derive(Debug, Clone)]
enum Union<A, B> {
    A(A),
    B(B),
}

fn search_for_trait (labels: &Vec<Label>, trait_name: &str, none_case: usize) -> usize {
    for label in labels {
        match label {
            Label::Trait(name, value) => {
                if name == trait_name { return *value as usize; }
            },
            _ => {}
        }
    } none_case
}

fn compile_script(instructions: Vec<Union<(Instruction, Vec<Parameter>, usize), Label>>, labels: Vec<Label>) -> Result<Vec<(usize, (u16, usize))>, String> {
    let mut bytes = vec![
        // the predefined header
        (0, (search_for_trait(&labels, "ram_size", 0) as u16, 0)),
        (0, (search_for_trait(&labels, "program_size", ({
            let mut i = 0;
            for inst in &instructions {
                match inst {
                    Union::A(_) => i += 1,
                    _ => {}
                }
            } i
        } + 1) * 3) as u16, 0)),
        (0, (search_for_trait(&labels, "name", 0) as u16, 0)),
    ];
    let mut raw_index = 0;
    for instruction in instructions {
        match instruction {
            Union::A((instruction, parameters, line_number)) => {
                let mut whole_bytes = vec![instruction.op_code];
                for i in 0..parameters.len() {
                    match &instruction.params[i] {
                        Param::Const16 => {
                            match parameters[i] {
                                Parameter::Constant(numeral) => {
                                    whole_bytes.push((numeral & 0xFF) as u8);  // low byte
                                    whole_bytes.push(((numeral & 0xFF00) >> 8) as u8);  // high byte
                                },
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            }
                        },
                        Param::Const8 => {
                            whole_bytes.push(match parameters[i] {
                                Parameter::Constant(numeral) => numeral as u8,
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            })
                        },
                        Param::Addr32 => {
                            match parameters[i] {
                                Parameter::Address(numeral) => {
                                    whole_bytes.push((numeral & 0xFF) as u8);  // low-low byte
                                    whole_bytes.push(((numeral & 0xFF00) >> 8) as u8);  // low high byte
                                    whole_bytes.push(((numeral & 0xFF0000) >> 16) as u8);  // high low byte
                                    whole_bytes.push(((numeral & 0xFF000000) >> 24) as u8);  // high-high byte
                                },
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            }
                        },
                        Param::Addr16 => {
                            match parameters[i] {
                                Parameter::Address(numeral) => {
                                    whole_bytes.push((numeral & 0xFF) as u8);  // low byte
                                    whole_bytes.push(((numeral & 0xFF00) >> 8) as u8);  // high byte
                                },
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            }
                        },
                        Param::Reg => {
                            whole_bytes.push(match parameters[i] {
                                Parameter::Register(numeral) => numeral,
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            })
                        },
                        Param::Ptr => {
                            whole_bytes.push(match parameters[i] {
                                Parameter::Pointer(numeral) => numeral,
                                _ => { return Err(format!("Invalid parameter: {:?}", parameters[i])); }
                            })
                        },
                    }
                }
                let mut count = 0;
                while !whole_bytes.is_empty() {
                    let low = whole_bytes.remove(0);
                    let high = if whole_bytes.is_empty() { 0 } else { whole_bytes.remove(0) };
                    bytes.push((raw_index, (((low as u16) << 8) | (high as u16), line_number)));
                    raw_index += 1;
                    count += 1;
                }
                while count < 3 {
                    bytes.push((raw_index, (0u16, line_number)));
                    raw_index += 1;
                    count += 1;
                }
            },
            Union::B(label) => {
                match label {
                    Label::Alloc(addr, byte_pairs) => {},
                    Label::Variable(name, addr) => {},
                    Label::Const(name, value) => {},
                    Label::Header(name, program_addr) => {},
                    Label::Trait(trait_name, byte_pair) => {
                        match &*trait_name {
                            "page" => {
                                raw_index = byte_pair as usize;
                            },
                            _ => {}
                        }
                    },
                }
            },
        }
    } Ok(bytes)
}

fn main() {
    //let script = std::fs::read_to_string("scripts/test.cisc").unwrap();
    let script = std::fs::read_to_string("scripts/boot.cisc").unwrap();
    let mut original_script = script
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<&str>>();
    original_script.iter_mut().for_each(|line| *line = line.split(";").collect::<Vec<&str>>()[0].trim());
    println!("{:?}", original_script);
    
    let mut script = script.lines().enumerate().map(|(index, e)| (vec![e], index)).collect::<Vec<(Vec<&str>, usize)>>();
    for line in script.iter_mut() {
        split_line(&mut line.0);
        line.0.retain(|e| !e.is_empty() && !BLANK_CHARS.contains(e));
        let mut commented = false;
        line.0.retain(|s| { if *s == ";" { commented = true } !commented })
    }
    script.retain(|e| !e.0.is_empty() && !e.0[0].is_empty());
    println!("Broken Tokens: \n{:?}", script);
    let (instructions, labels) = match parse_sudo(script) {
        Ok(instructions) => instructions,
        Err(errors) => {
            for error in errors {
                println!("{}", error);
            } return;
        }
    };
    println!("Instructions: {:?}", instructions);
    let bytes = match compile_script(instructions, labels) {
        Ok(bytes) => bytes,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };
    println!("Byte Pairs: {:0>4x?}", bytes);
    
    let mut emulation = Emulator::new(vec![0; u16::MAX as usize], vec![0; u16::MAX as usize]);
    for byte in bytes.into_iter() {
        emulation.disc[byte.0] = byte.1.0;  // writing in the bytes
        emulation.trace_disc[byte.0] = byte.1.1;
    }
    emulation.boot();
    emulation.run(original_script);
}

struct Emulator {
    v_ram: std::sync::Arc<parking_lot::RwLock<Vec<u16>>>,
    disc: Vec<u16>,
    ram: Vec<u16>,
    registers: Vec<u16>,
    stack: Vec<u16>,
    io_in_ports: std::sync::Arc<parking_lot::RwLock<Vec<(u16, bool)>>>,
    io_out_ports: Vec<(u16, bool)>,
    _display: std::thread::JoinHandle<()>,
    display_v_blank: crossbeam::channel::Receiver<()>,
    _kill_signal: crossbeam::channel::Sender<()>,
    kill_recv: crossbeam::channel::Receiver<()>,
    _io_handler: std::thread::JoinHandle<()>,
    trace_disc: Vec<usize>,  // the line number of the program
    trace_ram: Vec<usize>,  // the line number of the program
    frame_buffer_ptr: std::sync::Arc<parking_lot::RwLock<usize>>,
}

#[repr(u8)]
enum Register {
    ProgramCounter = 27u8,  // starting after 'acc' which comes after 'rda' - 'rdz'
    RamFrameStart = 28u8,
    StackFrameStart = 29u8,
    TimeoutDuration = 30u8,
    TimeOutCallbackAddr = 31u8,
    InterruptCallbackAddr = 32u8,
    ProgramStart = 33u8,
    ProgramSize = 34u8,
    RamSize = 35u8,
    StackSize = 36u8,
    Protected = 37u8,
    Cycles = 38u8,
    StackTopPtr = 39u8,
    FaultCallbackAddr = 40u8,
    InterruptedLine = 41u8,
    ConditionFlag = 42u8,
    ZeroFlag = 43u8,
    OverflowFlag = 44u8,
    FaultFlag = 45u8,
}

impl Emulator {
    pub fn new(
        trace_disc: Vec<usize>,
        trace_ram: Vec<usize>,
    ) -> Self {
        let v_ram = std::sync::Arc::new(parking_lot::RwLock::new(vec![0b11000_00101_11010_0u16; const {   480 * 320 * 2   }]));
        let frame_buffer_ptr = std::sync::Arc::new(parking_lot::RwLock::new(0));
        let frame_buffer_ptr_clone = frame_buffer_ptr.clone();
        let v_ram_clone = v_ram.clone();
        let (kill_signal, kill_receiver) = crossbeam::channel::bounded(0);
        let (sender, display_v_blank) = crossbeam::channel::bounded(0);
        let display = std::thread::spawn(move || {
            #[cfg(debug_assertions)]
            {
                println!("{}", "\n".repeat(50));
                //return;
            }
            print!("\x1b[?25l");
            let mut buf = std::io::BufWriter::new(std::io::stdout());
            println!("{}", "\n".repeat(336));
            while kill_receiver.try_recv().is_err() {
                let mut text = "\x1B[H".to_string();
                for y in 0..336 {
                    if y >= 300 {    // only 300 tall, as it couldn't fit 320 high
                        // blanking period of 16; 24 htz; 24 / (320 + 16) ~= 1/14 per row
                        //std::thread::sleep(std::time::Duration::from_secs_f64(const { 1.0 / 14.0 }));
                        continue;
                    }
                    for x in 0..504 {
                        if x >= 480 {
                            if x == 480 {  // only sending at the start of v_blank so there isn't a spam of this, keeping everything in sync
                                match sender.send(()) {  // the v_blank signal
                                    Ok(_) => {},
                                    Err(_) => { break; }  // probably the main thread closed
                                }
                            }
                            // blanking period of 480; 1/14 secs per row; (1/14) / (480 + 16) ~= 1/7056 per row
                            //std::thread::sleep(std::time::Duration::from_secs_f64(const { 1.0 / 7056.0 }));
                            continue;
                        }
                        let display_locked = v_ram_clone.read();
                        let r = ((display_locked[x + y * 480 + *frame_buffer_ptr_clone.read()] >> 10) & 0b11111) << 3;
                        let g = ((display_locked[x + y * 480 + *frame_buffer_ptr_clone.read()] >> 5 ) & 0b11111) << 3;
                        let b = ((display_locked[x + y * 480 + *frame_buffer_ptr_clone.read()] >> 0 ) & 0b11111) << 3;
                        text.push_str(&format!("\x1b[{};{}H\x1B[48;2;{};{};{}m   \x1B[0m", y + 1, x * 3 + 1, r, g, b));
                    }
                }
                writeln!(&mut buf, "{}", text).unwrap();
            }
            print!("\x1b[?25h");
        });
        let io_in_ports = std::sync::Arc::new(parking_lot::RwLock::new(vec![(0u16, false); 256]));
        let io_in_ports_clone = io_in_ports.clone();
        let (kill_send, kill_recv) = crossbeam::channel::bounded(0);
        let _io_handler = std::thread::spawn(move || {
            crossterm::terminal::enable_raw_mode().unwrap();
            let mut stdin = std::io::stdin();
            loop {
                let mut local_buffer = [0; 12];
                let result = stdin.read(&mut local_buffer);
                if let Ok(_n) = result {
                    if local_buffer[0] == 0x51 {  // safety release to prevent a runaway.....
                        // capital Q
                        crossterm::terminal::disable_raw_mode().unwrap();
                        kill_send.send(()).unwrap();
                        return;
                    }
                    // for this just writing to the first port
                    io_in_ports_clone.write()[0] = (local_buffer[0] as u16, true);
                }
            }
        });
        
        Self {
            // display: 480 x 320   * 2 (active and back buffers)    24 htz
            v_ram,
            disc     : vec![0u16; const { u32::MAX as usize }],
            ram      : vec![0u16; const { u16::MAX as usize }],
            registers: vec![0u16; const {  u8::MAX as usize }],
            stack    : vec![0u16; const { u16::MAX as usize }],
            io_in_ports,
            io_out_ports: vec![(0u16, false); 256],
            _display: display,
            display_v_blank,
            _kill_signal: kill_signal,
            kill_recv,
            _io_handler,
            trace_disc,
            trace_ram,
            frame_buffer_ptr,
        }
    }
    
    pub fn boot(&mut self) {
        // reading in the first 256 byte pairs (512 bytes) into ram to begin the bootloader
        for booter_index in 0..256 {
            self.ram[booter_index] = self.disc[booter_index];
            self.trace_ram[booter_index] = self.trace_disc[booter_index];
        }
        // the program counter is being set to start just past the initial header
        //    (which would get ignored from booting; only necessary for os operated files, but still is present in all files)
        self.registers[const { Register::ProgramCounter as usize }] = 3;
        self.registers[const { Register::Protected as usize }] = 1;  // entering protected mode for booting
    }
    
    #[inline(always)]
    fn get_protected_ram_offset(registers: &Vec<u16>) -> u16 {
        (1 - registers[const { Register::Protected as usize }]) * registers[const { Register::RamFrameStart as usize }]
    }
    
    #[inline(always)]
    fn get_protected_stack_offset(registers: &Vec<u16>) -> u16 {
        (1 - registers[const { Register::Protected as usize }]) * registers[const { Register::StackFrameStart as usize }]
    }
    
    #[inline(always)]
    fn get_protected_pgc_offset(registers: &Vec<u16>) -> u16 {
        (1 - registers[const { Register::Protected as usize }]) * registers[const { Register::ProgramStart as usize }]
    }
    
    pub fn run(&mut self, original_code: Vec<&str>) {
        let mut instruction_cycle_cost = [1u16; 256];
        for instruction in INSTRUCTIONS {
            instruction_cycle_cost[instruction.op_code as usize] = instruction.cycle_cost as u16;
        }
        
        let mut stack_trace: Vec<usize> = vec![];
        let mut held_cycle_count = 0;  // the last recorded cycle count for timeout purposes
        
        let mut edited_registers: Option<Vec<usize>> = None;
        let mut read_registers: Option<Vec<usize>> = None;
        let mut edited_ram: Option<Vec<usize>> = None;
        let mut read_ram: Option<Vec<usize>> = None;
        
        let emulation_start = std::time::Instant::now();
        let mut iterations = 0;
        'em_loop: loop {
            #[cfg(debug_assertions)]
            {
                if self.kill_recv.try_recv().is_ok() { break; }
                edited_registers = None;
                read_registers = None;
                edited_ram = None;
                read_ram = None;
            }
            iterations += 1;
            let pgc = self.registers[const { Register::ProgramCounter as usize }];
            let mut next_line = pgc + 3;
            let bytes = [
                self.ram[pgc as usize],
                self.ram[pgc as usize + 1],
                self.ram[pgc as usize + 2],
            ];
            
            let op_code = bytes[0] >> 8;
            self.registers[const { Register::Cycles as usize }] += instruction_cycle_cost[op_code as usize];
            
            match op_code as u8 {
                0b0000_0000 => {},  // Nop
                0b0000_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[0] & 0xFF) as usize] = (bytes[1] >> 8) | (bytes[1] << 8);
                },  // Ldi
                0b0000_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize];
                },  // Mov
                0b0000_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                    }
                    (self.registers[(bytes[1] >> 8) as usize], self.registers[(bytes[0] & 0xFF) as usize]) = (self.registers[(bytes[0] & 0xFF) as usize], self.registers[(bytes[1] >> 8) as usize])
                },  // Swp
                0b0001_0000 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![(((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) + Self::get_protected_ram_offset(&self.registers)) as usize]);
                    }
                    let index = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00));
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = (bytes[1] & 0xFF) | (bytes[2] & 0xFF00);
                },  // LdiR
                0b0001_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![(((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        read_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00));
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = self.registers[(bytes[1] & 0xFF) as usize];
                },  // Sto
                0b0001_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![(((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = ((bytes[1] & 0xFF) | (bytes[1] & 0xFF00));
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[0] & 0xFF) as usize] = self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize];
                },  // Get
                0b0001_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = (bytes[1] >> 8) | (bytes[1] << 8);
                },  // LdiPtr
                0b0001_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        read_registers = Some(vec![(bytes[1] >> 8) as usize, (bytes[0] & 0xFF) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = self.registers[(bytes[1] >> 8) as usize];
                },  // StoPtr
                0b0001_0101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[1] >> 8) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] >> 8) as usize] = self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize];
                },  // GetPtr
                0b0001_0110 => {
                    let offset = Self::get_protected_ram_offset(&self.registers) as usize;
                    let src_addr = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) as usize + offset;
                    let dest_addr = ((bytes[1] & 0xFF) | (bytes[2] & 0xFF00)) as usize + offset;
                    let slice_size = (bytes[2] & 0xFF) as usize;
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some((dest_addr..dest_addr + slice_size).collect());
                        read_ram = Some((src_addr..src_addr + slice_size).collect());
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 && (src_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize || dest_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize) {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram.copy_within(src_addr..src_addr + slice_size, dest_addr);
                },  // MemCpy
                0b0001_0111 => {
                    let offset = Self::get_protected_ram_offset(&self.registers) as usize;
                    let src_addr = self.registers[(bytes[0] & 0xFF) as usize] as usize + offset;
                    let dest_addr = self.registers[(bytes[1] >> 8) as usize] as usize + offset;
                    let slice_size = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some((dest_addr..dest_addr + slice_size).collect());
                        read_ram = Some((src_addr..src_addr + slice_size).collect());
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 && (src_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize || dest_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize) {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram.copy_within(src_addr..src_addr + slice_size, dest_addr);
                },  // MemCpyPtr
                0b0001_1000 => {
                    let offset = Self::get_protected_ram_offset(&self.registers) as usize;
                    let src_addr = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) as usize + offset;
                    let comp_addr = ((bytes[1] & 0xFF) | (bytes[2] & 0xFF00)) as usize + offset;
                    let slice_size = (bytes[2] & 0xFF) as usize;
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![(src_addr..src_addr + slice_size).collect::<Vec<usize>>(), (comp_addr..comp_addr + slice_size).collect::<Vec<usize>>()].concat());
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 && (src_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize || comp_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize) {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    let comparison = self.ram[src_addr..src_addr + slice_size] == self.ram[comp_addr..comp_addr + slice_size];
                    self.registers[const { Register::ConditionFlag as usize }] = comparison as u16;
                },  // MemCmp
                0b0001_1001 => {
                    let offset = Self::get_protected_ram_offset(&self.registers) as usize;
                    let src_addr = self.registers[(bytes[0] & 0xFF) as usize] as usize + offset;
                    let comp_addr = self.registers[(bytes[1] >> 8) as usize] as usize + offset;
                    let slice_size = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![(src_addr..src_addr + slice_size).collect::<Vec<usize>>(), (comp_addr..comp_addr + slice_size).collect::<Vec<usize>>()].concat());
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 && (src_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize || comp_addr + slice_size > self.registers[const { Register::RamSize as usize }] as usize) {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    let comparison = self.ram[src_addr..src_addr + slice_size] == self.ram[comp_addr..comp_addr + slice_size];
                    self.registers[const { Register::ConditionFlag as usize }] = comparison as u16;
                },  // MemCmpPtr
                0b0001_1010 => {
                    let offset = (bytes[1] >> 8) | (bytes[1] << 8) + self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && offset > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers) + offset) as usize]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[2] >> 8) as usize]);
                    }
                    self.ram[(Self::get_protected_ram_offset(&self.registers) + offset) as usize] = self.registers[(bytes[2] >> 8) as usize];
                },  // StoPtrOff
                0b0001_1011 => {
                    let offset = (bytes[1] >> 8) | (bytes[1] << 8) + self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && offset > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers) + offset) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.ram[(Self::get_protected_ram_offset(&self.registers) + offset) as usize];
                },  // GetPtrOff
                0b0001_1100 => {
                    let start = self.registers[(bytes[0] & 0xFF) as usize] as usize + Self::get_protected_ram_offset(&self.registers) as usize;
                    let size = ((bytes[1] >> 8) | (bytes[1] << 8)) as usize;
                    if self.registers[const { Register::Protected as usize }] == 0 && start + size > self.registers[const { Register::RamSize as usize }] as usize {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some((start..start + size).collect());
                    }
                    self.ram[start..start + size].fill((bytes[2] >> 8) | (bytes[2] << 8));
                },  // MemFill
                0b0001_1101 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize] + self.registers[(bytes[1] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = self.registers[(bytes[1] >> 8) as usize];
                },  // StoPtrOffPtr
                0b0001_1110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_ram = Some(vec![]);
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[1] >> 8) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize] + self.registers[(bytes[1] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] >> 8) as usize] = self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize];
                },  // GetPtrOffPtr
                0b0001_1111 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_ram = Some(vec![((bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        read_ram = Some(vec![((bytes[1] & 0xFF) | (bytes[2] & 0xFF00) + Self::get_protected_ram_offset(&self.registers)) as usize]);
                    }
                    let index = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00);
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::RamSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.ram[(index + Self::get_protected_ram_offset(&self.registers)) as usize] = self.ram[((bytes[1] & 0xFF) | (bytes[2] & 0xFF00) + Self::get_protected_ram_offset(&self.registers)) as usize];
                },  // MovR
                0b0010_0000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::RamFrameStart as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetRamFrame
                0b0010_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::StackFrameStart as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetStackFrame
                0b0010_0010 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let ptr = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        #[cfg(debug_assertions)]
                        {
                            read_registers = Some((0..64).collect());
                            edited_ram = Some((ptr..ptr + 64).collect());
                        }
                        self.ram[ptr..ptr + 64].copy_from_slice(&self.registers[0..64]);
                    }
                    // only saving the first 64 for now
                },  // SaveRegisters
                0b0010_0011 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let ptr = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        #[cfg(debug_assertions)]
                        {
                            edited_registers = Some((0..64).collect());
                            read_ram = Some((ptr..ptr + 64).collect());
                        }
                        self.registers[0..64].copy_from_slice(&self.ram[ptr..ptr + 64]);
                    }
                    // only saving the first 64 for now
                },  // LodRegisters
                0b0010_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::TimeoutDuration as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetTimeout
                0b0010_0101 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::TimeOutCallbackAddr as usize }] = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00); }
                },  // SetTimeoutAdd
                0b0010_0110 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::InterruptCallbackAddr as usize }] = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00); }
                },  // SetIntAddr
                0b0010_0111 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        self.registers[const { Register::InterruptedLine as usize }] = pgc + 3;  // next line past this for returning to
                        self.registers[const { Register::ProgramCounter as usize }] = self.registers[const { Register::InterruptCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                        continue;  // no need for bounds checks and other stuff as it's already known to be protected
                    } else {
                        //
                    }  // todo! bios/sys interrupts for when protected (could be used for things like basic character rendering to save my sanity)
                },  // Int
                0b0010_1000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize];  // +3 to skip the header
                        self.registers[const { Register::Protected as usize }] = 0;
                        held_cycle_count = self.registers[const { Register::Cycles as usize }];
                    }
                },  // CallPgrm
                0b0010_1001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        self.registers[const { Register::ProgramCounter as usize }] = self.registers[const { Register::InterruptedLine as usize }];
                        self.registers[const { Register::Protected as usize }] = 0;
                    }
                },  // RetInt
                0b0010_1010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::ProgramStart as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetPgrmStart
                0b0010_1011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::RamSize as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetRamSize
                0b0010_1100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::ProgramSize as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetPgrmSize
                0b0010_1101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::StackSize as usize }] = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // SetStackSize
                0b0010_1110 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::FaultCallbackAddr as usize }] = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00); }
                },  // SetFaultAddr
                0b0010_1111 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { break; }  // todo! send the kill signal or something? idk
                },  // Kill
                0b0011_0000 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let dead_zone = 153600 - *self.frame_buffer_ptr.read();  // copy to here
                        let v_ram_base = ((bytes[1] & 0xFF) | (bytes[2] & 0xFF00)) as usize;
                        let size = (bytes[2] & 0xFF) as usize;
                        let ram_base = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) as usize;
                        #[cfg(debug_assertions)]
                        {
                            read_ram = Some((ram_base..ram_base + size).collect());
                        }
                        self.v_ram.write()[dead_zone + v_ram_base..dead_zone + v_ram_base + size].copy_from_slice(&self.ram[ram_base..ram_base + size]);
                    }
                },  // CpyRegion
                0b0011_0001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize, (bytes[1] & 0xFF) as usize]);
                        }
                        let dead_zone = 153600 - *self.frame_buffer_ptr.read();  // copy to here
                        let c = self.registers[(bytes[0] & 0xFF) as usize];
                        let x = self.registers[(bytes[1] >> 8) as usize] as usize;
                        let y = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        self.v_ram.write()[dead_zone + x + y * 480] = c;
                    }
                },  // Plot
                0b0011_0010 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let dead_zone = 153600 - *self.frame_buffer_ptr.read();  // copy to here
                        let v_ram_base = self.registers[(bytes[1] >> 8) as usize] as usize;
                        let size = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        let ram_base = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        #[cfg(debug_assertions)]
                        {
                            read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize, (bytes[1] & 0xFF) as usize]);
                            read_ram = Some((ram_base..ram_base + size).collect());
                        }
                        self.v_ram.write()[dead_zone + v_ram_base..dead_zone + v_ram_base + size].copy_from_slice(&self.ram[ram_base..ram_base + size]);
                    }
                },  // CpyRegionPtr
                0b0011_0011 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        self.registers[const { Register::ConditionFlag as usize }] = self.display_v_blank.try_recv().is_ok() as u16;
                    }
                },  // VBlank
                0b0011_0100 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let mut guard = self.frame_buffer_ptr.write();
                        *guard = 153600 - *guard;
                    }
                },  // SwapFrameBuf
                0b0011_0101 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let active_zone = *self.frame_buffer_ptr.read();
                        let x_pos = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) as usize;
                        let y_pos = ((bytes[1] & 0xFF) | (bytes[2] & 0xFF00)) as usize;
                        self.registers[(bytes[2] & 0xFF) as usize] = self.v_ram.read()[active_zone + x_pos + y_pos * 480];
                    }
                },  // ColorAt
                0b0011_0110 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let active_zone = *self.frame_buffer_ptr.read();
                        let x_pos = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        let y_pos = self.registers[(bytes[1] >> 8) as usize] as usize;
                        #[cfg(debug_assertions)]
                        {
                            read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                            edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                        }
                        self.registers[(bytes[1] & 0xFF) as usize] = self.v_ram.read()[active_zone + x_pos + y_pos * 480];
                    }
                },  // ColorPtr
                0b0011_0111 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let dead_zone = 153600 - *self.frame_buffer_ptr.read();
                        let ram_pos = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        let x_pos = self.registers[(bytes[1] >> 8) as usize] as usize;
                        let y_pos = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        let sprite_size_x = self.registers[(bytes[2] >> 8) as usize] as usize;
                        let sprite_size_y = self.registers[(bytes[2] & 0xFF) as usize] as usize;
                        let mut guard = self.v_ram.write();
                        for y in y_pos..y_pos + sprite_size_y {
                            for x in x_pos..x_pos + sprite_size_x {
                                guard[dead_zone + x + y * 480] = self.ram[ram_pos + x_pos + y_pos * sprite_size_x];
                            }
                        }
                    }
                },  // Place
                0b0011_1000 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let active_zone = *self.frame_buffer_ptr.read();
                        self.v_ram.write().copy_within(active_zone..active_zone + 153600, 153600 - active_zone);
                    }
                },  // CpyShown
                0b0011_1001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let dead_zone = 153600 - *self.frame_buffer_ptr.read();
                        let x_pos = self.registers[(bytes[0] & 0xFF) as usize] as usize;
                        let y_pos = self.registers[(bytes[1] >> 8) as usize] as usize;
                        let size_x = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        let size_y = self.registers[(bytes[2] >> 8) as usize] as usize;
                        let color = self.registers[(bytes[2] & 0xFF) as usize];
                        #[cfg(debug_assertions)]
                        {
                            read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize, (bytes[1] & 0xFF) as usize, (bytes[2] >> 8) as usize, (bytes[2] & 0xFF) as usize])
                        }
                        let mut guard = self.v_ram.write();
                        for y in y_pos..y_pos + size_y {
                            for x in x_pos..x_pos + size_x {
                                guard[dead_zone + x + y * 480] = color;
                            }
                        }
                    }
                },  // Solid
                0b0100_0000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] + self.registers[(bytes[1] >> 8) as usize];
                },  // Add
                0b0100_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] - self.registers[(bytes[1] >> 8) as usize];
                },  // Sub
                0b0100_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] - self.registers[(bytes[1] >> 8) as usize];
                },  // SubRev
                0b0100_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] * self.registers[(bytes[1] >> 8) as usize];
                },  // Mul
                0b0100_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] / self.registers[(bytes[1] >> 8) as usize];
                },  // Div
                0b0100_0101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] % self.registers[(bytes[1] >> 8) as usize];
                },  // Mod
                0b0100_0110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] & self.registers[(bytes[1] >> 8) as usize];
                },  // And
                0b0100_0111 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] | self.registers[(bytes[1] >> 8) as usize];
                },  // Or
                0b0100_1000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[1] >> 8) as usize]);
                    }
                    self.registers[(bytes[1] >> 8) as usize] = !self.registers[(bytes[0] & 0xFF) as usize];
                },  // Not
                0b0100_1001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] ^ self.registers[(bytes[1] >> 8) as usize];
                },  // Xor
                0b0100_1010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize].pow(self.registers[(bytes[1] >> 8) as usize] as u32);
                },  // Pow
                0b0100_1011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] << self.registers[(bytes[1] >> 8) as usize];
                },  // Left
                0b0100_1100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize] >> self.registers[(bytes[1] >> 8) as usize];
                },  // Right
                0b0100_1101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize].rotate_left(self.registers[(bytes[1] >> 8) as usize] as u32);
                },  // RotLeft
                0b0100_1110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.registers[(bytes[0] & 0xFF) as usize].rotate_right(self.registers[(bytes[1] >> 8) as usize] as u32);
                },  // RotRight
                0b0101_0000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] + ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // AddImm
                0b0101_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] - ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // SubImm
                0b0101_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = ((bytes[1] >> 8) | (bytes[1] << 8)) - self.registers[(bytes[0] & 0xFF) as usize];
                },  // SubRevImm
                0b0101_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] * ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // MulImm
                0b0101_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] / ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // DivImm
                0b0101_0101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] % ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // ModImm
                0b0101_0110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] & ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // AndImm
                0b0101_0111 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] | ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // OrImm
                0b0101_1001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize].pow(((bytes[1] >> 8) | (bytes[1] << 8)) as u32);
                },  // XorImm
                0b0101_1010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] << ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // PowImm
                0b0101_1011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize] >> ((bytes[1] >> 8) | (bytes[1] << 8));
                },  // LeftImm
                0b0101_1100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize].rotate_left(((bytes[1] >> 8) | (bytes[1] << 8)) as u32);
                },  // RightImm
                0b0101_1101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[(bytes[2] >> 8) as usize] = self.registers[(bytes[0] & 0xFF) as usize].rotate_right(((bytes[1] >> 8) | (bytes[1] << 8)) as u32);
                },  // RotLeftImm
                0b0101_1110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] < self.registers[(bytes[1] >> 8) as usize]) as u16;
                },  // Less
                0b0110_0001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] > self.registers[(bytes[1] >> 8) as usize]) as u16;
                },  // Grtr
                0b0110_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize, (bytes[1] >> 8) as usize]);
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] == self.registers[(bytes[1] >> 8) as usize]) as u16;
                },  // Eq
                0b0110_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] < ((bytes[1] >> 8) | (bytes[1] << 8))) as u16;
                },  // LessImm
                0b0110_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] > ((bytes[1] >> 8) | (bytes[1] << 8))) as u16;
                },  // GrtrImm
                0b0110_0101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                        edited_registers = Some(vec![(bytes[2] >> 8) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] == ((bytes[1] >> 8) | (bytes[1] << 8))) as u16;
                },  // EqImm
                0b0110_0110 => {
                    self.registers[const { Register::ConditionFlag as usize }] = 0;
                    self.registers[const { Register::FaultFlag as usize }] = 0;
                    self.registers[const { Register::ZeroFlag as usize }] = 0;
                    self.registers[const { Register::OverflowFlag as usize }] = 0;
                },  // ClrFlags
                0b0110_0111 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let value =
                        self.registers[const { Register::ConditionFlag as usize }] |
                        (self.registers[const { Register::FaultFlag as usize }] << 3) |
                        (self.registers[const { Register::ZeroFlag as usize }] << 1) |
                        (self.registers[const { Register::OverflowFlag as usize }] << 2);
                    self.registers[(bytes[0] & 0xFF) as usize] = value;
                },  // SaveFlags
                0b0110_1000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[const { Register::ConditionFlag as usize }] = (self.registers[(bytes[0] & 0xFF) as usize] == 0) as u16;
                },  // Zero
                0b0110_1001 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let value = self.registers[(bytes[0] & 0xFF) as usize];
                    self.registers[const { Register::ConditionFlag as usize }] = value & 0b1;
                    self.registers[const { Register::FaultFlag as usize }] = (value >> 3) & 0b1;
                    self.registers[const { Register::ZeroFlag as usize }] = (value >> 1) & 0b1;
                    self.registers[const { Register::OverflowFlag as usize }] = (value >> 2) & 0b1;
                },  // loadFlags
                
                0b0111_0000 => {
                    next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                },  // Jmp
                0b0111_0001 => {
                    if self.registers[const { Register::ConditionFlag as usize }] > 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // Jic
                0b0111_0010 => {
                    if self.registers[const { Register::ConditionFlag as usize }] == 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // Jnc
                0b0111_0011 => {
                    if self.registers[const { Register::ZeroFlag as usize }] > 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // Jiz
                0b0111_0100 => {
                    if self.registers[const { Register::ZeroFlag as usize }] == 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // Jnz
                0b0111_0101 => {
                    if self.registers[const { Register::FaultFlag as usize }] > 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JiErr
                0b0111_0110 => {
                    if self.registers[const { Register::FaultFlag as usize }] == 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JnErr
                0b0111_0111 => {
                    if self.registers[const { Register::OverflowFlag as usize }] > 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JiCry
                0b0111_1000 => {
                    if self.registers[const { Register::OverflowFlag as usize }] == 0 {
                        next_line = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JnCry
                0b0111_1001 => {
                    next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                },  // JmpPtr
                0b0111_1010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::ConditionFlag as usize }] > 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JicPtr
                0b0111_1011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::ConditionFlag as usize }] == 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JncPtr
                0b0111_1100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::ZeroFlag as usize }] > 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JizPtr
                0b0111_1101 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::ZeroFlag as usize }] == 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JnzPtr
                0b0111_1110 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::OverflowFlag as usize }] > 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JiCryPtr
                0b0111_1111 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::OverflowFlag as usize }] == 0 {
                        next_line = self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // JnCryPtr
                
                0b1000_0000 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.stack[(self.registers[const { Register::StackTopPtr as usize }] + Self::get_protected_stack_offset(&self.registers)) as usize] = self.registers[(bytes[0] & 0xFF) as usize];
                    self.registers[const { Register::StackTopPtr as usize }] += 1;
                    
                },  // Psh
                0b1000_0001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.stack[(self.registers[const { Register::StackTopPtr as usize }] + Self::get_protected_stack_offset(&self.registers)) as usize] = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00);
                    self.registers[const { Register::StackTopPtr as usize }] += 1;
                },  // PshCon
                0b1000_0010 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[(bytes[0] & 0xFF) as usize] = self.stack[(self.registers[const { Register::StackTopPtr as usize }] + Self::get_protected_stack_offset(&self.registers)) as usize];
                    self.registers[const { Register::StackTopPtr as usize }] -= 1;
                },  // Pop
                0b1000_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00);
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize];
                },  // Index
                0b1000_0100 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00));
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize] = self.registers[(bytes[1] & 0xFF) as usize];
                },  // Edit
                0b1000_0101 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            stack_trace.push(self.trace_ram[(self.registers[const { Register::ProgramCounter as usize }] + Self::get_protected_ram_offset(&self.registers)) as usize]);
                        }
                        self.stack[(self.registers[const { Register::StackTopPtr as usize }] + Self::get_protected_stack_offset(&self.registers)) as usize] = self.registers[const { Register::ProgramCounter as usize }] + 3;
                        self.registers[const { Register::StackTopPtr as usize }] += 1;
                        next_line = ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00)) + Self::get_protected_pgc_offset(&self.registers);
                    }
                },  // Call
                0b1000_0110 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            stack_trace.pop();
                        }
                        next_line = self.stack[(self.registers[const { Register::StackTopPtr as usize }] - 1 + Self::get_protected_stack_offset(&self.registers)) as usize];
                        self.registers[const { Register::StackTopPtr as usize }] -= 1;
                    }
                },  // Ret
                0b1000_0111 => {
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            stack_trace.pop();
                        }
                        next_line = self.stack[self.registers[(index + Self::get_protected_stack_offset(&self.registers)) as usize] as usize];
                    }
                },  // RetFramed
                0b1000_1000 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    self.registers[const { Register::StackTopPtr as usize }] = self.registers[(bytes[0] & 0xFF) as usize];
                },  // SetStackPtr
                0b1000_1001 => {
                    let constant = 0;
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] || self.registers[const { Register::StackTopPtr as usize }] - constant < self.registers[const { Register::StackFrameStart as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        #[cfg(debug_assertions)]
                        {
                            stack_trace.pop();
                        }
                        next_line = self.stack[(self.registers[const { Register::StackTopPtr as usize }] - 1 + Self::get_protected_stack_offset(&self.registers) - constant) as usize];
                        self.registers[const { Register::StackTopPtr as usize }] -= 1;
                    }
                },  // RetConst
                0b1000_1010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] >> 8) as usize] = self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize];
                },  // IndexPtr
                0b1000_1011 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = self.registers[const { Register::StackTopPtr as usize }] - self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] >> 8) as usize] = self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize];
                },  // IndexOff
                0b1000_1100 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[1] & 0xFF) as usize]);
                    }
                    let index = self.registers[const { Register::StackTopPtr as usize }] - ((bytes[0] & 0xFF) | (bytes[1] & 0xFF00));
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    self.registers[(bytes[1] & 0xFF) as usize] = self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize];
                },  // IndexOffConst
                0b1000_1101 => {
                    let index = self.registers[(bytes[0] & 0xFF) as usize];
                    if self.registers[const { Register::Protected as usize }] == 0 && index > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let value = self.ram[(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers)) as usize];
                    self.stack[(index + Self::get_protected_stack_offset(&self.registers)) as usize] = value;
                },  // EditPtr
                0b1000_1110 => {
                    if self.registers[const { Register::Protected as usize }] == 0 && self.registers[const { Register::StackTopPtr as usize }] > self.registers[const { Register::StackSize as usize }] {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    }
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    let value = self.ram[(self.registers[(bytes[0] & 0xFF) as usize] + Self::get_protected_ram_offset(&self.registers)) as usize];
                    self.stack[(self.registers[const { Register::StackTopPtr as usize }] + Self::get_protected_stack_offset(&self.registers)) as usize] = value;
                    self.registers[const { Register::StackTopPtr as usize }] += 1;
                },  // PshPtr
                0b1001_0000 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00);
                        let disc_addr_high = (bytes[1] & 0xFF) | (bytes[2] & 0xFF00);
                        self.disc[((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize] = self.registers[(bytes[2] & 0xFF) as usize];
                    }
                },  // Write
                0b1001_0001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = (bytes[0] & 0xFF) | (bytes[1] & 0xFF00);
                        let disc_addr_high = (bytes[1] & 0xFF) | (bytes[2] & 0xFF00);
                        self.registers[(bytes[2] & 0xFF) as usize] = self.disc[((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize];
                    }
                },  // Load
                0b1001_0010 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = self.registers[(bytes[0] & 0xFF) as usize];
                        let disc_addr_high = self.registers[(bytes[1] >> 8) as usize];
                        self.disc[((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize] = self.registers[(bytes[1] & 0xFF) as usize];
                    }
                },  // WritePtr
                0b1001_0011 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = self.registers[(bytes[0] & 0xFF) as usize];
                        let disc_addr_high = self.registers[(bytes[1] >> 8) as usize];
                        self.registers[(bytes[1] & 0xFF) as usize] = self.disc[((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize];
                    }
                },  // LoadPtr
                0b1001_0100 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = self.registers[(bytes[0] & 0xFF) as usize];
                        let disc_addr_high = self.registers[(bytes[1] >> 8) as usize];
                        let ram_start = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        let size = self.registers[(bytes[2] >> 8) as usize] as usize;
                        let disc_addr = ((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize;
                        self.disc[disc_addr..disc_addr + size].copy_from_slice(&self.ram[ram_start..ram_start + size]);
                        #[cfg(debug_assertions)]
                        {
                            self.trace_disc[disc_addr..disc_addr + size].copy_from_slice(&self.trace_ram[ram_start..ram_start + size]);
                        }
                    }
                },  // WriteSeg
                0b1001_0101 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else {
                        let disc_addr_low = self.registers[(bytes[0] & 0xFF) as usize];
                        let disc_addr_high = self.registers[(bytes[1] >> 8) as usize];
                        let ram_start = self.registers[(bytes[1] & 0xFF) as usize] as usize;
                        let size = self.registers[(bytes[2] >> 8) as usize] as usize;
                        let disc_addr = ((disc_addr_low as u32) | ((disc_addr_high as u32) << 16)) as usize;
                        self.ram[ram_start..ram_start + size].copy_from_slice(&self.disc[disc_addr..disc_addr + size]);
                        #[cfg(debug_assertions)]
                        {
                            self.trace_ram[ram_start..ram_start + size].copy_from_slice(&self.trace_disc[disc_addr..disc_addr + size]);
                        }
                    }
                },  // LoadSeg
                0b1010_0000 => {
                    #[cfg(debug_assertions)]
                    {
                        edited_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[(bytes[0] & 0xFF) as usize] = self.io_in_ports.read()[(bytes[1] >> 8) as usize].0; }
                },  // readIn
                0b1010_0001 => {
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.registers[const { Register::ConditionFlag as usize }] = self.io_in_ports.read()[(bytes[0] & 0xFF) as usize].1 as u16; }
                },  // readInFlag
                0b1010_0010 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.io_out_ports[(bytes[1] >> 8) as usize].0 = self.registers[(bytes[0] & 0xFF) as usize]; }
                },  // writeOut
                0b1010_0011 => {
                    #[cfg(debug_assertions)]
                    {
                        read_registers = Some(vec![(bytes[0] & 0xFF) as usize]);
                    }
                    if self.registers[const { Register::Protected as usize }] == 0 {
                        // calling the fault callback
                        next_line = self.registers[const { Register::FaultCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                    } else { self.io_out_ports[(bytes[1] >> 8) as usize].1 = self.registers[(bytes[0] & 0xFF) as usize] > 0; }
                },  // writeOutFlag
                _ => {
                    println!("Invalid instruction found");
                    break;
                },
            }
            
            #[cfg(debug_assertions)]
            {
                // // highlight_col: \x1B[48;2;55;55;55m
                println!("\x1b[1;1H{:0>4x} | {:0>4x} | {:0>4x}", bytes[0], bytes[1], bytes[2]);
                println!("\x1b[2;1H Stack Trace: {}{} [{}]                                ", stack_trace.iter().map(|e| format!("{} [{}] -> ", original_code[*e], e + 1)).collect::<Vec<String>>().join(""), original_code[self.trace_ram[pgc as usize]], self.trace_ram[pgc as usize] + 1);
                println!("\x1b[4;1HRegisters[0..16]: {}", { self.registers[0..16].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_registers {
                        Some(vector) => { vector.iter().any(|v| *v == i) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_registers {
                        Some(vector) => { vector.iter().any(|v| *v == i) },
                        _ => false,
                    } {
                        "\x1B[48;2;95;55;55m"
                    } else { "" }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[5;1HRam[0 ..16]     : {}", { self.ram[0..16].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i) },
                        _ => false,
                    } {
                        "\x1B[48;2;95;55;55m"
                    } else {
                            if i == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[6;1HRam[16..32]     : {}", { self.ram[16..32].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i + 16) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_ram {
                            Some(vector) => { vector.iter().any(|v| *v == i + 16) },
                            _ => false,
                        } {
                            "\x1B[48;2;95;55;55m"
                        } else {
                            if i + 16 == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[7;1HRam[32..48]     : {}", { self.ram[32..48].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i + 32) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                            if match &edited_ram {
                            Some(vector) => { vector.iter().any(|v| *v == i + 32) },
                            _ => false,
                        } {
                                "\x1B[48;2;95;55;55m"
                        } else {
                            if i + 32 == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[8;1HRam[48..64]     : {}", { self.ram[48..64].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i + 48) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_ram {
                            Some(vector) => { vector.iter().any(|v| *v == i + 48) },
                            _ => false,
                        } {
                            "\x1B[48;2;95;55;55m"
                        } else {
                            if i + 48 == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[9;1HRam[64..80]     : {}", { self.ram[64..80].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i + 64) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_ram {
                            Some(vector) => { vector.iter().any(|v| *v == i + 64) },
                            _ => false,
                        } {
                            "\x1B[48;2;95;55;55m"
                        } else {
                            if i + 64 == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[10;1HRam[80..96]     : {}", { self.ram[80..96].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if match &read_ram {
                        Some(vector) => { vector.iter().any(|v| *v == i + 80) },
                        _ => false,
                    } {
                        "\x1B[48;2;55;55;55m"
                    } else {
                        if match &edited_ram {
                            Some(vector) => { vector.iter().any(|v| *v == i + 80) },
                            _ => false,
                        } {
                            "\x1B[48;2;95;55;55m"
                        } else {
                            if i + 80 == self.registers[Register::ProgramCounter as usize] as usize {
                                "\x1B[48;2;55;95;55m"
                            } else { "" }
                        }
                    }
                }, v)).collect::<Vec<String>>().join(", ") });
                println!("\x1b[11;1HStack[0..16]    : {}", { self.stack[0..16].iter().enumerate().map(|(i, v)| format!("{}{:0>4x}\x1B[0m", {
                    if self.registers[const { Register::StackTopPtr as usize }] as usize == i {
                        "\x1B[48;2;55;55;55m"
                    } else { "" }
                }, v)).collect::<Vec<String>>().join(", ") });
                //std::thread::sleep(std::time::Duration::from_millis(250));
                // temp, to test things without it going too fast
                while !self.io_in_ports.read()[0].1 {
                    if self.kill_recv.try_recv().is_ok() { break 'em_loop; }
                }
                while self.io_in_ports.read()[0].1 {
                    self.io_in_ports.write()[0].1 = false;
                    if self.kill_recv.try_recv().is_ok() { break 'em_loop; }
                }
            }
            
            match self.registers[const { Register::Protected as usize }] {
                0 => {
                    if next_line - self.registers[const { Register::ProgramStart as usize }] > self.registers[const { Register::ProgramSize as usize }] {
                        println!("Seg Fault; accessed ram for the program memory beyond the allocated bounds for the current program.");
                        break;
                    }
                    if self.registers[const { Register::TimeoutDuration as usize }] + held_cycle_count > self.registers[const { Register::Cycles as usize }] {
                        self.registers[const { Register::ProgramCounter as usize }] = self.registers[const { Register::TimeOutCallbackAddr as usize }];
                        self.registers[const { Register::Protected as usize }] = 1;  // protected
                        
                        // done when re-entering a program instead (to avoid complications with timing when handling os stuff behind the scenes)
                        //held_cycle_count = self.registers[const { Register::Cycles as usize }];
                    }
                },  // unprotected
                _ => {
                    // protected
                }
            }
            self.registers[const { Register::ProgramCounter as usize }] = next_line;
        }
        let end = emulation_start.elapsed();
        let avg_cycle_duration = end.as_secs_f64() / iterations as f64;
        let avg_cps = 1f64 / avg_cycle_duration;
        let avg_cycle_duration = std::time::Duration::from_secs_f64(avg_cycle_duration);
        let iters_per_sec = self.registers[Register::Cycles as usize] as f64 / end.as_secs_f64();
        let avg_iter_duration = std::time::Duration::from_secs_f64(1f64 / iters_per_sec);
        print!("\x1b[14;1HTotal time    : {:?}\x1b[15;1HAvg Iteration : {:?}\x1b[16;1HAvg Ittrs/Sec : {:.0}\x1b[17;1HIterations    : {}\x1b[18;1HCycles        : {}\x1b[19;1HAvg Cycles/Sec: {:.0}\x1b[20;1HAvg Cycle     : {:?}", end, avg_cycle_duration, avg_cps, iterations, self.registers[Register::Cycles as usize], iters_per_sec, avg_iter_duration);
        
        crossterm::terminal::disable_raw_mode().unwrap();
        print!("\x1b[?25h");
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        crossterm::terminal::disable_raw_mode().unwrap();
        print!("\x1b[?30h\x1b[23;1H");
    }
}

/*
            Correct?!?!?!?!?!?
0101, 0500, 0000,  Instruction { name: "Ldi" , params: [Reg   , Const16], op_code: 1  }, [Register(1), Constant(5    )],  6
0102, 4a00, 0000,  Instruction { name: "Ldi" , params: [Reg   , Const16], op_code: 1  }, [Register(2), Constant(74   )],  8
0101, 00ff, 0000,  Instruction { name: "Ldi" , params: [Reg   , Const16], op_code: 1  }, [Register(1), Constant(65280)], 10
1001, 0000, ff00,  Instruction { name: "LdiR", params: [Addr16, Const16], op_code: 16 }, [Address (1), Constant(65280)], 11
*/

