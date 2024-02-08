use crate::event::WidgetEvent;
use crate::Id;

/// 部品 (Widget) の集約
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetAggregate {
    id: Id<WidgetAggregate>,
    name: String,
    description: String,
    version: u64,
}

impl WidgetAggregate {
    /// 部品の id
    pub fn id(&self) -> &Id<WidgetAggregate> {
        &self.id
    }

    /// 部品の名前
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 部品の説明
    pub fn description(&self) -> &str {
        &self.description
    }

    /// 集約のバージョン (= 更新回数)
    pub fn version(&self) -> u64 {
        self.version
    }
}

/// 集約 (Aggregate) に対するコマンドの処理を成功して保存可能になった状態
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetCommandState {
    widget_id: Id<WidgetAggregate>,
    events: Vec<WidgetEvent>,
    aggregate_version: u64,
}

impl WidgetCommandState {
    /// 部品の id
    pub fn widget_id(&self) -> &Id<WidgetAggregate> {
        &self.widget_id
    }

    /// 部品に対するコマンド
    pub fn events(&self) -> &[WidgetEvent] {
        &self.events
    }

    /// 集約のバージョン
    pub fn aggregate_version(&self) -> u64 {
        self.aggregate_version
    }
}
