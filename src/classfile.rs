const CAFEBABE: u32 = 0xCAFEBABE;
const MAJOR_VERSION: u16 = 52;
const MINOR_VERSION: u16 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct Classfile {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: Vec<Constant>,
    pub access_flags: u16,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<Interface>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Utf8(String),          //  1
    Integer(i32),          //  3
    Float(f32),            //  4
    Class(u16),            //  7
    String(u16),           //  8
    Fieldref(u16, u16),    //  9
    Methodref(u16, u16),   // 10
    NameAndType(u16, u16), // 12
}

#[derive(Clone, Debug, PartialEq)]
pub struct Interface;

#[derive(Clone, Debug, PartialEq)]
pub struct Field;

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Vec<Attribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Attribute {
    Code(u16, u16, u16, Vec<Instruction>, Vec<ExceptionTableEntry>, Vec<Attribute>),
    LineNumberTable(u16, Vec<LineNumberTableEntry>),
    SourceFile(u16, u16),
    StackMapTable(u16, Vec<StackMapFrame>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExceptionTableEntry;

#[derive(Clone, Debug, PartialEq)]
pub struct LineNumberTableEntry {
    pub start_pc: u16,
    pub line_number: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackMapFrame {
    SameFrame(u8),
    SameLocals1StackItemFrame(u8, VerificationType),
    SameLocals1StackItemFrameExtended(u16, VerificationType),
    ChopFrame(u8, u16),
    SameFrameExtended(u16),
    AppendFrame(u8, u16, Vec<VerificationType>),
    FullFrame(u16, Vec<VerificationType>, Vec<VerificationType>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerificationType {
    Top,                // 0
    Integer,            // 1
    Float,              // 2
    Long,               // 3
    Double,             // 4
    Null,               // 5
    UninitializedThis,  // 6
    Object(u16),        // 7
    Uninitialized(u16), // 8
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    Fload0,             // 0x22
    Fload1,             // 0x23
    Fload2,             // 0x24
    Fload3,             // 0x25
    Fload(u8),          // 0x17
    Fstore0,            // 0x43
    Fstore1,            // 0x44
    Fstore2,            // 0x45
    Fstore3,            // 0x46
    Fstore(u8),         // 0x38
    Fconst0,            // 0x0b
    Fconst1,            // 0x0c
    Fconst2,            // 0x0d
    FReturn,            // 0xae
    I2C,                // 0x92
    IconstM1,           // 0x02
    Iconst0,            // 0x03
    Iconst1,            // 0x04
    Iconst2,            // 0x05
    Iconst3,            // 0x06
    Iconst4,            // 0x07
    Iconst5,            // 0x08
    Istore0,            // 0x3b
    Istore1,            // 0x3c
    Istore2,            // 0x3d
    Istore3,            // 0x3e
    Istore(u8),         // 0x36
    Bipush(u8),         // 0x10
    Sipush(u8, u8),     // 0x11
    Iload0,             // 0x1a
    Iload1,             // 0x1b
    Iload2,             // 0x1c
    Iload3,             // 0x1d
    Iload(u8),          // 0x15
    LoadConstant(u8),   // 0x12
    Aload0,             // 0x2A
    Aload1,             // 0x2B
    Aload2,             // 0x2C
    Aload3,             // 0x2D
    Aaload,             // 0x32
    Iadd,               // 0x60
    Isub,               // 0x64
    Imul,               // 0x68
    Idiv,               // 0x6c
    IfEq(u16),          // 0x99
    IfNe(u16),          // 0x9A
    IfLt(u16),          // 0x9B
    IfGe(u16),          // 0x9C
    IfGt(u16),          // 0x9D
    IfLe(u16),          // 0x9E
    IfIcmpEq(u16),      // 0x9F
    IfIcmpNe(u16),      // 0xA0
    IfIcmpLt(u16),      // 0xA1
    IfIcmpGe(u16),      // 0xA2
    IfIcmpGt(u16),      // 0xA3
    IfIcmpLe(u16),      // 0xA4
    Goto(u16),          // 0xA7
    IReturn,            // 0xac
    Return,             // 0xB1
    GetStatic(u16),     // 0xB2
    InvokeVirtual(u16), // 0xB6
    InvokeSpecial(u16), // 0xB7
    InvokeStatic(u16),  // 0xB8
    ArrayLength,        // 0xBE
}

impl Classfile {
    pub fn new(constants: Vec<Constant>, access_flags: u16, this_class: u16, super_class: u16, methods: Vec<Method>) -> Classfile {
        Classfile {
            magic: CAFEBABE,
            minor_version: MINOR_VERSION,
            major_version: MAJOR_VERSION,
            constant_pool: constants,
            access_flags: access_flags,
            this_class: this_class,
            super_class: super_class,
            interfaces: vec![],
            fields: vec![],
            methods: methods,
            attributes: vec![],
        }
    }

    pub fn lookup_constant(&self, index: u16) -> &Constant {
        &self.constant_pool[index as usize - 1]
    }

    pub fn lookup_string(&self, index: u16) -> &str {
        let val = self.lookup_constant(index);
        match *val {
            Constant::Utf8(ref str) => str,
            _ => panic!("Wanted string, found {:?}", val)
        }
    }
}

impl Method {
    pub fn new(access_flags: u16, name_index: u16, descriptor_index: u16,
               attributes: Vec<Attribute>) -> Method {
        Method {
            access_flags: access_flags,
            name_index: name_index,
            descriptor_index: descriptor_index,
            attributes: attributes,
        }
    }
}

impl Instruction {
    pub fn size(&self) -> u8 {
        match *self {
            Instruction::Fload0 => 1,
            Instruction::Fload1 => 1,
            Instruction::Fload2 => 1,
            Instruction::Fload3 => 1,
            Instruction::Fload(u8) => 2,
            Instruction::Fstore0 => 1, 
            Instruction::Fstore1 => 1, 
            Instruction::Fstore2 => 1, 
            Instruction::Fstore3 => 1, 
            Instruction::Fstore(u8) => 2,
            Instruction::Fconst0 => 1,   
            Instruction::Fconst1 => 1,   
            Instruction::Fconst2 => 1,
            Instruction::FReturn => 1,
            Instruction::I2C => 1,
            Instruction::IconstM1 => 1,
            Instruction::Iconst0 => 1,
            Instruction::Iconst1 => 1,
            Instruction::Iconst2 => 1,
            Instruction::Iconst3 => 1,
            Instruction::Iconst4 => 1,
            Instruction::Iconst5 => 1,
            Instruction::Istore0 => 1,
            Instruction::Istore1 => 1,
            Instruction::Istore2 => 1,
            Instruction::Istore3 => 1,
            Instruction::Istore(_) => 2,
            Instruction::Bipush(_) => 2,
            Instruction::Sipush(_, _) => 3,
            Instruction::Iload0 => 1,
            Instruction::Iload1 => 1,
            Instruction::Iload2 => 1,
            Instruction::Iload3 => 1,
            Instruction::Iload(_) => 2,
            Instruction::LoadConstant(_) => 2,
            Instruction::Aload0 => 1,
            Instruction::Aload1 => 1,
            Instruction::Aload2 => 1,
            Instruction::Aload3 => 1,
            Instruction::Aaload => 1,
            Instruction::Iadd => 1,
            Instruction::Isub => 1,
            Instruction::Imul => 1,
            Instruction::Idiv => 1,
            Instruction::IfEq(_) => 3,
            Instruction::IfNe(_) => 3,
            Instruction::IfLt(_) => 3,
            Instruction::IfGe(_) => 3,
            Instruction::IfGt(_) => 3,
            Instruction::IfLe(_) => 3,
            Instruction::IfIcmpEq(_) => 3,
            Instruction::IfIcmpNe(_) => 3,
            Instruction::IfIcmpLt(_) => 3,
            Instruction::IfIcmpGe(_) => 3,
            Instruction::IfIcmpGt(_) => 3,
            Instruction::IfIcmpLe(_) => 3,
            Instruction::Goto(_) => 3,
            Instruction::IReturn => 1,
            Instruction::Return => 1,
            Instruction::GetStatic(_) => 3,
            Instruction::InvokeVirtual(_) => 3,
            Instruction::InvokeSpecial(_) => 3,
            Instruction::InvokeStatic(_) => 3,
            Instruction::ArrayLength => 1,
        }
    }
}
