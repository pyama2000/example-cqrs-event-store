use kernel::{Aggregate, Id, Item, KernelError, QueryProcessor, Restaurant};

#[derive(Debug, Clone)]
pub struct QueryRepository;

impl QueryProcessor for QueryRepository {
    async fn list_restaurants(&self) -> Result<Vec<Restaurant>, KernelError> {
        todo!()
    }

    async fn list_items(&self, _aggregate_id: Id<Aggregate>) -> Result<Vec<Item>, KernelError> {
        todo!()
    }
}
