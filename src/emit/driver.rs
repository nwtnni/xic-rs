use crate::check;
use crate::emit;
use crate::data::ast;

#[derive(Debug)]
pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
    fold: bool,
}

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool, fold: bool) -> Self {
        Driver { directory, diagnostic, fold }
    }

    pub fn drive(&self, ast: &ast::Program, env: &check::Env) {
        let emitter = emit::Emitter::new(env);
        let hir = emitter.emit_program(ast);
        println!("{:#?}", hir);
    }
}
