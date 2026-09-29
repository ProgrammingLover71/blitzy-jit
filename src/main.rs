use std::error::Error;
use std::mem::transmute;

use cranelift::prelude::*;
use cranelift::codegen::ir::UserFuncName;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};

fn main() -> Result<(), Box<dyn Error>> {
    // ------------------------------------------------------------
    // 1. Create the JIT compiler.
    // ------------------------------------------------------------

    // JITBuilder::new() creates a JIT for the native machine.
    // It also configures the settings needed by the JIT.
    let jit_builder = JITBuilder::new(default_libcall_names())?;

    let mut module = JITModule::new(jit_builder);

    // We will use this when finalizing FunctionBuilder objects.
    //
    // IMPORTANT:
    // FunctionBuilder::finalize() needs a TargetFrontendConfig.
    // The Module gives us the correct one for its target.
    let target_config = module.target_config();

    // ------------------------------------------------------------
    // 2. Declare:
    //
    //     fn abs(x: i64) -> i64
    //
    // ------------------------------------------------------------

    let mut abs_signature = module.make_signature();

    abs_signature
        .params
        .push(AbiParam::new(types::I64));

    abs_signature
        .returns
        .push(AbiParam::new(types::I64));

    let abs_id = module.declare_function(
        "abs",
        Linkage::Export,
        &abs_signature,
    )?;

    // ------------------------------------------------------------
    // 3. Generate the body of abs().
    //
    //     if x < 0 {
    //         return -x;
    //     } else {
    //         return x;
    //     }
    // ------------------------------------------------------------

    let mut abs_ctx = module.make_context();

    abs_ctx.func.signature = abs_signature.clone();
    abs_ctx.func.name = UserFuncName::user(0, abs_id.as_u32());

    let mut abs_builder_ctx = FunctionBuilderContext::new();

    {
        let mut builder =
            FunctionBuilder::new(&mut abs_ctx.func, &mut abs_builder_ctx);

        // Create the basic blocks.
        let entry = builder.create_block();
        let negative = builder.create_block();
        let non_negative = builder.create_block();

        // The entry block receives the function arguments.
        builder.append_block_params_for_function_params(entry);

        // Start generating instructions in entry.
        builder.switch_to_block(entry);

        // Get x.
        let x = builder.block_params(entry)[0];

        // Create constant 0.
        let zero = builder.ins().iconst(types::I64, 0);

        // condition = x < 0
        let condition = builder.ins().icmp(
            IntCC::SignedLessThan,
            x,
            zero,
        );

        // if condition:
        //     goto negative
        // else:
        //     goto non_negative
        builder.ins().brif(
            condition,
            negative,
            &[],
            non_negative,
            &[],
        );

        // The entry block has now received all of its incoming control flow.
        builder.seal_block(entry);

        // --------------------------------------------------------
        // negative:
        //
        //     return -x
        // --------------------------------------------------------

        builder.switch_to_block(negative);

        let result = builder.ins().ineg(x);

        builder.ins().return_(&[result]);

        builder.seal_block(negative);

        // --------------------------------------------------------
        // non_negative:
        //
        //     return x
        // --------------------------------------------------------

        builder.switch_to_block(non_negative);

        builder.ins().return_(&[x]);

        builder.seal_block(non_negative);

        // Tell the frontend that we're done building this function.
        builder.finalize(target_config);
    }

    // Compile abs() to native machine code.
    module.define_function(abs_id, &mut abs_ctx)?;

    // We don't need the context anymore.
    module.clear_context(&mut abs_ctx);

    // ------------------------------------------------------------
    // 4. Declare:
    //
    //     fn add_abs(a: i64, b: i64) -> i64
    //
    // ------------------------------------------------------------

    let mut add_abs_signature = module.make_signature();

    add_abs_signature
        .params
        .push(AbiParam::new(types::I64));

    add_abs_signature
        .params
        .push(AbiParam::new(types::I64));

    add_abs_signature
        .returns
        .push(AbiParam::new(types::I64));

    let add_abs_id = module.declare_function(
        "add_abs",
        Linkage::Export,
        &add_abs_signature,
    )?;

    // ------------------------------------------------------------
    // 5. Generate add_abs().
    //
    //     return abs(a) + abs(b)
    // ------------------------------------------------------------

    let mut add_abs_ctx = module.make_context();

    add_abs_ctx.func.signature = add_abs_signature.clone();
    add_abs_ctx.func.name =
        UserFuncName::user(0, add_abs_id.as_u32());

    // Before creating the FunctionBuilder, create a reference
    // from this function to abs().
    let abs_ref =
        module.declare_func_in_func(abs_id, &mut add_abs_ctx.func);

    let mut add_abs_builder_ctx = FunctionBuilderContext::new();

    {
        let mut builder =
            FunctionBuilder::new(
                &mut add_abs_ctx.func,
                &mut add_abs_builder_ctx,
            );

        let entry = builder.create_block();

        builder.append_block_params_for_function_params(entry);

        builder.switch_to_block(entry);
        builder.seal_block(entry);

        // Get a and b.
        let params = builder.block_params(entry);

        let a = params[0];
        let b = params[1];

        // --------------------------------------------------------
        // Call abs(a).
        // --------------------------------------------------------

        let call_a = builder.ins().call(abs_ref, &[a]);

        let abs_a = builder.inst_results(call_a)[0];

        // --------------------------------------------------------
        // Call abs(b).
        // --------------------------------------------------------

        let call_b = builder.ins().call(abs_ref, &[b]);

        let abs_b = builder.inst_results(call_b)[0];

        // --------------------------------------------------------
        // abs(a) + abs(b)
        // --------------------------------------------------------

        let result = builder.ins().iadd(abs_a, abs_b);

        // Return it.
        builder.ins().return_(&[result]);

        builder.finalize(target_config);
    }

    // Compile add_abs().
    module.define_function(add_abs_id, &mut add_abs_ctx)?;

    module.clear_context(&mut add_abs_ctx);

    // ------------------------------------------------------------
    // 6. Finalize the entire JIT module.
    // ------------------------------------------------------------

    //
    // This performs the relocations needed to connect references
    // between generated functions and makes the memory executable.
    //
    module.finalize_definitions()?;

    // ------------------------------------------------------------
    // 7. Get the address of add_abs().
    // ------------------------------------------------------------

    let code_ptr = module.get_finalized_function(add_abs_id);

    // ------------------------------------------------------------
    // 8. Turn the raw pointer into a callable Rust function.
    // ------------------------------------------------------------

    let add_abs: extern "C" fn(i64, i64) -> i64 =
        unsafe {
            transmute(code_ptr)
        };

    // ------------------------------------------------------------
    // 9. CALL THE JITTED CODE.
    // ------------------------------------------------------------

    let result = add_abs(-20, 7);

    println!("add_abs(-20, 7) = {result}");

    Ok(())
}