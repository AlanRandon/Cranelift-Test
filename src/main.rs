use cranelift::codegen;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module};
use std::collections::HashMap;

fn main() {
    let isa = cranelift_native::builder()
        .unwrap()
        .finish(codegen::settings::Flags::new(codegen::settings::builder()))
        .unwrap();

    let jit_builder = JITBuilder::with_isa(isa.clone(), cranelift_module::default_libcall_names());

    let mut module = JITModule::new(jit_builder);

    let mut ctx = module.make_context();

    let functions = cranelift_reader::parse_functions(include_str!("avg.clif"))
        .unwrap()
        .into_iter()
        .map(|func| {
            let name = func.name.to_string();
            ctx.func = func;
            ctx.compile(isa.as_ref(), &mut Default::default()).unwrap();

            let func_id = module
                .declare_function(&name, Linkage::Export, &ctx.func.signature)
                .map_err(|e| e.to_string())
                .unwrap();

            module
                .define_function(func_id, &mut ctx)
                .map_err(|e| e.to_string())
                .unwrap();

            module.clear_context(&mut ctx);

            module.finalize_definitions().unwrap();

            (name, module.get_finalized_function(func_id))
        })
        .collect::<HashMap<_, _>>();
    let average = unsafe {
        std::mem::transmute::<_, unsafe fn(*const f32, i64) -> f32>(
            functions.get("%average").unwrap(),
        )
    };

    let data = vec![1.0f32, 2.0, 3.0];

    let result = unsafe { average(data.as_slice().as_ptr(), data.len() as i64) };
    println!("{result}");
}
