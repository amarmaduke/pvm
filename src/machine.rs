

enum StackFrame {

}

enum Instruction {

}

pub struct Machine {
   program : Vec<Instruction>,
   stack : Vec<StackFrame>,
   pc : u32
}

impl Machine {
   pub fn execute() {

   }

   pub fn new() -> Machine {
      Machine {
         program: vec![],
         stack: vec![],
         pc: 0
      }
   }
}
