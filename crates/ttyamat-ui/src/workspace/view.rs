use iced::widget::{column, container, text};
use iced::{Element, Fill};

use crate::style;
use crate::terminal::metrics::CellMetrics;

use super::{Message, tabs::Tabs, title_bar};

pub(super) fn view(tabs: &Tabs, metrics: CellMetrics) -> Element<'_, Message> {
    let title_bar =
        title_bar::view(tabs.items(), tabs.active_id(), tabs.hovered_id()).map(Message::TitleBar);
    let content = match tabs.active_tab() {
        Some(tab) => {
            let id = tab.id();
            tab.view(metrics)
                .map(move |message| Message::Terminal(id, message))
        }
        None => container(text("No terminal session. Use + to open a tab."))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .style(|_| {
                container::Style::default()
                    .background(style::TERMINAL_BACKGROUND)
                    .color(style::PRIMARY_TEXT)
            })
            .into(),
    };

    column![title_bar, content].width(Fill).height(Fill).into()
}
