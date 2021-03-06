use std::collections::HashMap;

use classfile::*;
use java_type_signatures::*;

pub const ACC_PUBLIC: u16 = 0x1;
pub const ACC_STATIC: u16 = 0x8;

pub struct ClassBuilder {
    access_flags: u16,
    this_class_index: u16,
    super_class_index: u16,
    constants: Vec<Constant>,
    methods: Vec<Method>,
}

impl ClassBuilder {
    pub fn new(access_flags: u16, this_class: &str, super_class: &str) -> ClassBuilder {
        let mut builder = ClassBuilder {
            access_flags: access_flags,
            this_class_index: 0,
            super_class_index: 0,
            constants: vec![],
            methods: vec![],
        };

        builder.this_class_index = builder.define_class(this_class);
        builder.super_class_index = builder.define_class(super_class);
        builder
    }

    pub fn define_method(&mut self, access_flags: u16, name: &str, argument_types: &[Java],
                         return_type: &Java) -> MethodBuilder {
        MethodBuilder::new(self, access_flags, name, argument_types, return_type)
    }
    
    fn push_constant(&mut self, constant: Constant) -> u16 {
        let mut i: u16 = 1;
        for c in &self.constants {
            if constant == *c {
                return i;
            }

            i += 1;
        }
        
        self.constants.push(constant);
        self.constants.len() as u16
    }

    fn define_integer(&mut self, n: i32) -> u16 {
        self.push_constant(Constant::Integer(n))
    }

    fn define_float(&mut self, n: f32) -> u16 {
        self.push_constant(Constant::Float(n))
    }
    
    fn define_utf8(&mut self, string: &str) -> u16 {
        self.push_constant(Constant::Utf8(string.to_owned()))
    }

    fn define_class(&mut self, class: &str) -> u16 {
        let name_index = self.define_utf8(class);
        self.push_constant(Constant::Class(name_index))
    }

    fn define_string(&mut self, value: &str) -> u16 {
        let string_index = self.define_utf8(value);
        self.push_constant(Constant::String(string_index))
    }

    fn define_fieldref(&mut self, class: &str, name: &str, field_type: &Java) -> u16 {
        let class_index = self.define_class(class);
        let descriptor = format!("{}", field_type);
        let name_and_type_index = self.define_name_and_type(name, &descriptor);
        self.push_constant(Constant::Fieldref(class_index, name_and_type_index))
    }

    fn define_methodref(&mut self, class: &str, name: &str, argument_types: &[Java],
                        return_type: &Java) -> u16 {
        let class_index = self.define_class(class);
        let descriptor = method_signature(argument_types, return_type);
        let name_and_type_index = self.define_name_and_type(name, &descriptor);
        self.push_constant(Constant::Methodref(class_index, name_and_type_index))
    }

    fn define_name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
        let name_index = self.define_utf8(name);
        let descriptor_index = self.define_utf8(&descriptor);
        self.push_constant(Constant::NameAndType(name_index, descriptor_index))
    }

    pub fn done(self) -> Classfile {
        Classfile::new(self.constants, self.access_flags, self.this_class_index,
                       self.super_class_index, self.methods)
    }
}

pub struct MethodBuilder<'a> {
    classfile: &'a mut ClassBuilder,
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    instructions: Vec<(u16, IntermediateInstruction<'a>)>,
    labels: HashMap<(String, u16), u16>,
    stack_index: u16,
    curr_stack_depth: u16,
    max_stack_depth: u16,
    stack_frames: Vec<StackMapFrame>,
    last_stack_frame_index: Option<u16>,
    num_locals: u16,
    stack_types: Vec<VerificationType>,
    env_num: u16,
    env_count: u16,
}

#[derive(Debug)]
pub enum IntermediateInstruction<'a> {
    Ready(Instruction),
    Waiting(&'a str, u16, Instruction),
}

