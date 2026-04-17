mod cache_options;

pub use cache_options::{CacheOptions, LuaAnalysisPhase};
use emmylua_parser::LuaSyntaxId;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    FileId, FlowId, LuaFunctionType,
    db_index::LuaType,
    semantic::infer::{ConditionFlowAction, VarRefId},
};
use crate::semantic::{InferSession, InferSessionRef, InferSessionScope};
use crate::InferFailReason;

#[derive(Debug)]
pub enum CacheEntry<T> {
    Ready,
    Cache(T),
}

#[derive(Debug)]
pub struct LuaInferCache {
    file_id: FileId,
    config: CacheOptions,
    infer_session: InferSessionRef,
    pub expr_cache: HashMap<LuaSyntaxId, CacheEntry<LuaType>>,
    pub call_cache:
        HashMap<(LuaSyntaxId, Option<usize>, LuaType), CacheEntry<Arc<LuaFunctionType>>>,
    pub(crate) flow_node_cache: HashMap<(VarRefId, FlowId, bool), CacheEntry<LuaType>>,
    pub(in crate::semantic) condition_flow_cache:
        HashMap<(VarRefId, FlowId, bool), CacheEntry<ConditionFlowAction>>,
    pub index_ref_origin_type_cache: HashMap<VarRefId, CacheEntry<LuaType>>,
    pub expr_var_ref_id_cache: HashMap<LuaSyntaxId, VarRefId>,
    pub narrow_by_literal_stop_position_cache: HashSet<LuaSyntaxId>,
}

impl LuaInferCache {
    pub fn new(file_id: FileId, config: CacheOptions, infer_reentry_limit: u32) -> Self {
        Self::new_with_session(file_id, config, InferSession::new(infer_reentry_limit))
    }

    pub fn new_with_session(
        file_id: FileId,
        config: CacheOptions,
        infer_session: InferSessionRef,
    ) -> Self {
        Self {
            file_id,
            config,
            infer_session,
            expr_cache: HashMap::new(),
            call_cache: HashMap::new(),
            flow_node_cache: HashMap::new(),
            condition_flow_cache: HashMap::new(),
            index_ref_origin_type_cache: HashMap::new(),
            expr_var_ref_id_cache: HashMap::new(),
            narrow_by_literal_stop_position_cache: HashSet::new(),
        }
    }

    pub fn get_config(&self) -> &CacheOptions {
        &self.config
    }

    pub fn get_file_id(&self) -> FileId {
        self.file_id
    }

    pub fn get_infer_session(&self) -> &InferSessionRef {
        &self.infer_session
    }

    pub fn enter_file_scope(&self, file_id: FileId) -> Result<InferSessionScope, InferFailReason> {
        self.infer_session.enter(file_id)
    }

    pub fn set_phase(&mut self, phase: LuaAnalysisPhase) {
        self.config.analysis_phase = phase;
    }

    pub fn clear(&mut self) {
        self.expr_cache.clear();
        self.call_cache.clear();
        self.flow_node_cache.clear();
        self.condition_flow_cache.clear();
        self.index_ref_origin_type_cache.clear();
        self.expr_var_ref_id_cache.clear();
    }
}
