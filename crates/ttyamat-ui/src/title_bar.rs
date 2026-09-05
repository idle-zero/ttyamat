use iced::border::Radius;
use iced::widget::{button, container, mouse_area, responsive, row, space, text};
use iced::{Background, Border, Bottom, Center, Color, Element, Fill, Length, Theme};

use crate::style;
use crate::tab::{Tab, TabId};

#[derive(Clone, Debug)]
pub(crate) enum Message {
    MinimizeWindow,
    ToggleMaximize,
    CloseWindow,
    StartWindowDrag,
    TabPressed(TabId),
    TabClosePressed(TabId),
    TabHoverChanged { id: TabId, is_hovered: bool },
}

const TAB_MAX_WIDTH: f32 = 250.0;
const TAB_HEIGHT: f32 = 32.0;
const TITLE_BAR_HEIGHT: f32 = 40.0;
const CONTROL_WIDTH: f32 = 46.0;
const TAB_CLOSE_SLOT_WIDTH: f32 = 40.0;
const TAB_CLOSE_BUTTON_SIZE: f32 = 28.0;
const TAB_GROUP_LEFT_INSET: f32 = 10.0;
const TAB_GROUP_RIGHT_INSET: f32 = 50.0;
const TAB_SEPARATOR_WIDTH: f32 = 1.0;
const TAB_SEPARATOR_HEIGHT: f32 = 16.0;

pub(crate) fn view<'a>(
    tabs: &'a [Tab],
    active_tab: TabId,
    hovered_tab: Option<TabId>,
) -> Element<'a, Message> {
    let tabs_region = responsive(move |size| {
        let separator_count = tabs.len().saturating_sub(1);
        let separators_width = separator_count as f32 * TAB_SEPARATOR_WIDTH;
        let available_width =
            (size.width - TAB_GROUP_LEFT_INSET - TAB_GROUP_RIGHT_INSET - separators_width).max(0.0);
        let tab_width = match tabs.len() {
            0 => 0.0,
            count => (available_width / count as f32).min(TAB_MAX_WIDTH),
        };

        let leading_region = drag_region(Length::Fixed(TAB_GROUP_LEFT_INSET));

        let tabs_row =
            tabs.iter()
                .enumerate()
                .fold(row![leading_region], |tabs_row, (index, item)| {
                    let is_active = item.id == active_tab;
                    let is_hovered = hovered_tab == Some(item.id);
                    let tabs_row = tabs_row.push(tab(item, is_active, is_hovered, tab_width));

                    if let Some(next) = tabs.get(index + 1) {
                        let next_is_active = next.id == active_tab;
                        let next_is_hovered = hovered_tab == Some(next.id);
                        let show_separator =
                            !is_active && !is_hovered && !next_is_active && !next_is_hovered;

                        tabs_row.push(tab_separator(show_separator))
                    } else {
                        tabs_row
                    }
                });

        tabs_row
            .push(drag_region(Fill))
            .push(drag_region(Length::Fixed(TAB_GROUP_RIGHT_INSET)))
            .width(Fill)
            .height(TITLE_BAR_HEIGHT)
            .align_y(Bottom)
            .into()
    })
    .width(Fill)
    .height(TITLE_BAR_HEIGHT);

    let minimize = button(centered_label("—", 13.0))
        .on_press(Message::MinimizeWindow)
        .width(CONTROL_WIDTH)
        .height(TITLE_BAR_HEIGHT)
        .padding(0)
        .style(caption_button_style);
    let maximize = button(centered_label("□", 18.0))
        .on_press(Message::ToggleMaximize)
        .width(CONTROL_WIDTH)
        .height(TITLE_BAR_HEIGHT)
        .padding(0)
        .style(caption_button_style);
    let close = button(centered_label("×", 20.0))
        .on_press(Message::CloseWindow)
        .width(CONTROL_WIDTH)
        .height(TITLE_BAR_HEIGHT)
        .padding(0)
        .style(close_button_style);
    let controls = row![minimize, maximize, close]
        .width(Length::Shrink)
        .height(TITLE_BAR_HEIGHT)
        .align_y(Bottom);

    container(
        row![tabs_region, controls]
            .width(Fill)
            .height(TITLE_BAR_HEIGHT)
            .align_y(Center),
    )
    .width(Fill)
    .height(TITLE_BAR_HEIGHT)
    .style(title_bar_style)
    .into()
}

