use std::sync::{Arc, RwLock};

use rand::Rng;

use super::RepositoryError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId([u8; 32]);

impl SessionId {
    fn new() -> Self {
        Self(rand::thread_rng().gen::<[u8; 32]>())
    }
}

pub trait SessionData: Clone + std::marker::Sync + std::marker::Send + 'static {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session<D> {
    pub id: SessionId,
    pub data: D,
}
impl<D> Session<D> {
    pub fn new(data: D) -> Self {
        Self {
            id: SessionId::new(),
            data,
        }
    }
}

pub trait SessionRepository<D>: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn add(&self, data: D) -> Session<D>;
    fn get(&self, sessid: SessionId) -> Option<Session<D>>;
    fn update(&self, sessid:SessionId, data:D) -> Result<Session<D>, RepositoryError>;
    fn delete(&self, sessid: SessionId);
    fn clear(&self);
}

#[derive(Debug, Clone)]
pub struct SingletonSessionRepositoryForMemory<D> {
    store: Arc<RwLock<Option<Session<D>>>>,
}

impl<D> SingletonSessionRepositoryForMemory<D>
where
    D: SessionData,
{
    fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(None)),
        }
    }
}

impl<D> SessionRepository<D> for SingletonSessionRepositoryForMemory<D>
where
    D: SessionData,
{
    fn add(&self, data: D) -> Session<D> {
        let sess = Session::new(data);
        let id = sess.id;
        let mut store = self.store.write().unwrap();
        *store = Some(sess.clone());
        sess
    }

    fn get(&self, sessid: SessionId) -> Option<Session<D>> {
        let store = self.store.read().unwrap();
        if let Some(sess) = store.clone() {
            if sess.id == sessid {
                return Some(sess);
            }
        }
        None
    }

    fn update(&self, sessid:SessionId, data:D) -> Result<Session<D>, RepositoryError> {
        let mut store = self.store.write().unwrap();
        if let Some(mut sess) = store.clone() {
            if sess.id == sessid {
                sess.data = data;
                return Ok(sess);
            } 
        }
        Err(RepositoryError::NotFound)
    }

    fn delete(&self, sessid: SessionId) {
        let mut store = self.store.write().unwrap();
        if let Some(sess) = store.clone() {
            if sess.id == sessid {
                *store = None;
            }
        }
    }

    fn clear(&self) {
        let mut store = self.store.write().unwrap();
        *store = None;
    }
    

}

#[cfg(test)]
mod test {
    use crate::repositories::RepositoryError;

    use super::{SessionData, SessionRepository, SingletonSessionRepositoryForMemory};

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SessionDataTest {
        value: isize,
    }
    impl SessionDataTest {
        fn new(value: isize) -> Self {
            Self { value }
        }
    }
    impl SessionData for SessionDataTest {}

    #[test]
    fn singleton_repo_crud_scenario() {
        let sess_data = SessionDataTest::new(35);
        let sess_repo = SingletonSessionRepositoryForMemory::<SessionDataTest>::new();
        // get()
        let sess1_id = sess_repo.add(sess_data.clone()).id;
        assert_eq!(sess_repo.get(sess1_id).unwrap().data.value, 35);
        // delete()
        sess_repo.delete(sess1_id);
        sess_repo.delete(sess1_id); // call twice
        assert_eq!(sess_repo.get(sess1_id), None);
        // clear()
        let sess_data = SessionDataTest::new(45);
        sess_repo.add(sess_data);
        sess_repo.clear();
        sess_repo.clear(); // call twice
        assert_eq!(sess_repo.get(sess1_id), None);
        // add() singleton check
        let sess_data = SessionDataTest::new(55);
        let sess3_id = sess_repo.add(sess_data).id;
        let sess_data = SessionDataTest::new(65);
        let sess4_id = sess_repo.add(sess_data).id;
        assert_eq!(sess_repo.get(sess3_id), None);
        assert_eq!(sess_repo.get(sess4_id).unwrap().data.value, 65);
        // update()
        let result = sess_repo.update(sess4_id, SessionDataTest::new(12));
        assert_eq!(result.unwrap().data.value, 12);
        let result = sess_repo.update(sess1_id, SessionDataTest::new(12));
        assert_eq!(result, Err(RepositoryError::NotFound));
    }
}
