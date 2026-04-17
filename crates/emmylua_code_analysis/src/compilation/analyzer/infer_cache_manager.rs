use hashbrown::HashMap;

use crate::{
    FileId, InferFailReason, LuaAnalysisPhase,
    semantic::{InferSession, InferSessionRef, InferSessionScope, LuaInferCache},
};

#[derive(Debug)]
pub struct InferCacheManager {
    infer_map: HashMap<FileId, LuaInferCache>,
    infer_session: InferSessionRef,
}

impl InferCacheManager {
    pub fn new(infer_reentry_limit: u32) -> Self {
        InferCacheManager {
            infer_map: HashMap::new(),
            infer_session: InferSession::new(infer_reentry_limit),
        }
    }

    pub fn get_infer_cache(&mut self, file_id: FileId) -> &mut LuaInferCache {
        self.infer_map.entry(file_id).or_insert_with(|| {
            LuaInferCache::new_with_session(
                file_id,
                crate::CacheOptions {
                    analysis_phase: LuaAnalysisPhase::Ordered,
                },
                self.infer_session.clone(),
            )
        })
    }

    pub fn enter_file(&self, file_id: FileId) -> Result<InferSessionScope, InferFailReason> {
        self.infer_session.enter(file_id)
    }

    pub fn set_force(&mut self) {
        for (_, infer_cache) in self.infer_map.iter_mut() {
            infer_cache.set_phase(LuaAnalysisPhase::Force);
        }
    }

    pub fn clear(&mut self) {
        for (_, infer_cache) in self.infer_map.iter_mut() {
            infer_cache.clear();
        }
        self.infer_session.clear();
    }
}

impl Default for InferCacheManager {
    fn default() -> Self {
        Self::new(2)
    }
}