impl<'a> MethodBuilder<'a> {
    fn new(classfile: &'a mut ClassBuilder, access_flags: u16, name: &str,
           argument_types: &[Java], return_type: &Java) -> MethodBuilder<'a> {
        let name_index = classfile.define_utf8(name);
        let descriptor = method_signature(argument_types, return_type);
        let descriptor_index = classfile.define_utf8(&descriptor);
        MethodBuilder {
            classfile: classfile,
            access_flags: access_flags,
            name_index: name_index,
            descriptor_index: descriptor_index,
            instructions: vec![],
            labels: HashMap::new(),
            stack_index: 0,
            curr_stack_depth: 0,
            max_stack_depth: 0,
            stack_frames: vec![],
            last_stack_frame_index: None,
            num_locals: argument_types.len() as u16,
            stack_types: Vec::new(),
            env_num: 0,
            env_count: 0,
        }
    }

    pub fn new_env(&mut self) -> u16 {
        self.env_count += 1;
        self.env_count
    }

    pub fn set_env(&mut self, n: u16) {
        self.env_num = n;
    }

    pub fn set_new_env(&mut self) -> u16 {
        self.env_num = self.new_env();
        self.env_num
    }
    
    pub fn nyew(&mut self, class_name: &str) {
        let idx: u16 = self.classfile.define_class(class_name);
        // the index needs to be split into two u8s (idx1 is the bigger half)
        let idx1 = (idx >> 8) as u8;
        let idx2 = (idx | 0xff) as u8;
        self.push_instruction(Instruction::New(idx1, idx2));
        self.increase_stack_depth();
    }

    pub fn dup(&mut self) {
        self.push_instruction(Instruction::Dup);
        self.increase_stack_depth();
    }

    pub fn i2c(&mut self) {
        self.push_instruction(Instruction::I2C);
    }

    pub fn i2f(&mut self) {
        self.push_instruction(Instruction::I2F);
    }

    pub fn f2i(&mut self) {
        self.push_instruction(Instruction::F2I);
    }
    
    pub fn irem(&mut self) {
        self.push_instruction(Instruction::Irem);
        self.decrease_stack_depth();
    }

    pub fn frem(&mut self) {
        self.push_instruction(Instruction::Frem);
        self.decrease_stack_depth();
    }
    
    pub fn iconstm1(&mut self) {
        self.push_instruction(Instruction::IconstM1);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst0(&mut self) {
        self.push_instruction(Instruction::Iconst0);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst1(&mut self) {
        self.push_instruction(Instruction::Iconst1);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst2(&mut self) {
        self.push_instruction(Instruction::Iconst2);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst3(&mut self) {
        self.push_instruction(Instruction::Iconst3);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst4(&mut self) {
        self.push_instruction(Instruction::Iconst4);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iconst5(&mut self) {
        self.push_instruction(Instruction::Iconst5);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn istore0(&mut self) {
        self.push_instruction(Instruction::Istore0);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn istore1(&mut self) {
        self.push_instruction(Instruction::Istore1);
        self.decrease_stack_depth();
        self.increase_locals();
    }
    
    pub fn istore2(&mut self) {
        self.push_instruction(Instruction::Istore2);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn istore3(&mut self) {
        self.push_instruction(Instruction::Istore3);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn istore(&mut self, idx: u8) {
        self.push_instruction(Instruction::Istore(idx));
        self.decrease_stack_depth();
        self.increase_locals();
    }
    

    pub fn fconst0(&mut self) {
        self.push_instruction(Instruction::Fconst0);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Float);
    }

    pub fn fconst1(&mut self) {
        self.push_instruction(Instruction::Fconst1);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn fconst2(&mut self) {
        self.push_instruction(Instruction::Fconst2);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn fstore0(&mut self) {
        self.push_instruction(Instruction::Fstore0);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn fstore1(&mut self) {
        self.push_instruction(Instruction::Fstore1);
        self.decrease_stack_depth();
        self.increase_locals();
    }
    
    pub fn fstore2(&mut self) {
        self.push_instruction(Instruction::Fstore2);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn fstore3(&mut self) {
        self.push_instruction(Instruction::Fstore3);
        self.decrease_stack_depth();
        self.increase_locals();
    }

    pub fn fstore(&mut self, idx: u8) {
        self.push_instruction(Instruction::Fstore(idx));
        self.decrease_stack_depth();
        self.increase_locals();
    }
    
    pub fn fload0(&mut self) {
        self.push_instruction(Instruction::Fload0);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }
    
    pub fn fload1(&mut self) {
        self.push_instruction(Instruction::Fload1);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn fload2(&mut self) {
        self.push_instruction(Instruction::Fload2);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn fload3(&mut self) {
        self.push_instruction(Instruction::Fload3);
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn fload(&mut self, reg: u8) {
        self.push_instruction(Instruction::Fload(reg));
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }    

    pub fn fadd(&mut self) {
        self.push_instruction(Instruction::Fadd);
        self.decrease_stack_depth();
    }

    pub fn fsub(&mut self) {
        self.push_instruction(Instruction::Fsub);
        self.decrease_stack_depth();
    }

    pub fn fmul(&mut self) {
        self.push_instruction(Instruction::Fmul);
        self.decrease_stack_depth();
    }

    pub fn fdiv(&mut self) {
        self.push_instruction(Instruction::Fdiv);
        self.decrease_stack_depth();
    }
    
    pub fn bipush(&mut self, value: i8) {
        self.push_instruction(Instruction::Bipush(value as u8));
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn sipush(&mut self, val0: i8, val1: i8) {
        self.push_instruction(Instruction::Sipush(val0 as u8, val1 as u8));
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }
    
    pub fn iload0(&mut self) {
        self.push_instruction(Instruction::Iload0);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }
    
    pub fn iload1(&mut self) {
        self.push_instruction(Instruction::Iload1);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iload2(&mut self) {
        self.push_instruction(Instruction::Iload2);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iload3(&mut self) {
        self.push_instruction(Instruction::Iload3);
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn iload(&mut self, reg: u8) {
        self.push_instruction(Instruction::Iload(reg));
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }    
    
    pub fn load_constant(&mut self, value: &str) {
        let string_index = self.classfile.define_string(value);
        if string_index > ::std::u8::MAX as u16 {
            panic!("Placed a constant in too high of an index: {}", string_index)
        }
        self.push_instruction(Instruction::LoadConstant(string_index as u8));
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn load_constant_integer(&mut self, value: i32) {
        let i32_index = self.classfile.define_integer(value);
        if i32_index > ::std::u8::MAX as u16 {
            panic!("Placed a constant in too high of an index: {}", i32_index)
        }
        self.push_instruction(Instruction::LoadConstant(i32_index as u8));
        self.increase_stack_depth();
        self.stack_types.push(VerificationType::Integer);
    }

    pub fn load_constant_float(&mut self, value: f32) {
        let f32_index = self.classfile.define_float(value);
        if f32_index > ::std::u8::MAX as u16 {
            panic!("Placed a constant in too high of an index: {}", f32_index)
        }
        self.push_instruction(Instruction::LoadConstant(f32_index as u8));
        self.increase_stack_depth();
        //self.stack_types.push(VerificationType::Integer);
    }

    pub fn aconst_null(&mut self) {
        self.push_instruction(Instruction::AConstNull);
        self.increase_stack_depth();
    }

    pub fn astore0(&mut self) {
        self.push_instruction(Instruction::Astore0);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn astore1(&mut self) {
        self.push_instruction(Instruction::Astore1);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn astore2(&mut self) {
        self.push_instruction(Instruction::Astore2);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn astore3(&mut self) {
        self.push_instruction(Instruction::Astore3);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn astore(&mut self, reg: u8) {
        self.push_instruction(Instruction::Astore(reg));
        self.increase_stack_depth();
    }
    
    pub fn aload0(&mut self) {
        self.push_instruction(Instruction::Aload0);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn aload1(&mut self) {
        self.push_instruction(Instruction::Aload1);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn aload2(&mut self) {
        self.push_instruction(Instruction::Aload2);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn aload3(&mut self) {
        self.push_instruction(Instruction::Aload3);
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn aload(&mut self, reg: u8) {
        self.push_instruction(Instruction::Aload(reg));
        self.increase_stack_depth();
    }
    
    pub fn aaload(&mut self) {
        self.push_instruction(Instruction::Aaload);
        self.decrease_stack_depth();
        // TODO: push to stack_types
    }

    pub fn iadd(&mut self) {
        self.push_instruction(Instruction::Iadd);
        self.decrease_stack_depth();
    }

    pub fn isub(&mut self) {
        self.push_instruction(Instruction::Isub);
        self.decrease_stack_depth();
    }

    pub fn imul(&mut self) {
        self.push_instruction(Instruction::Imul);
        self.decrease_stack_depth();
    }

    pub fn idiv(&mut self) {
        self.push_instruction(Instruction::Idiv);
        self.decrease_stack_depth();
    }
    
    pub fn ifeq(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfEq(0));
        self.decrease_stack_depth();
    }

    pub fn ifne(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfNe(0));
        self.decrease_stack_depth();
    }

    pub fn iflt(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfLt(0));
        self.decrease_stack_depth();
    }

    pub fn ifge(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfGe(0));
        self.decrease_stack_depth();
    }

    pub fn ifgt(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfGt(0));
        self.decrease_stack_depth();
    }

    pub fn ifle(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfLe(0));
        self.decrease_stack_depth();
    }

    pub fn if_icmp_eq(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpEq(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn if_icmp_ne(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpNe(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn if_icmp_lt(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpLt(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn if_icmp_ge(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpGe(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn if_icmp_gt(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpGt(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn if_icmp_le(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::IfIcmpLe(0));
        self.decrease_stack_depth_by(2);
        // TODO: push to stack_types?
    }

    pub fn goto(&mut self, label: &'a str) {
        self.delay_instruction(label, Instruction::Goto(0));
    }
    
    pub fn ireturn(&mut self) {
        self.push_instruction(Instruction::IReturn);
        self.decrease_stack_depth();
    }

    pub fn freturn(&mut self) {
        self.push_instruction(Instruction::FReturn);
        self.decrease_stack_depth();
    }
    
    pub fn do_return(&mut self) {
        self.push_instruction(Instruction::Return);
    }

    pub fn areturn(&mut self) {
        self.push_instruction(Instruction::Areturn);
    }
    
    pub fn get_static(&mut self, class: &str, name: &str, argument_type: &Java) {
        let fieldref_index = self.classfile.define_fieldref(class, name, argument_type);
        self.push_instruction(Instruction::GetStatic(fieldref_index));
        self.increase_stack_depth();
        // TODO: push to stack_types
    }

    pub fn invoke_virtual(&mut self, class: &str, name: &str,
                          argument_types: &[Java], return_type: &Java) {
        let methodref_index =
            self.classfile.define_methodref(class, name, argument_types, return_type);
        self.push_instruction(Instruction::InvokeVirtual(methodref_index));
        self.decrease_stack_depth_by(argument_types.len() as u8 + 1);
        if *return_type != Java::Void { self.increase_stack_depth(); }
        // TODO: push to stack_types
    }

    pub fn invoke_special(&mut self, class: &str, name: &str,
                          argument_types: &[Java], return_type: &Java) {
        let methodref_index =
            self.classfile.define_methodref(class, name, argument_types, return_type);
        self.push_instruction(Instruction::InvokeSpecial(methodref_index));
        self.decrease_stack_depth_by(argument_types.len() as u8 + 1);
        if *return_type != Java::Void { self.increase_stack_depth(); }
        // TODO: push to stack_types
    }

    pub fn invoke_static(&mut self, class: &str, name: &str,
                         argument_types: &[Java], return_type: &Java) {
        let methodref_index =
            self.classfile.define_methodref(class, name, argument_types, return_type);
        self.push_instruction(Instruction::InvokeStatic(methodref_index));
        self.decrease_stack_depth_by(argument_types.len() as u8);
        if *return_type != Java::Void { self.increase_stack_depth(); }
        // TODO: push to stack_types
    }

    pub fn array_length(&mut self) {
        self.push_instruction(Instruction::ArrayLength);
        // TODO: push to stack_types?
    }

    pub fn label(&mut self, name: &str) {
        let env = self.env_num;
        self.labels.insert((name.to_owned(), env), self.stack_index);
        
        // create a stack map table entry
        let offset = match self.last_stack_frame_index {
            Some(i) => self.stack_index - i - 1,
            None => self.stack_index
        };
        
        let frame = if self.stack_types.is_empty() {
            if offset > ::std::u8::MAX as u16 {
                StackMapFrame::SameFrameExtended(offset)
            } else {
                StackMapFrame::SameFrame(offset as u8)
            }
        } else {
            let last_type = self.stack_types[self.stack_types.len()-1].clone();
            if offset > ::std::u8::MAX as u16 {
                StackMapFrame::SameLocals1StackItemFrameExtended(offset, last_type)
            } else {
                StackMapFrame::SameLocals1StackItemFrame(offset as u8, last_type)
            }
        };
        
        self.stack_frames.push(frame);
        self.last_stack_frame_index = Some(self.stack_index);
    }

    fn push_instruction(&mut self, instruction: Instruction) {
        let index = self.stack_index;
        self.stack_index += instruction.size() as u16;
        self.instructions.push((index, IntermediateInstruction::Ready(instruction)));
    }

    fn delay_instruction(&mut self, label: &'a str, instruction: Instruction) {
        let index = self.stack_index;
        let env = self.env_num;
        self.stack_index += instruction.size() as u16;
        self.instructions.push((index, IntermediateInstruction::Waiting(label, env,
                                                                        instruction)));
    }

    fn increase_locals(&mut self) {
        self.num_locals += 1;
    }

    fn increase_stack_depth(&mut self) {
        // self.curr_stack_depth += 1;
        // if self.curr_stack_depth > self.max_stack_depth {
        //     self.max_stack_depth = self.curr_stack_depth;
        // }
    }

    fn decrease_stack_depth(&mut self) {
        // if self.curr_stack_depth > 0 {
        //     self.curr_stack_depth -= 1;
        //     self.stack_types.pop();
        // }
    }

    fn decrease_stack_depth_by(&mut self, n: u8) {
        // self.curr_stack_depth -= n as u16;
        // TODO: pop from stack_types
    }
    
    pub fn done(self) {
        // if self.curr_stack_depth != 0 {
        //     println!("Warning: stack depth at the end of a method should be 0, but is {} instead", self.curr_stack_depth);
        // }

        let classfile = self.classfile;
        let labels = self.labels;
        let real_instructions = self.instructions.into_iter().map(|(pos, ir)| match ir {
            IntermediateInstruction::Ready(i) => i,
            IntermediateInstruction::Waiting(l, e, i) => {
                let tup = (l.to_string(), e);
                let label_pos = labels.get(&tup).unwrap();
                let offset = label_pos - pos;
                fill_offset(i, offset)
            }
        }).collect();
        
        let stack_map_table_index = classfile.define_utf8("StackMapTable");
        let stack_map_table = Attribute::StackMapTable(stack_map_table_index,
                                                       self.stack_frames);
        
        let code_index = classfile.define_utf8("Code");
        let code = Attribute::Code(code_index, self.max_stack_depth, self.num_locals,
                                   real_instructions, vec![], vec![stack_map_table]);

        let method = Method::new(self.access_flags, self.name_index, self.descriptor_index,
                                 vec![code]);
        classfile.methods.push(method);
    }
}

fn fill_offset(instruction: Instruction, offset: u16) -> Instruction {
    match instruction {
        Instruction::IfEq(_) => Instruction::IfEq(offset),
        Instruction::IfNe(_) => Instruction::IfNe(offset),
        Instruction::IfLt(_) => Instruction::IfLt(offset),
        Instruction::IfGe(_) => Instruction::IfGe(offset),
        Instruction::IfGt(_) => Instruction::IfGt(offset),
        Instruction::IfLe(_) => Instruction::IfLe(offset),
        Instruction::IfIcmpEq(_) => Instruction::IfIcmpEq(offset),
        Instruction::IfIcmpNe(_) => Instruction::IfIcmpNe(offset),
        Instruction::IfIcmpLt(_) => Instruction::IfIcmpLt(offset),
        Instruction::IfIcmpGe(_) => Instruction::IfIcmpGe(offset),
        Instruction::IfIcmpGt(_) => Instruction::IfIcmpGt(offset),
        Instruction::IfIcmpLe(_) => Instruction::IfIcmpLe(offset),
        Instruction::Goto(_) => Instruction::Goto(offset),
        _ => panic!("Instruction type doesn't have an offset to fill: {:?}", instruction)
    }
}
