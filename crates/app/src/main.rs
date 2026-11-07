//! BrambleRS desktop app (iced): one screen — a sentence, the four severities in words,
//! the modules, the findings of every module in one list, the detail of the selected one
//! with where it comes from and what the person could do, and the button that opens the
//! module there. Reads the same core as `bramblers`; writes only its preferences and the saved
//! reports (BRIEF R2).
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::Arc;

use iced::widget::{
    button, checkbox, column, container, horizontal_space, mouse_area, row, scrollable, text,
    text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Task, Theme};

use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard, ModuleStatus, Run, RunError};
use bramblers_report::{ActionKind, Finding, Severity, SubjectKind, Tone};
use bramblers_ui::palette::{alert, amber, good, muted};
use bramblers_ui::widgets::{card, inner, small_button, swatch};
use bramblers_ui::{Lang, Prefs, Strings};

const EN: &str = include_str!("../../../locales/en.json");
const FR: &str = include_str!("../../../locales/fr.json");

fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .window_size((1280.0, 820.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Message {
    /// A module's run finished (on its thread).
    Refreshed(String, Result<Arc<Run>, RunError>),
    RefreshAll,
    Refresh(String),
    SetLang(Lang),
    ToggleTheme,
    Search(String),
    FilterModule(Option<String>),
    FilterSeverity(Severity),
    ToggleInfo(bool),
    Select(String, String),
    OpenFile(String),
    OpenUrl(String),
    Copy(String),
}

struct App {
    prefs: Prefs<()>,
    lang: Lang,
    strings: Strings,
    dark: bool,
    cache: Cache,
    dash: Dashboard,
    /// Modules whose run is in progress.
    running: BTreeSet<String>,
    search: String,
    module_filter: Option<String>,
    /// `Some(s)`: that severity only (a severity card was clicked).
    severity_filter: Option<Severity>,
    show_info: bool,
    /// `(module id, finding id)`.
    selected: Option<(String, String)>,
}

impl App {
    fn new() -> (App, Task<Message>) {
        let prefs: Prefs<()> = Prefs::load("bramblers");
        let lang = prefs.initial_lang();
        let dark = prefs.initial_dark();
        let strings = Strings::load(EN, FR, lang);
        let cache = Cache::default_location();
        let dash = Dashboard::from_saved(&cache);
        let mut app = App {
            prefs,
            lang,
            strings,
            dark,
            cache,
            dash,
            running: BTreeSet::new(),
            search: String::new(),
            module_filter: None,
            severity_filter: None,
            show_info: false,
            selected: None,
        };
        // `BRAMBLERS_SELECT=module:finding-id` preselects a finding (documentation
        // screenshots); `BRAMBLERS_NO_REFRESH=1` skips the start-up refresh. The old BBL_*
        // names are still read for the archived screenshots.
        let select = std::env::var("BRAMBLERS_SELECT").or_else(|_| std::env::var("BBL_SELECT"));
        if let Ok(v) = select {
            if let Some((m, f)) = v.split_once(':') {
                app.selected = Some((m.to_string(), f.to_string()));
            }
        }
        // T2: the saved reports are shown at once, every installed module is run again in
        // the background and replaces its report when ready.
        let no_refresh =
            std::env::var_os("BRAMBLERS_NO_REFRESH").or_else(|| std::env::var_os("BBL_NO_REFRESH"));
        let task = if no_refresh.is_some() {
            Task::none()
        } else {
            app.refresh_all()
        };
        (app, task)
    }

    fn title(&self) -> String {
        self.strings.t("app.title", &[])
    }

    fn theme(&self) -> Theme {
        if self.dark {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn t(&self, key: &str) -> String {
        self.strings.t(key, &[])
    }

    fn remember(&mut self) {
        self.prefs.remember(self.lang, self.dark);
        self.prefs.save();
    }

    fn refresh_all(&mut self) -> Task<Message> {
        let ids: Vec<String> = self
            .dash
            .modules
            .iter()
            .filter(|m| !self.running.contains(m.provider.id))
            .map(|m| m.provider.id.to_string())
            .collect();
        Task::batch(ids.into_iter().map(|id| self.refresh(id)))
    }

    fn refresh(&mut self, id: String) -> Task<Message> {
        if self.running.contains(&id) {
            return Task::none();
        }
        let Some(provider) = bramblers_core::registry::get(&id).copied() else {
            let r: Result<Run, RunError> = Err(RunError::Failed(format!("unknown module {id:?}")));
            self.dash.apply(&id, &r, &self.cache);
            return Task::none();
        };
        self.running.insert(id.clone());
        let (tx, rx) = iced::futures::channel::oneshot::channel();
        std::thread::spawn(move || {
            let r = bramblers_core::runner::report(&provider).map(Arc::new);
            let _ = tx.send(r);
        });
        Task::perform(
            async move {
                rx.await
                    .unwrap_or_else(|_| Err(RunError::Failed("the run thread stopped".into())))
            },
            move |r| Message::Refreshed(id.clone(), r),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refreshed(id, result) => {
                self.running.remove(&id);
                let r: Result<Run, RunError> = result.map(|a| (*a).clone());
                self.dash.apply(&id, &r, &self.cache);
                // The selected finding may have moved or gone.
                if let Some((m, f)) = &self.selected {
                    if !self
                        .dash
                        .lines
                        .iter()
                        .any(|l| &l.module == m && &l.finding.id == f)
                    {
                        self.selected = None;
                    }
                }
            }
            Message::RefreshAll => return self.refresh_all(),
            Message::Refresh(id) => return self.refresh(id),
            Message::SetLang(l) => {
                self.lang = l;
                self.strings = Strings::load(EN, FR, l);
                self.remember();
            }
            Message::ToggleTheme => {
                self.dark = !self.dark;
                self.remember();
            }
            Message::Search(s) => self.search = s,
            Message::FilterModule(m) => {
                if let (Some(m), Some((sm, _))) = (&m, &self.selected) {
                    if sm != m {
                        self.selected = None;
                    }
                }
                self.module_filter = m;
            }
            Message::FilterSeverity(s) => {
                self.severity_filter = if self.severity_filter == Some(s) {
                    None
                } else {
                    Some(s)
                };
                if s == Severity::Info && self.severity_filter.is_some() {
                    self.show_info = true;
                }
            }
            Message::ToggleInfo(b) => self.show_info = b,
            Message::Select(m, f) => self.selected = Some((m, f)),
            Message::OpenFile(p) | Message::OpenUrl(p) => {
                let _ = std::process::Command::new("xdg-open").arg(&p).spawn();
            }
            Message::Copy(s) => return iced::clipboard::write(s),
        }
        Task::none()
    }

    // ------------------------------------------------------------------ helpers

    fn age(&self, secs: u64) -> String {
        let s = &self.strings;
        if secs < 60 {
            s.t("age.now", &[])
        } else if secs < 3600 {
            s.t("age.min", &[("n", &(secs / 60).to_string())])
        } else if secs < 86_400 {
            s.t("age.h", &[("n", &(secs / 3600).to_string())])
        } else {
            s.t("age.d", &[("n", &(secs / 86_400).to_string())])
        }
    }

    fn sev_color(&self, s: Severity) -> Color {
        match s {
            Severity::ActNow => alert(self.dark),
            Severity::Review => amber(self.dark),
            Severity::GoodToKnow => good(self.dark),
            Severity::Info => muted(),
        }
    }

    fn tone_color(&self, t: Tone) -> Option<Color> {
        match t {
            Tone::Neutral => None,
            Tone::Good => Some(good(self.dark)),
            Tone::Amber => Some(amber(self.dark)),
            Tone::Alert => Some(alert(self.dark)),
        }
    }

    fn has_reports(&self) -> bool {
        self.dash.modules.iter().any(|m| m.report.is_some())
    }

    /// The lines after the filters, in order.
    fn visible(&self) -> Vec<&bramblers_core::Line> {
        let q = self.search.trim().to_lowercase();
        self.dash
            .lines
            .iter()
            .filter(|l| self.module_filter.as_deref().is_none_or(|m| l.module == m))
            .filter(|l| match self.severity_filter {
                Some(s) => l.finding.severity == s,
                None => self.show_info || l.finding.severity != Severity::Info,
            })
            .filter(|l| {
                q.is_empty()
                    || l.finding.title.to_lowercase().contains(&q)
                    || l.finding.subject.label.to_lowercase().contains(&q)
                    || l.finding.detail.to_lowercase().contains(&q)
                    || l.module_name.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn selected_line(&self) -> Option<&bramblers_core::Line> {
        let (m, f) = self.selected.as_ref()?;
        self.dash
            .lines
            .iter()
            .find(|l| &l.module == m && &l.finding.id == f)
    }

    // ------------------------------------------------------------------ view

    fn view(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let modules = self.dash.modules.len();
        let reports = self
            .dash
            .modules
            .iter()
            .filter(|m| m.report.is_some())
            .count();
        let status = s.t(
            "status.summary",
            &[
                ("modules", &modules.to_string()),
                ("reports", &reports.to_string()),
            ],
        );
        let theme_label = if self.dark {
            self.t("theme.light")
        } else {
            self.t("theme.dark")
        };
        let footer = bramblers_ui::widgets::footer(
            status,
            self.lang,
            Message::SetLang,
            theme_label,
            Message::ToggleTheme,
        );
        column![
            self.view_headline(),
            self.view_severities(),
            self.view_modules(),
            row![self.view_list(), self.view_detail()]
                .spacing(12)
                .height(Length::Fill),
            footer,
        ]
        .spacing(12)
        .padding(14)
        .into()
    }

    fn view_headline(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let counts = self.dash.counts();
        let (act, review) = (counts[0].1, counts[1].1);
        let things = |n: usize| {
            if n == 1 {
                s.t("headline.things_one", &[])
            } else {
                s.t("headline.things", &[("n", &n.to_string())])
            }
        };
        let sentence: Element<'_, Message> = if !self.has_reports() {
            text(s.t("headline.none", &[])).size(20).into()
        } else if act == 0 && review == 0 {
            text(s.t("headline.clear", &[])).size(20).into()
        } else if act == 0 {
            text(s.t("headline.review", &[("review", &review.to_string())]))
                .size(20)
                .into()
        } else {
            // The "act now" part in the alert colour: split the sentence around it.
            let full = s.t(
                "headline.act",
                &[("act", &things(act)), ("review", &review.to_string())],
            );
            let marker = things(act);
            match full.find(&marker) {
                Some(i) => {
                    let (a, rest) = full.split_at(i);
                    let (b, c) = rest.split_at(marker.len());
                    row![
                        text(a.to_string()).size(20),
                        text(b.to_string()).size(20).color(alert(self.dark)),
                        text(c.to_string()).size(20),
                    ]
                    .into()
                }
                None => text(full).size(20).into(),
            }
        };
        let when: String = if !self.running.is_empty() {
            let names: Vec<String> = self
                .running
                .iter()
                .filter_map(|id| self.dash.module(id).map(|m| m.provider.name.to_string()))
                .collect();
            s.t("when.refreshing", &[("list", &names.join(", "))])
        } else {
            let newest = self
                .dash
                .modules
                .iter()
                .filter_map(|m| match &m.state {
                    ReportState::Saved { age_s } => Some(*age_s),
                    ReportState::Fresh { .. } => Some(0),
                    _ => None,
                })
                .min();
            match newest {
                Some(a) => s.t("when.reports", &[("age", &self.age(a))]),
                None => s.t("when.none", &[]),
            }
        };
        let refresh: Element<'_, Message> = if self.running.is_empty() {
            button(text(s.t("refresh.all", &[])).size(12))
                .style(button::primary)
                .on_press(Message::RefreshAll)
                .into()
        } else {
            button(text(s.t("refresh.running", &[])).size(12))
                .style(button::secondary)
                .into()
        };
        row![
            sentence,
            horizontal_space(),
            text(when).size(12).style(text::secondary),
            refresh
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }

    fn view_severities(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let counts = self.dash.counts();
        let mut r = row![].spacing(12);
        for (sev, n) in counts {
            let k = sev.number().to_string();
            let value = if self.has_reports() {
                n.to_string()
            } else {
                "—".to_string()
            };
            let on = self.severity_filter == Some(sev);
            let color = self.sev_color(sev);
            let number: Element<'_, Message> = if sev <= Severity::Review && n > 0 {
                text(value).size(22).color(color).into()
            } else {
                text(value).size(22).into()
            };
            let tile = card(
                column![
                    row![swatch(color, 9), number]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    text(s.t(&format!("sev.{k}"), &[])).size(13),
                    text(s.t(&format!("sev.{k}.desc"), &[]))
                        .size(11)
                        .style(text::secondary),
                ]
                .spacing(2)
                .into(),
            )
            .width(Length::Fill)
            .height(Length::Fixed(104.0))
            .style(move |theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(Background::Color(p.background.weak.color)),
                    border: Border {
                        radius: 10.0.into(),
                        width: if on { 2.0 } else { 1.0 },
                        color: if on {
                            p.primary.base.color
                        } else {
                            p.background.strong.color
                        },
                    },
                    ..Default::default()
                }
            });
            r = r.push(mouse_area(tile).on_press(Message::FilterSeverity(sev)));
        }
        r.into()
    }

    fn view_modules(&self) -> Element<'_, Message> {
        let mut r = row![].spacing(12);
        for m in &self.dash.modules {
            r = r.push(self.view_module(m));
        }
        r.into()
    }

    fn view_module(&self, m: &ModuleStatus) -> Element<'_, Message> {
        let s = &self.strings;
        let running = self.running.contains(m.provider.id);
        // The card is a filter (T6): a click stays in BrambleRS, on that module.
        let head = row![text(m.provider.name).size(14), horizontal_space()]
            .align_y(Alignment::Center)
            .spacing(8);
        let mut col = column![
            head,
            text(m.provider.tagline).size(12).style(text::secondary)
        ]
        .spacing(3);
        // Figures, with the module's tones.
        if let Some(r) = &m.report {
            // The module's figures, two per line, with the module's tones.
            let mut grid = column![].spacing(1);
            for pair in r.summary.chunks(2) {
                let mut line = row![].spacing(10);
                for f in pair {
                    let value: Element<'_, Message> = match self.tone_color(f.tone) {
                        Some(c) => text(f.value.clone()).size(12).color(c).into(),
                        None => text(f.value.clone()).size(12).into(),
                    };
                    line = line.push(
                        row![text(f.label.clone()).size(11).style(text::secondary), value]
                            .spacing(4)
                            .align_y(Alignment::Center)
                            .width(Length::Fill),
                    );
                }
                grid = grid.push(line);
            }
            col = col.push(grid);
        } else if let ReportState::Failed { .. } = m.state {
        } else {
            col = col.push(
                text(s.t("module.no_report", &[]))
                    .size(12)
                    .style(text::secondary),
            );
        }
        let version = m
            .report
            .as_ref()
            .map(|r| r.module.version.clone())
            .unwrap_or_default();
        let state = if running {
            match &m.state {
                ReportState::Saved { age_s } => {
                    s.t("module.running", &[("age", &self.age(*age_s))])
                }
                _ => s.t("module.running_first", &[]),
            }
        } else {
            match &m.state {
                ReportState::Fresh { .. } => s.t("module.fresh", &[("version", &version)]),
                ReportState::Saved { age_s } => s.t(
                    "module.report_age",
                    &[("age", &self.age(*age_s)), ("version", &version)],
                ),
                ReportState::NoReport => s.t("module.no_report", &[]),
                ReportState::Failed { error } => {
                    s.t("module.failed", &[("error", &shorten(error, 70))])
                }
            }
        };
        let state_text = match &m.state {
            ReportState::Failed { .. } if !running => text(state)
                .size(12)
                .color(alert(self.dark))
                .width(Length::Fill),
            _ => text(state)
                .size(12)
                .style(text::secondary)
                .width(Length::Fill),
        };
        let action: Element<'_, Message> = if running {
            text(s.t("refresh.running", &[]))
                .size(11)
                .style(text::secondary)
                .into()
        } else {
            small_button(
                s.t("refresh.one", &[]),
                Message::Refresh(m.provider.id.to_string()),
            )
        };
        col = col.push(
            row![state_text, action]
                .spacing(8)
                .align_y(Alignment::Center),
        );
        self.module_card(m, card(col.into()).height(Length::Fixed(140.0)))
    }

    /// A module card is a filter: a click keeps you in BrambleRS, on that module (decision
    /// T6); a second click lifts the filter. The chosen card is outlined.
    fn module_card<'a>(
        &self,
        m: &ModuleStatus,
        c: container::Container<'a, Message>,
    ) -> Element<'a, Message> {
        let chosen = self.module_filter.as_deref() == Some(m.provider.id);
        let c = if chosen { c.style(chosen_card) } else { c };
        let next = if chosen {
            None
        } else {
            Some(m.provider.id.to_string())
        };
        mouse_area(c.width(Length::Fill))
            .on_press(Message::FilterModule(next))
            .into()
    }

    fn view_list(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let lines = self.visible();
        let hidden_info = if self.show_info || self.severity_filter.is_some() {
            0
        } else {
            self.dash
                .lines
                .iter()
                .filter(|l| l.finding.severity == Severity::Info)
                .filter(|l| self.module_filter.as_deref().is_none_or(|m| l.module == m))
                .count()
        };

        // Filters: search, module chips, information toggle.
        let mut chips = row![].spacing(6).align_y(Alignment::Center);
        let chip = |label: String, on: bool, msg: Message| -> Element<'_, Message> {
            let b = button(text(label).size(11)).padding([2, 9]);
            if on {
                b.style(button::primary).on_press(msg).into()
            } else {
                b.style(button::secondary).on_press(msg).into()
            }
        };
        chips = chips.push(chip(
            s.t("list.all", &[]),
            self.module_filter.is_none(),
            Message::FilterModule(None),
        ));
        for m in self.dash.modules.iter().filter(|m| m.report.is_some()) {
            chips = chips.push(chip(
                m.provider.name.to_string(),
                self.module_filter.as_deref() == Some(m.provider.id),
                Message::FilterModule(Some(m.provider.id.to_string())),
            ));
        }
        let filters = row![
            text_input(&s.t("list.search", &[]), &self.search)
                .on_input(Message::Search)
                .size(12)
                .padding(5)
                .width(Length::FillPortion(2)),
            chips,
            checkbox(s.t("list.show_info", &[]), self.show_info)
                .on_toggle(Message::ToggleInfo)
                .size(14)
                .text_size(12),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let body: Element<'_, Message> = if lines.is_empty() {
            let (title, sub) = if !self.has_reports() {
                (s.t("list.empty_none", &[]), s.t("list.empty_none_sub", &[]))
            } else if self.dash.lines.is_empty() {
                (s.t("list.empty_clear", &[]), String::new())
            } else {
                (s.t("list.empty_filter", &[]), String::new())
            };
            let mut c = column![
                text(title).size(15),
                text(sub).size(13).style(text::secondary)
            ]
            .spacing(6)
            .align_x(Alignment::Center)
            .max_width(460);
            if !self.has_reports() && self.running.is_empty() {
                c = c.push(
                    button(text(s.t("refresh.all", &[])).size(12))
                        .style(button::primary)
                        .on_press(Message::RefreshAll),
                );
            }
            container(c).center(Length::Fill).into()
        } else {
            let mut rows = column![].spacing(2);
            let mut current: Option<Severity> = None;
            for l in &lines {
                let sev = l.finding.severity;
                if current != Some(sev) {
                    current = Some(sev);
                    let n = lines.iter().filter(|x| x.finding.severity == sev).count();
                    rows = rows.push(
                        text(s.t(
                            "list.group",
                            &[
                                (
                                    "sev",
                                    &s.t(&format!("sev.{}", sev.number()), &[]).to_uppercase(),
                                ),
                                ("n", &n.to_string()),
                            ],
                        ))
                        .size(11)
                        .style(text::secondary),
                    );
                }
                rows = rows.push(self.view_row(l));
            }
            if hidden_info > 0 {
                rows = rows.push(
                    text(s.t("list.more_hidden", &[("n", &hidden_info.to_string())]))
                        .size(12)
                        .style(text::secondary),
                );
            }
            scrollable(rows.padding(iced::Padding {
                right: 14.0,
                ..Default::default()
            }))
            .height(Length::Fill)
            .into()
        };

        card(
            column![
                text(s.t("list.title", &[])).size(15),
                text(s.t("list.subtitle", &[]))
                    .size(12)
                    .style(text::secondary),
                filters,
                body,
            ]
            .spacing(8)
            .into(),
        )
        .width(Length::FillPortion(5))
        .height(Length::Fill)
        .into()
    }

    fn view_row(&self, l: &bramblers_core::Line) -> Element<'_, Message> {
        let selected = self
            .selected
            .as_ref()
            .is_some_and(|(m, f)| m == &l.module && f == &l.finding.id);
        let color = self.sev_color(l.finding.severity);
        let module_label = if selected {
            text(l.module_name.clone())
                .size(11)
                .width(Length::Fixed(88.0))
        } else {
            text(l.module_name.clone())
                .size(11)
                .style(text::secondary)
                .width(Length::Fixed(88.0))
        };
        let r = row![
            swatch(color, 9),
            module_label,
            text(l.finding.title.clone()).size(13),
            horizontal_space(),
            text("›").size(13).style(text::secondary),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let cell = container(r)
            .padding([6, 8])
            .width(Length::Fill)
            .style(move |theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: if selected {
                        Some(Background::Color(p.primary.weak.color))
                    } else {
                        None
                    },
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });
        mouse_area(cell)
            .on_press(Message::Select(l.module.clone(), l.finding.id.clone()))
            .into()
    }

    fn view_detail(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match (self.selected_line(), &self.module_filter) {
            (Some(l), _) => self.view_finding(l, &l.finding),
            (None, Some(id)) => match self.dash.module(id) {
                Some(m) => self.view_module_detail(m),
                None => self.view_about(),
            },
            (None, None) => self.view_about(),
        };
        card(
            scrollable(container(content).padding(iced::Padding {
                right: 14.0,
                ..Default::default()
            }))
            .height(Length::Fill)
            .into(),
        )
        .width(Length::FillPortion(4))
        .height(Length::Fill)
        .into()
    }

    fn view_finding(&self, l: &bramblers_core::Line, f: &Finding) -> Element<'_, Message> {
        let s = &self.strings;
        let sev = f.severity;
        let kind = s.t(&format!("subject.{}", subject_slug(f.subject.kind)), &[]);
        let mut col = column![
            bramblers_ui::widgets::pill(
                s.t(&format!("sev.{}", sev.number()), &[]).to_uppercase(),
                self.sev_color(sev)
            ),
            text(f.title.clone()).size(16),
            text(s.t(
                "detail.found_by",
                &[
                    ("module", &l.module_name),
                    ("kind", &kind),
                    ("label", &f.subject.label)
                ]
            ))
            .size(12)
            .style(text::secondary),
        ]
        .spacing(4);
        let heading = |k: &str| {
            text(s.t(k, &[]).to_uppercase())
                .size(11)
                .style(text::secondary)
        };
        if !f.detail.is_empty() {
            col = col.push(Space::with_height(8));
            col = col.push(heading("detail.means"));
            col = col.push(text(f.detail.clone()).size(13));
        }
        if !f.evidence.is_empty() {
            col = col.push(Space::with_height(8));
            col = col.push(heading("detail.where"));
            for e in &f.evidence {
                let mut block =
                    column![text(e.label.clone()).size(12).font(iced::Font::MONOSPACE)].spacing(4);
                if let Some(p) = &e.path {
                    block = block.push(
                        row![
                            small_button(
                                s.t("detail.open_file", &[]),
                                Message::OpenFile(p.clone())
                            ),
                            small_button(s.t("detail.copy_path", &[]), Message::Copy(p.clone())),
                        ]
                        .spacing(6),
                    );
                }
                col = col.push(inner(block.into()));
            }
        }
        if !f.actions.is_empty() {
            col = col.push(Space::with_height(8));
            col = col.push(heading("detail.do"));
            for a in &f.actions {
                let line: Element<'_, Message> = match (&a.kind, &a.target) {
                    (ActionKind::Hint, _) | (_, None) => text(a.label.clone()).size(13).into(),
                    (ActionKind::Command, Some(t)) => column![
                        text(a.label.clone()).size(13),
                        row![
                            text(t.clone()).size(12).font(iced::Font::MONOSPACE),
                            small_button(s.t("detail.copy_command", &[]), Message::Copy(t.clone()))
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(3)
                    .into(),
                    (ActionKind::OpenFile, Some(t)) => row![
                        text(a.label.clone()).size(13),
                        small_button(s.t("detail.open_file", &[]), Message::OpenFile(t.clone()))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into(),
                    (ActionKind::OpenUrl, Some(t)) => row![
                        text(a.label.clone()).size(13),
                        small_button(s.t("detail.open_link", &[]), Message::OpenUrl(t.clone()))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .into(),
                };
                col = col.push(row![text("→").size(13).style(text::secondary), line].spacing(8));
            }
        }
        col = col.push(Space::with_height(12));
        col = col.push(
            text(s.t("detail.note", &[]))
                .size(12)
                .style(text::secondary),
        );
        col.into()
    }

    /// The module in the pane: what it is, its figures, the state of its report, and its
    /// findings by severity.
    fn view_module_detail(&self, m: &ModuleStatus) -> Element<'_, Message> {
        let s = &self.strings;
        let mut col = column![
            text(m.provider.name).size(16),
            text(m.provider.tagline).size(12).style(text::secondary),
        ]
        .spacing(4);
        let what = if m.provider.description.is_empty() {
            m.provider.tagline
        } else {
            m.provider.description
        };
        col = col.push(Space::with_height(4));
        col = col.push(text(what).size(13));
        if let Some(r) = &m.report {
            col = col.push(Space::with_height(8));
            col = col.push(
                text(s.t("module.figures", &[]).to_uppercase())
                    .size(11)
                    .style(text::secondary),
            );
            for f in &r.summary {
                let value: Element<'_, Message> = match self.tone_color(f.tone) {
                    Some(c) => text(f.value.clone()).size(13).color(c).into(),
                    None => text(f.value.clone()).size(13).into(),
                };
                col = col.push(
                    row![
                        text(f.label.clone())
                            .size(12)
                            .style(text::secondary)
                            .width(Length::FillPortion(2)),
                        value
                    ]
                    .spacing(8),
                );
            }
            col = col.push(Space::with_height(8));
            col = col.push(
                text(s.t("module.findings", &[]).to_uppercase())
                    .size(11)
                    .style(text::secondary),
            );
            for sev in Severity::ALL {
                let n = self
                    .dash
                    .lines
                    .iter()
                    .filter(|l| l.module == m.provider.id && l.finding.severity == sev)
                    .count();
                if n > 0 {
                    col = col.push(
                        row![
                            swatch(self.sev_color(sev), 10),
                            text(s.t(&format!("sev.{}", sev.number()), &[])).size(12),
                            horizontal_space(),
                            text(n.to_string()).size(12)
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    );
                }
            }
        }
        let version = m
            .report
            .as_ref()
            .map(|r| r.module.version.clone())
            .unwrap_or_default();
        let state = match &m.state {
            ReportState::Fresh { .. } => s.t("module.fresh", &[("version", &version)]),
            ReportState::Saved { age_s } => s.t(
                "module.report_age",
                &[("age", &self.age(*age_s)), ("version", &version)],
            ),
            ReportState::NoReport => s.t("module.no_report", &[]),
            ReportState::Failed { error } => {
                s.t("module.failed", &[("error", &shorten(error, 120))])
            }
        };
        col = col.push(Space::with_height(8));
        col = col.push(text(state).size(12).style(text::secondary));
        let mut actions = row![].spacing(8);
        if self.running.contains(m.provider.id) {
            actions = actions.push(
                text(s.t("refresh.running", &[]))
                    .size(11)
                    .style(text::secondary),
            );
        } else {
            actions = actions.push(small_button(
                s.t("refresh.one", &[]),
                Message::Refresh(m.provider.id.to_string()),
            ));
        }
        col = col.push(Space::with_height(4));
        col = col.push(actions);
        col.into()
    }

    fn view_about(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let mut col = column![
            text(s.t("about.title", &[])).size(15),
            text(s.t("about.subtitle", &[]))
                .size(12)
                .style(text::secondary),
            Space::with_height(4),
            text(s.t("about.body", &[])).size(13),
            Space::with_height(8),
            text(s.t("about.how", &[]).to_uppercase())
                .size(11)
                .style(text::secondary),
        ]
        .spacing(4);
        for sev in Severity::ALL {
            let k = sev.number().to_string();
            col = col.push(
                row![
                    swatch(self.sev_color(sev), 9),
                    text(s.t(&format!("sev.{k}"), &[])).size(13),
                    text(format!("— {}", s.t(&format!("sev.{k}.desc"), &[])))
                        .size(13)
                        .style(text::secondary),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            );
        }
        col.into()
    }
}

/// The first `max` characters, with an ellipsis when something was cut.
fn shorten(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if out.len() < s.len() {
        out.push('…');
    }
    out
}

fn subject_slug(k: SubjectKind) -> &'static str {
    match k {
        SubjectKind::Host => "host",
        SubjectKind::Server => "server",
        SubjectKind::Path => "path",
        SubjectKind::File => "file",
        SubjectKind::Setting => "setting",
        SubjectKind::Other => "other",
    }
}

/// The module the tower is filtered on: the card with the accent border.
fn chosen_card(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            radius: 10.0.into(),
            width: 1.5,
            color: p.primary.base.color,
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locales_have_the_same_keys() {
        bramblers_ui::i18n::same_keys(EN, FR).unwrap();
    }

    #[test]
    fn shorten_keeps_short_text_and_marks_cuts() {
        assert_eq!(shorten("abc", 5), "abc");
        assert_eq!(shorten("abcdefgh", 5), "abcde…");
    }
}
