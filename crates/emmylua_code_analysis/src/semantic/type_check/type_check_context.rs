use std::collections::HashSet;

use crate::{DbIndex, LuaMemberKey};
use crate::semantic::{InferSession, InferSessionRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeCheckCheckLevel {
    Normal,
    GenericConditional,
}

#[derive(Debug, Clone)]
pub struct TypeCheckContext<'db> {
    pub detail: bool,
    pub db: &'db DbIndex,
    pub level: TypeCheckCheckLevel,
    pub infer_session: InferSessionRef,
    pub table_member_checked: Option<HashSet<LuaMemberKey>>,
}

impl<'db> TypeCheckContext<'db> {
    pub fn new(db: &'db DbIndex, detail: bool, level: TypeCheckCheckLevel) -> Self {
        Self::new_with_session(
            db,
            detail,
            level,
            InferSession::new(db.get_emmyrc().runtime.infer_reentry_limit),
        )
    }

    pub fn new_with_session(
        db: &'db DbIndex,
        detail: bool,
        level: TypeCheckCheckLevel,
        infer_session: InferSessionRef,
    ) -> Self {
        Self {
            detail,
            db,
            level,
            infer_session,
            table_member_checked: None,
        }
    }

    pub fn is_key_checked(&self, key: &LuaMemberKey) -> bool {
        if let Some(checked) = &self.table_member_checked {
            checked.contains(key)
        } else {
            false
        }
    }

    pub fn mark_key_checked(&mut self, key: LuaMemberKey) {
        if self.table_member_checked.is_none() {
            self.table_member_checked = Some(HashSet::new());
        }
        if let Some(checked) = &mut self.table_member_checked {
            checked.insert(key);
        }
    }
}
