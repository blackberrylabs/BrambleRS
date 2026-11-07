//! The building blocks every Blackberrylabs window is made of, generic over the app's
//! message type. Paddings and radii are Cleanator's.

use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::i18n::Lang;
use crate::palette;

/// A panel: the weak background, rounded, with a thin border.
pub fn card<'a, M: 'a>(content: Element<'a, M>) -> container::Container<'a, M> {
    container(content).padding(14).style(|theme: &Theme| {
        let p = theme.extended_palette();
        container::Style {
            background: Some(Background::Color(p.background.weak.color)),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: p.background.strong.color,
            },
            ..Default::default()
        }
    })
}

/// A detail block inside a card: the page background, rounded.
pub fn inner<'a, M: 'a>(content: Element<'a, M>) -> Element<'a, M> {
    container(content)
        .padding(iced::Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        })
        .width(Length::Fill)
        .style(|theme: &Theme| {
            let p = theme.extended_palette();
            container::Style {
                background: Some(Background::Color(p.background.base.color)),
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// A small coloured square with rounded corners (a status dot).
pub fn swatch<'a, M: 'a>(color: Color, size: u16) -> Element<'a, M> {
    container(Space::new(size, size))
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: (size as f32 / 3.0).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// A coloured label; the text is black or white, whichever reads.
pub fn pill<'a, M: 'a>(label: impl Into<String>, color: Color) -> Element<'a, M> {
    container(
        text(label.into())
            .size(11)
            .color(palette::contrast_text(color)),
    )
    .padding([2, 8])
    .style(move |_: &Theme| container::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// A compact secondary button.
pub fn small_button<'a, M: Clone + 'a>(label: impl Into<String>, msg: M) -> Element<'a, M> {
    button(text(label.into()).size(11))
        .padding([2, 7])
        .style(button::secondary)
        .on_press(msg)
        .into()
}

/// A KPI card: a label (with an optional status dot), a large value, a small line under it.
pub fn kpi<'a, M: 'a>(
    label: impl Into<String>,
    value: impl Into<String>,
    sub: impl Into<String>,
    dot: Option<Color>,
) -> Element<'a, M> {
    let mut head = row![].spacing(6).align_y(Alignment::Center);
    if let Some(c) = dot {
        head = head.push(swatch(c, 8));
    }
    head = head.push(text(label.into()).size(12).style(text::secondary));
    card(
        iced::widget::column![
            head,
            text(value.into()).size(28),
            text(sub.into()).size(11).style(text::secondary),
        ]
        .spacing(2)
        .into(),
    )
    .width(Length::Fill)
    .into()
}

/// The EN · FR picker: the current language is the primary button, the others are text
/// buttons that send `on_pick(lang)`.
pub fn lang_picker<'a, M: Clone + 'a>(
    current: Lang,
    on_pick: impl Fn(Lang) -> M,
) -> Element<'a, M> {
    let mut r = row![].spacing(2).align_y(Alignment::Center);
    for lang in Lang::ALL {
        let b = button(text(lang.label()).size(11)).padding([2, 7]);
        r = r.push(if lang == current {
            Element::from(b.style(button::primary))
        } else {
            Element::from(b.style(button::text).on_press(on_pick(lang)))
        });
    }
    r.into()
}

/// The footer line: the status on the left; the language picker and the theme toggle on
/// the right.
pub fn footer<'a, M: Clone + 'a>(
    status: impl Into<String>,
    lang: Lang,
    on_lang: impl Fn(Lang) -> M,
    theme_label: impl Into<String>,
    on_theme: M,
) -> Element<'a, M> {
    row![
        text(status.into()).size(12).style(text::secondary),
        iced::widget::horizontal_space(),
        lang_picker(lang, on_lang),
        Space::with_width(6),
        button(text(theme_label.into()).size(12))
            .style(button::text)
            .on_press(on_theme),
    ]
    .align_y(Alignment::Center)
    .into()
}
