use deadpool_sqlite::Pool;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

impl AppState {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}
