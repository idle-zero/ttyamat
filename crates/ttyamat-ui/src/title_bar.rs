use iced::border::Radius;
use iced::widget::{button, container, responsive, row, space, text};
use iced::{Background, Border, Bottom, Center, Element, Fill, Length, Theme};

use crate::style;
use crate::tab::{Tab, TabId};

#[derive(Clone, Debug)]
pub(crate) enum Message {
    MinimizeWindow,
    ToggleMaximize,
    CloseWindow,
    TabPressed(TabId),
    TabClosePressed(TabId),
}

const TAB_MAX_WIDTH: f32 = 250.0;
const TAB_HEIGHT: f32 = 32.0;
const TITLE_BAR_HEIGHT: f32 = 40.0;
const CONTROL_WIDTH: f32 = 46.0;
const TAB_CLOSE_SLOT_WIDTH: f32 = 40.0;
const TAB_CLOSE_BUTTON_SIZE: f32 = 28.0;
const TAB_LEFT_INSET: f32 = 10.0;
const TAB_JOIN_RADIUS: f32 = 10.0;

pub(crate) fn view<'a>(tabs: &'a [Tab], active_tab: TabId) -> Element<'a, Message> {
    let tabs_region = responsive(move |size| {
        let available_width = (size.width - TAB_LEFT_INSET).max(0.0);
        let tab_width = match tabs.len() {
            0 => 0.0,
            count => (available_width / count as f32).min(TAB_MAX_WIDTH),
        };

        let tabs = tabs
            .iter()
            .map(|item| tab(item, item.id == active_tab, tab_width));
        let tabs_row = tabs.fold(row![active_tab_leading_edge()], |tabs_row, tab| {
            tabs_row.push(tab)
        });

        tabs_row
            .push(space::horizontal())
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

fn tab<'a>(tab: &'a Tab, is_active: bool, width: f32) -> Element<'a, Message> {
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
        .style(move |theme, status| tab_title_button_style(theme, status, is_active));
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

    container(
        row![title_region, close_slot]
            .width(Fill)
            .height(TAB_HEIGHT)
            .align_y(Center),
    )
    .width(width)
    .height(TAB_HEIGHT)
    .style(move |_| tab_style(is_active))
    .into()
}

fn centered_label(label: &'static str, size: f32) -> Element<'static, Message> {
    container(text(label).size(size)).center(Fill).into()
}

fn active_tab_leading_edge() -> Element<'static, Message> {
    let chrome_cutout = container(space::horizontal())
        .width(TAB_JOIN_RADIUS)
        .height(TAB_JOIN_RADIUS)
        .style(|_| {
            container::Style::default()
                .background(style::TITLE_BAR_BACKGROUND)
                .border(Border {
                    radius: Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_right: TAB_JOIN_RADIUS,
                        bottom_left: 0.0,
                    },
                    ..Border::default()
                })
        });

    let connector = container(chrome_cutout)
        .width(TAB_JOIN_RADIUS)
        .height(TAB_JOIN_RADIUS)
        .style(|_| container::Style::default().background(style::TERMINAL_BACKGROUND));

    container(connector)
        .width(TAB_LEFT_INSET)
        .height(TAB_HEIGHT)
        .align_y(Bottom)
        .into()
}

fn title_bar_style(_: &Theme) -> container::Style {
    container::Style::default().background(style::TITLE_BAR_BACKGROUND)
}

fn tab_style(is_active: bool) -> container::Style {
    let background = if is_active {
        style::TERMINAL_BACKGROUND
    } else {
        style::TITLE_BAR_BACKGROUND
    };

    container::Style::default()
        .background(background)
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

fn tab_title_button_style(_: &Theme, status: button::Status, is_active: bool) -> button::Style {
    let background =
        if !is_active && matches!(status, button::Status::Hovered | button::Status::Pressed) {
            Some(Background::Color(style::CONTROL_HOVER))
        } else {
            None
        };

    button::Style {
        background,
        text_color: style::PRIMARY_TEXT,
        border: Border::default(),
        ..button::Style::default()
    }
}
