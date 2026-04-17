use emmylua_parser::LuaCallExpr;

use crate::{
    DbIndex, InferFailReason, LuaInferCache, LuaType, infer_expr,
    semantic::infer::InferResult,
    semantic::with_module_export_type,
};

pub fn infer_require_call(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    call_expr: LuaCallExpr,
) -> InferResult {
    let arg_list = call_expr.get_args_list().ok_or(InferFailReason::None)?;
    let first_arg = arg_list.get_args().next().ok_or(InferFailReason::None)?;
    let require_path_type = infer_expr(db, cache, first_arg)?;
    let module_path: String = match &require_path_type {
        LuaType::StringConst(module_path) => module_path.as_ref().to_string(),
        _ => {
            return Ok(LuaType::Any);
        }
    };

    let module_info = db
        .get_module_index()
        .find_module(&module_path)
        .ok_or(InferFailReason::None)?;
    with_module_export_type(db, cache, module_info.file_id, |ty| {
        Ok(match ty {
            LuaType::Def(id) => Ok(LuaType::Ref(id.clone())),
            _ => Ok(ty.clone()),
        }?)
    })
}
