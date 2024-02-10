use kernel::processor::CommandProcessor;
use lib::Result;

use crate::model::WidgetAggregateModel;
use crate::persistence::ConnectionPool;

pub struct WidgetRepository {
    pool: ConnectionPool,
}

impl CommandProcessor for WidgetRepository {
    async fn create_widget_aggregate(
        &self,
        command_state: kernel::aggregate::WidgetCommandState,
    ) -> Result<()> {
        let model: WidgetAggregateModel = command_state.try_into()?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO aggregate (widget_id, last_events, aggregate_version) VALUES (?, ?, ?)",
        )
        .bind(model.widget_id())
        .bind(model.last_events())
        .bind(model.aggregate_version())
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
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
