
use machine;

enum Expression {

}

/*
   syntax for grammar:

   lambda {
      main { expr }
      expr { "(" expr ")" / app / abs / var }
      app { expr " " expr }
      abs { abs_multi / abs_single }
      abs_multi { "\" var (" " var)+ "." expr }
      abs_single { "\" var " " expr }
      var { ["a".."z""A".."Z"] }
   }

   name { ["a".."z"]+ }
}
*/

fn translate(grammar: &str) {
    //machine::Machine::new()
}
