use std::sync::{Arc, Mutex, MutexGuard};

use hashbrown::HashMap;

use crate::{FileId, InferFailReason};

pub type InferSessionRef = Arc<InferSession>;

#[derive(Debug)]
pub struct InferSession {
    reentry_limit: u32,
    state: Mutex<InferSessionState>,
}

#[derive(Debug, Default)]
struct InferSessionState {
    stack: Vec<FileId>,
    enter_count: HashMap<FileId, u32>,
}

#[derive(Debug)]
pub struct InferSessionScope {
    session: InferSessionRef,
    file_id: FileId,
    active: bool,
}

impl InferSession {
    pub fn new(reentry_limit: u32) -> InferSessionRef {
        Arc::new(Self {
            reentry_limit,
            state: Mutex::new(InferSessionState::default()),
        })
    }

    pub fn enter(self: &Arc<Self>, file_id: FileId) -> Result<InferSessionScope, InferFailReason> {
        if self.reentry_limit == 0 {
            return Ok(InferSessionScope {
                session: Arc::clone(self),
                file_id,
                active: false,
            });
        }

        let mut state = self.lock_state();
        if state.stack.last().copied() == Some(file_id) {
            return Ok(InferSessionScope {
                session: Arc::clone(self),
                file_id,
                active: false,
            });
        }

        let current_count = state.enter_count.get(&file_id).copied().unwrap_or(0);
        if current_count >= self.reentry_limit {
            return Err(InferFailReason::RecursiveInfer);
        }

        state.stack.push(file_id);
        state.enter_count.insert(file_id, current_count + 1);

        Ok(InferSessionScope {
            session: Arc::clone(self),
            file_id,
            active: true,
        })
    }

    pub fn clear(&self) {
        let mut state = self.lock_state();
        state.stack.clear();
        state.enter_count.clear();
    }

    fn exit(&self, file_id: FileId) {
        let mut state = self.lock_state();
        if state.stack.last().copied() == Some(file_id) {
            state.stack.pop();
        } else if let Some(index) = state.stack.iter().rposition(|id| *id == file_id) {
            state.stack.remove(index);
        }

        if let Some(count) = state.enter_count.get_mut(&file_id) {
            if *count <= 1 {
                state.enter_count.remove(&file_id);
            } else {
                *count -= 1;
            }
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, InferSessionState> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl Drop for InferSessionScope {
    fn drop(&mut self) {
        if self.active {
            self.session.exit(self.file_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_limit_disables_reentry_guard() {
        let session = InferSession::new(0);
        let _first = session.enter(FileId::new(1)).unwrap();
        let _second = session.enter(FileId::new(1)).unwrap();
        let _third = session.enter(FileId::new(1)).unwrap();
    }

    #[test]
    fn test_scope_drop_restores_file_count() {
        let session = InferSession::new(1);

        {
            let _scope = session.enter(FileId::new(1)).unwrap();
        }

        assert!(session.enter(FileId::new(1)).is_ok());
    }

    #[test]
    fn test_same_stack_top_reentry_does_not_increment_count() {
        let session = InferSession::new(1);
        let _first = session.enter(FileId::new(1)).unwrap();
        let _same_top = session.enter(FileId::new(1)).unwrap();

        let _other = session.enter(FileId::new(2)).unwrap();
        assert_eq!(
            session.enter(FileId::new(1)).unwrap_err(),
            InferFailReason::RecursiveInfer
        );
    }

    #[test]
    fn test_cross_file_reentry_limit_two_allows_a_b_a_and_blocks_next_a() {
        let session = InferSession::new(2);
        let _a1 = session.enter(FileId::new(1)).unwrap();
        let _b1 = session.enter(FileId::new(2)).unwrap();
        let _a2 = session.enter(FileId::new(1)).unwrap();
        let _b2 = session.enter(FileId::new(2)).unwrap();

        assert_eq!(
            session.enter(FileId::new(1)).unwrap_err(),
            InferFailReason::RecursiveInfer
        );
    }

    #[test]
    fn test_cross_file_reentry_limit_three_allows_one_more_roundtrip() {
        let session = InferSession::new(3);
        let _a1 = session.enter(FileId::new(1)).unwrap();
        let _b1 = session.enter(FileId::new(2)).unwrap();
        let _a2 = session.enter(FileId::new(1)).unwrap();
        let _b2 = session.enter(FileId::new(2)).unwrap();
        let _a3 = session.enter(FileId::new(1)).unwrap();
        let _b3 = session.enter(FileId::new(2)).unwrap();

        assert_eq!(
            session.enter(FileId::new(1)).unwrap_err(),
            InferFailReason::RecursiveInfer
        );
    }
}