fn tab<'a>(tab: &'a Tab, is_active: bool, is_hovered: bool, width: f32) -> Element<'a, Message> {
    let icon = text(">_").size(13).color(style::ACCENT_BLUE);
    let title = text(&tab.title).size(14).color(style::PRIMARY_TEXT);
    let title_content = container(row![icon, title].spacing(10).align_y(Center).width(Fill))
        .width(Fill)
        .center_y(Fill);

    let title_region = button(title_content)
        .on_press_maybe((!is_active).then_some(Message::TabPressed(tab.id)))
        .width(Fill)
        .height(TAB_HEIGHT)
        .padding([0, 14])
        .style(tab_title_button_style);
    let close_button = button(centered_label("×", 18.0))
        .on_press(Message::TabClosePressed(tab.id))
        .width(TAB_CLOSE_BUTTON_SIZE)
        .height(TAB_CLOSE_BUTTON_SIZE)
        .padding(0)
        .style(tab_close_button_style);
    let close_slot = container(close_button)
        .width(TAB_CLOSE_SLOT_WIDTH)
        .height(TAB_HEIGHT)
        .align_x(Center)
        .align_y(Center);

    let tab_surface = container(
        row![title_region, close_slot]
            .width(Fill)
            .height(TAB_HEIGHT)
            .align_y(Center),
    )
    .width(width)
    .height(TAB_HEIGHT)
    .style(move |_| tab_style(is_active, is_hovered));

    mouse_area(tab_surface)
        .on_enter(Message::TabHoverChanged {
            id: tab.id,
            is_hovered: true,
        })
        .on_exit(Message::TabHoverChanged {
            id: tab.id,
            is_hovered: false,
        })
        .into()
}

fn centered_label(label: &'static str, size: f32) -> Element<'static, Message> {
    container(text(label).size(size)).center(Fill).into()
}

fn tab_separator(visible: bool) -> Element<'static, Message> {
    let color = if visible {
        style::TAB_SEPARATOR
    } else {
        Color::TRANSPARENT
    };
    let separator = container(space::vertical())
        .width(TAB_SEPARATOR_WIDTH)
        .height(TAB_SEPARATOR_HEIGHT)
        .style(move |_| container::Style::default().background(color));

    container(separator)
        .width(TAB_SEPARATOR_WIDTH)
        .height(TAB_HEIGHT)
        .align_y(Center)
        .into()
}

fn drag_region(width: Length) -> Element<'static, Message> {
    mouse_area(space::horizontal().width(width).height(TITLE_BAR_HEIGHT))
        .on_press(Message::StartWindowDrag)
        .on_double_click(Message::ToggleMaximize)
        .into()
}

fn title_bar_style(_: &Theme) -> container::Style {
    container::Style::default().background(style::TITLE_BAR_BACKGROUND)
}

fn tab_style(is_active: bool, is_hovered: bool) -> container::Style {
    container::Style::default()
        .background(tab_background(is_active, is_hovered))
        .border(Border {
            radius: Radius {
                top_left: 8.0,
                top_right: 8.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            ..Border::default()
        })
}

fn tab_background(is_active: bool, is_hovered: bool) -> Color {
    if is_active {
        style::TERMINAL_BACKGROUND
    } else if is_hovered {
        style::TAB_HOVER_BACKGROUND
    } else {
        style::TITLE_BAR_BACKGROUND
    }
}

fn tab_close_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = matches!(status, button::Status::Hovered | button::Status::Pressed)
        .then_some(Background::Color(style::CONTROL_HOVER));

    button::Style {
        background,
        text_color: style::SECONDARY_TEXT,
        border: Border {
            radius: 5.0.into(),
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn caption_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = matches!(status, button::Status::Hovered | button::Status::Pressed)
        .then_some(Background::Color(style::CONTROL_HOVER));

    button::Style {
        background,
        text_color: style::PRIMARY_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}

fn close_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = matches!(status, button::Status::Hovered | button::Status::Pressed)
        .then_some(Background::Color(style::CLOSE_HOVER));

    button::Style {
        background,
        text_color: style::PRIMARY_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}

fn tab_title_button_style(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: style::PRIMARY_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}
