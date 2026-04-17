use crate::{DbIndex, FileId, InferFailReason, LuaInferCache, LuaType};

use super::InferSessionRef;

pub(crate) fn with_module_export_type<T, F>(
    db: &DbIndex,
    cache: &LuaInferCache,
    file_id: FileId,
    f: F,
) -> Result<T, InferFailReason>
where
    F: FnOnce(&LuaType) -> Result<T, InferFailReason>,
{
    with_module_export_type_session(db, cache.get_infer_session(), file_id, f)
}

pub(crate) fn with_module_export_type_session<T, F>(
    db: &DbIndex,
    infer_session: &InferSessionRef,
    file_id: FileId,
    f: F,
) -> Result<T, InferFailReason>
where
    F: FnOnce(&LuaType) -> Result<T, InferFailReason>,
{
    let _scope = infer_session.enter(file_id)?;
    let module_info = db
        .get_module_index()
        .get_module(file_id)
        .ok_or(InferFailReason::FieldNotFound)?;
    let export_type = module_info
        .export_type
        .as_ref()
        .ok_or(InferFailReason::UnResolveModuleExport(file_id))?;

    f(export_type)
}
