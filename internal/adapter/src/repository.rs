use kernel::processor::CommandProcessor;
use lib::Result;

use crate::persistence::ConnectionPool;

pub struct WidgetRepository {
    pool: ConnectionPool,
}

impl CommandProcessor for WidgetRepository {
    async fn create_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<()> {
        todo!()
    }

    async fn get_widget_aggregate(
        &self,
        widget_id: kernel::Id<kernel::aggregate::WidgetAggregate>,
    ) -> Result<kernel::aggregate::WidgetAggregate> {
        todo!()
    }

    async fn update_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<()> {
        todo!()
    }
}
