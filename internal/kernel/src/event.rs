use crate::command::WidgetCommand;
use crate::Id;

/// 部品 (Widget) に発生するイベント
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WidgetEvent {
    /// 部品を新しく作成する
    WidgetCreated {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_name: String,
        /// 部品の説明
        widget_description: String,
    },
    /// 部品の名前を変更する
    WidgetNameChanged {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_name: String,
    },
    /// 部品の説明を変更する
    WidgetDescriptionChanged {
        /// イベントの id
        id: Id<WidgetEvent>,
        /// 部品の名前
        widget_description: String,
    },
}

impl From<WidgetCommand> for Vec<WidgetEvent> {
    fn from(value: WidgetCommand) -> Self {
        let id = Id::generate();
        match value {
            WidgetCommand::CreateWidget {
                widget_name,
                widget_description,
            } => vec![WidgetEvent::WidgetCreated {
                id,
                widget_name,
                widget_description,
            }],
            WidgetCommand::ChangeWidgetName { widget_name } => {
                vec![WidgetEvent::WidgetNameChanged { id, widget_name }]
            }
            WidgetCommand::ChangeWidgetDescription { widget_description } => {
                vec![WidgetEvent::WidgetDescriptionChanged {
                    id,
                    widget_description,
                }]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::command::WidgetCommand;
    use crate::event::WidgetEvent;

    #[test]
    /// コマンドからイベントに変換するときのテスト
    fn test_convert_command_to_event() {
        struct TestCase {
            name: &'static str,
            command: WidgetCommand,
            assert: fn(name: &'static str, command: WidgetCommand) -> (),
        }
        let tests = vec![
            TestCase {
                name: "部品作成コマンドの場合、部品作成イベントにのみ変換できる",
                command: WidgetCommand::CreateWidget {
                    widget_name: String::default(),
                    widget_description: String::default(),
                },
                assert: |name, command| {
                    let events: Vec<_> = command.into();
                    assert_eq!(events.len(), 1, "{name}");
                    assert!(
                        matches!(events[0], WidgetEvent::WidgetCreated { .. }),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "部品名変更コマンドの場合、部品名変更イベントにのみ変換できる",
                command: WidgetCommand::ChangeWidgetName {
                    widget_name: String::default(),
                },
                assert: |name, command| {
                    let events: Vec<_> = command.into();
                    assert_eq!(events.len(), 1, "{name}");
                    assert!(
                        matches!(events[0], WidgetEvent::WidgetNameChanged { .. }),
                        "{name}"
                    );
                },
            },
            TestCase {
                name: "部品の説明変更コマンドの場合、部品の説明変更イベントにのみ変換できる",
                command: WidgetCommand::ChangeWidgetDescription {
                    widget_description: String::default(),
                },
                assert: |name, command| {
                    let events: Vec<_> = command.into();
                    assert_eq!(events.len(), 1, "{name}");
                    assert!(
                        matches!(events[0], WidgetEvent::WidgetDescriptionChanged { .. }),
                        "{name}"
                    );
                },
            },
        ];

        for test in tests {
            (test.assert)(test.name, test.command);
        }
    }
}
