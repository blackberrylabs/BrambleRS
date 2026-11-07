//! BrambleRS desktop app (iced): one screen. A sentence, the four severities in words,
//! the six modules, the findings of every module in one list, and the detail of the selected
//! one with where it comes from and what the person could do. The same binary is the command
//! line when given arguments (`bramblers report --json`); it writes only its preferences and
//! the saved reports (BRIEF R2).
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::Arc;

use iced::widget::{
    button, checkbox, column, container, horizontal_space, mouse_area, row, scrollable, text,
    text_input, Space,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme};

use bramblers_billfold_view as billfold_view;
use bramblers_cleanator_view as cleanator_view;
use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard, ModuleStatus, Run, RunError};
use bramblers_lastrun_view as lastrun_view;
use bramblers_linstruct_view as linstruct_view;
use bramblers_mcproster_view as mcproster_view;
use bramblers_permitt_view as permitt_view;
use bramblers_report::{ActionKind, Finding, Severity, SubjectKind, Tone};
use bramblers_ui::palette::{alert, amber, blend, good, muted};
use bramblers_ui::widgets::{card, inner, small_button, swatch};
use bramblers_ui::{Lang, Prefs, Strings};

const EN: &str = include_str!("../../../locales/en.json");
const FR: &str = include_str!("../../../locales/fr.json");

fn main() {
    // One binary: arguments mean the command line, no arguments means the app.
    if std::env::args_os().len() > 1 {
        if let Err(e) = bramblers_cli::run(std::env::args()) {
            eprintln!("{e:#}");
            std::process::exit(1);
        }
        return;
    }
    if let Err(e) = run_app() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run_app() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
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
    /// A message from a module's detailed view.
    Permitt(permitt_view::Message),
    Mcproster(mcproster_view::Message),
    Linstruct(linstruct_view::Message),
    Cleanator(cleanator_view::Message),
    Billfold(billfold_view::Message),
    Lastrun(lastrun_view::Message),
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
    /// Built the first time the Permitt tab is opened, then kept.
    permitt: Option<permitt_view::View>,
    /// Same for MCProster.
    mcproster: Option<mcproster_view::View>,
    /// Same for Linstruct.
    linstruct: Option<linstruct_view::View>,
    /// Same for Cleanator.
    cleanator: Option<cleanator_view::View>,
    /// Same for Billfold.
    billfold: Option<billfold_view::View>,
    /// Same for Lastrun.
    lastrun: Option<lastrun_view::View>,
}

impl App {
    fn new() -> (App, Task<Message>) {
        let prefs: Prefs<()> = Prefs::load_with_fallback("bramblers", "blackberrylabs");
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
            permitt: None,
            mcproster: None,
            linstruct: None,
            cleanator: None,
            billfold: None,
            lastrun: None,
        };
        // `BRAMBLERS_MODULE=permitt` opens on a module tab (documentation screenshots);
        // `BRAMBLERS_SELECT=module:finding-id` preselects a finding; `BRAMBLERS_NO_REFRESH=1`
        // skips the start-up refresh. The old BBL_* names are still read for the archived
        // screenshots.
        if let Ok(m) = std::env::var("BRAMBLERS_MODULE") {
            if bramblers_core::registry::get(&m).is_some() {
                app.module_filter = Some(m);
            }
        }
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
        let mut tasks: Vec<Task<Message>> = Vec::new();
        // `BRAMBLERS_MODULE=permitt` opens the tab at once, even with the refresh off.
        if let Some(id) = app.module_filter.clone() {
            if let Some(t) = app.open_view(&id) {
                tasks.push(t);
            }
        }
        if no_refresh.is_none() {
            tasks.push(app.refresh_all());
        }
        (app, Task::batch(tasks))
    }

    /// Build a module's detailed view the first time its tab is opened.
    fn open_view(&mut self, id: &str) -> Option<Task<Message>> {
        match id {
            "permitt" if self.permitt.is_none() => {
                let (mut view, task) = permitt_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.permitt = Some(view);
                Some(task.map(Message::Permitt))
            }
            "mcproster" if self.mcproster.is_none() => {
                let (mut view, task) = mcproster_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.mcproster = Some(view);
                Some(task.map(Message::Mcproster))
            }
            "linstruct" if self.linstruct.is_none() => {
                let (mut view, task) = linstruct_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.linstruct = Some(view);
                Some(task.map(Message::Linstruct))
            }
            "cleanator" if self.cleanator.is_none() => {
                let (mut view, task) = cleanator_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.cleanator = Some(view);
                Some(task.map(Message::Cleanator))
            }
            "billfold" if self.billfold.is_none() => {
                let (mut view, task) = billfold_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.billfold = Some(view);
                Some(task.map(Message::Billfold))
            }
            "lastrun" if self.lastrun.is_none() => {
                let (mut view, task) = lastrun_view::View::new();
                view.set_lang(self.lang);
                view.set_dark(self.dark);
                self.lastrun = Some(view);
                Some(task.map(Message::Lastrun))
            }
            _ => None,
        }
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

    /// The Cleanator loading screen counts while its scan runs.
    fn subscription(&self) -> Subscription<Message> {
        match self.module_filter.as_deref() {
            Some("cleanator") => self
                .cleanator
                .as_ref()
                .map(|v| v.subscription().map(Message::Cleanator))
                .unwrap_or_else(Subscription::none),
            _ => Subscription::none(),
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
            Message::RefreshAll => {
                let mut tasks = vec![self.refresh_all()];
                // The header refreshes what is on screen, not only the reports behind the rail.
                match self.module_filter.as_deref() {
                    Some("permitt") => {
                        if let Some(v) = &mut self.permitt {
                            tasks.push(v.refresh().map(Message::Permitt));
                        }
                    }
                    Some("mcproster") => {
                        if let Some(v) = &mut self.mcproster {
                            tasks.push(v.refresh().map(Message::Mcproster));
                        }
                    }
                    Some("linstruct") => {
                        if let Some(v) = &mut self.linstruct {
                            tasks.push(v.refresh().map(Message::Linstruct));
                        }
                    }
                    Some("cleanator") => {
                        if let Some(v) = &mut self.cleanator {
                            tasks.push(v.refresh().map(Message::Cleanator));
                        }
                    }
                    Some("billfold") => {
                        if let Some(v) = &mut self.billfold {
                            tasks.push(v.refresh().map(Message::Billfold));
                        }
                    }
                    Some("lastrun") => {
                        if let Some(v) = &mut self.lastrun {
                            tasks.push(v.refresh().map(Message::Lastrun));
                        }
                    }
                    _ => {}
                }
                return Task::batch(tasks);
            }
            Message::Refresh(id) => return self.refresh(id),
            Message::SetLang(l) => {
                self.lang = l;
                self.strings = Strings::load(EN, FR, l);
                if let Some(v) = &mut self.permitt {
                    v.set_lang(l);
                }
                if let Some(v) = &mut self.mcproster {
                    v.set_lang(l);
                }
                if let Some(v) = &mut self.linstruct {
                    v.set_lang(l);
                }
                if let Some(v) = &mut self.cleanator {
                    v.set_lang(l);
                }
                if let Some(v) = &mut self.billfold {
                    v.set_lang(l);
                }
                if let Some(v) = &mut self.lastrun {
                    v.set_lang(l);
                }
                self.remember();
            }
            Message::ToggleTheme => {
                self.dark = !self.dark;
                if let Some(v) = &mut self.permitt {
                    v.set_dark(self.dark);
                }
                if let Some(v) = &mut self.mcproster {
                    v.set_dark(self.dark);
                }
                if let Some(v) = &mut self.linstruct {
                    v.set_dark(self.dark);
                }
                if let Some(v) = &mut self.cleanator {
                    v.set_dark(self.dark);
                }
                if let Some(v) = &mut self.billfold {
                    v.set_dark(self.dark);
                }
                if let Some(v) = &mut self.lastrun {
                    v.set_dark(self.dark);
                }
                self.remember();
            }
            Message::Search(s) => self.search = s,
            Message::FilterModule(m) => {
                if self.module_filter != m {
                    self.selected = None;
                    self.severity_filter = None;
                }
                let opened = m.as_deref().and_then(|id| self.open_view(id));
                self.module_filter = m;
                return opened.unwrap_or_else(Task::none);
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
            Message::Permitt(m) => {
                if let Some(v) = &mut self.permitt {
                    return v.update(m).map(Message::Permitt);
                }
            }
            Message::Mcproster(m) => {
                if let Some(v) = &mut self.mcproster {
                    return v.update(m).map(Message::Mcproster);
                }
            }
            Message::Linstruct(m) => {
                if let Some(v) = &mut self.linstruct {
                    return v.update(m).map(Message::Linstruct);
                }
            }
            Message::Cleanator(m) => {
                if let Some(v) = &mut self.cleanator {
                    return v.update(m).map(Message::Cleanator);
                }
            }
            Message::Billfold(m) => {
                if let Some(v) = &mut self.billfold {
                    return v.update(m).map(Message::Billfold);
                }
            }
            Message::Lastrun(m) => {
                if let Some(v) = &mut self.lastrun {
                    return v.update(m).map(Message::Lastrun);
                }
            }
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
        let content: Element<'_, Message> = match self.module_filter.as_deref() {
            Some("permitt") => match &self.permitt {
                Some(v) => self.view_module(v.view().map(Message::Permitt)),
                None => self.view_overview(),
            },
            Some("mcproster") => match &self.mcproster {
                Some(v) => self.view_module(v.view().map(Message::Mcproster)),
                None => self.view_overview(),
            },
            Some("linstruct") => match &self.linstruct {
                Some(v) => self.view_module(v.view().map(Message::Linstruct)),
                None => self.view_overview(),
            },
            Some("cleanator") => match &self.cleanator {
                Some(v) => self.view_module(v.view().map(Message::Cleanator)),
                None => self.view_overview(),
            },
            Some("billfold") => match &self.billfold {
                Some(v) => self.view_module(v.view().map(Message::Billfold)),
                None => self.view_overview(),
            },
            Some("lastrun") => match &self.lastrun {
                Some(v) => self.view_module(v.view().map(Message::Lastrun)),
                None => self.view_overview(),
            },
            _ => self.view_overview(),
        };
        column![
            self.view_header(),
            row![self.view_sidebar(), content]
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .into()
    }

    /// A module's own screen, under the shell's page head.
    fn view_module<'a>(&'a self, body: Element<'a, Message>) -> Element<'a, Message> {
        column![self.view_page_head(), body]
            .spacing(12)
            .padding(14)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The shared finding list, used by the overview and the tabs without a view yet.
    fn view_overview(&self) -> Element<'_, Message> {
        column![
            self.view_page_head(),
            self.view_severities(),
            row![self.view_list(), self.view_detail()]
                .spacing(12)
                .height(Length::Fill),
        ]
        .spacing(12)
        .padding(14)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let s = &self.strings;
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
        let accent = (if self.dark { Theme::Dark } else { Theme::Light })
            .extended_palette()
            .primary
            .base
            .color;
        container(
            row![
                swatch(accent, 9),
                text("BrambleRS").size(15),
                text(format!("v{}", env!("CARGO_PKG_VERSION")))
                    .size(10)
                    .style(text::secondary),
                horizontal_space(),
                text(self.when_line()).size(12).style(text::secondary),
                refresh,
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([9, 14])
        .width(Length::Fill)
        .style(header_style)
        .into()
    }

    fn view_sidebar(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let mut nav = column![].spacing(2);
        nav = nav.push(self.nav_group(s.t("nav.overview", &[])));
        nav = nav.push(self.nav_item(
            s.t("nav.all", &[]),
            None,
            self.dash.lines.len(),
            self.worst(None),
        ));
        nav = nav.push(self.nav_group(s.t("nav.modules", &[])));
        for m in &self.dash.modules {
            let n = self
                .dash
                .lines
                .iter()
                .filter(|l| l.module == m.provider.id)
                .count();
            nav = nav.push(self.nav_item(
                m.provider.name.to_string(),
                Some(m.provider.id),
                n,
                self.worst(Some(m.provider.id)),
            ));
        }
        let reports = self
            .dash
            .modules
            .iter()
            .filter(|m| m.report.is_some())
            .count();
        let status = s.t(
            "status.summary",
            &[
                ("modules", &self.dash.modules.len().to_string()),
                ("reports", &reports.to_string()),
            ],
        );
        let theme_label = if self.dark {
            self.t("theme.light")
        } else {
            self.t("theme.dark")
        };
        let foot = column![
            text(status).size(10).style(text::secondary),
            row![
                bramblers_ui::widgets::lang_picker(self.lang, Message::SetLang),
                horizontal_space(),
                button(text(theme_label).size(11))
                    .style(button::text)
                    .on_press(Message::ToggleTheme),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(8)
        .padding(iced::Padding {
            top: 10.0,
            ..Default::default()
        });
        container(column![scrollable(nav).height(Length::Fill), foot])
            .width(210)
            .height(Length::Fill)
            .padding(12)
            .style(sidebar_style)
            .into()
    }

    fn nav_group(&self, label: String) -> Element<'_, Message> {
        container(text(label.to_uppercase()).size(10).style(text::secondary))
            .padding(iced::Padding {
                top: 12.0,
                right: 9.0,
                bottom: 2.0,
                left: 9.0,
            })
            .into()
    }

    fn nav_item(
        &self,
        label: String,
        id: Option<&'static str>,
        count: usize,
        worst: Option<Severity>,
    ) -> Element<'_, Message> {
        let active = self.module_filter.as_deref() == id;
        let mut line = row![].spacing(8).align_y(Alignment::Center);
        match worst {
            Some(sev) => line = line.push(swatch(self.sev_color(sev), 7)),
            None => line = line.push(Space::with_width(7)),
        }
        let name = if active {
            text(label).size(13).style(text::primary)
        } else {
            text(label).size(13).style(text::secondary)
        };
        line = line.push(name);
        line = line.push(horizontal_space());
        line = line.push(text(count.to_string()).size(11).style(text::secondary));
        let cell =
            container(line)
                .padding([7, 9])
                .width(Length::Fill)
                .style(move |theme: &Theme| {
                    let p = theme.extended_palette();
                    container::Style {
                        background: if active {
                            Some(Background::Color(blend(
                                p.background.weak.color,
                                p.primary.base.color,
                                0.18,
                            )))
                        } else {
                            None
                        },
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                });
        mouse_area(cell)
            .on_press(Message::FilterModule(id.map(str::to_string)))
            .into()
    }

    fn view_page_head(&self) -> Element<'_, Message> {
        let s = &self.strings;
        match self.module_filter.as_deref() {
            Some(id) => {
                let (name, tagline) = self
                    .dash
                    .module(id)
                    .map(|m| (m.provider.name.to_string(), m.provider.tagline.to_string()))
                    .unwrap_or_else(|| (id.to_string(), String::new()));
                column![
                    text(name).size(22),
                    text(tagline).size(12).style(text::secondary),
                ]
                .spacing(2)
                .into()
            }
            None => column![text(s.t("nav.all", &[])).size(22), self.headline_sentence(),]
                .spacing(4)
                .into(),
        }
    }

    fn headline_sentence(&self) -> Element<'_, Message> {
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
        if !self.has_reports() {
            text(s.t("headline.none", &[])).size(14).into()
        } else if act == 0 && review == 0 {
            text(s.t("headline.clear", &[])).size(14).into()
        } else if act == 0 {
            text(s.t("headline.review", &[("review", &review.to_string())]))
                .size(14)
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
                        text(a.to_string()).size(14),
                        text(b.to_string()).size(14).color(alert(self.dark)),
                        text(c.to_string()).size(14),
                    ]
                    .into()
                }
                None => text(full).size(14).into(),
            }
        }
    }

    fn when_line(&self) -> String {
        let s = &self.strings;
        if !self.running.is_empty() {
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
        }
    }

    fn view_severities(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let counts = self.scoped_counts();
        let mut r = row![].spacing(12);
        for (sev, n) in counts {
            let k = sev.number().to_string();
            let value = if self.scope_has_reports() {
                n.to_string()
            } else {
                "n/a".to_string()
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

    /// The four severity counts for the current page: all modules, or the selected one.
    fn scoped_counts(&self) -> [(Severity, usize); 4] {
        let scope = self.module_filter.as_deref();
        Severity::ALL.map(|s| {
            (
                s,
                self.dash
                    .lines
                    .iter()
                    .filter(|l| l.finding.severity == s && scope.is_none_or(|m| l.module == m))
                    .count(),
            )
        })
    }

    fn scope_has_reports(&self) -> bool {
        match self.module_filter.as_deref() {
            Some(id) => self.dash.module(id).is_some_and(|m| m.report.is_some()),
            None => self.has_reports(),
        }
    }

    /// The most serious severity of a page, for the dot in the sidebar.
    fn worst(&self, module: Option<&str>) -> Option<Severity> {
        self.dash
            .lines
            .iter()
            .filter(|l| module.is_none_or(|m| l.module == m))
            .map(|l| l.finding.severity)
            .min()
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

        // Filters: search and the information toggle. The modules are the sidebar.
        let filters = row![
            text_input(&s.t("list.search", &[]), &self.search)
                .on_input(Message::Search)
                .size(12)
                .padding(5)
                .width(Length::Fill),
            checkbox(s.t("list.show_info", &[]), self.show_info)
                .on_toggle(Message::ToggleInfo)
                .size(14)
                .text_size(12),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let scope_has_lines = self
            .dash
            .lines
            .iter()
            .any(|l| self.module_filter.as_deref().is_none_or(|m| l.module == m));
        let body: Element<'_, Message> = if lines.is_empty() {
            let (title, sub) = if !self.scope_has_reports() {
                (
                    s.t("list.empty_none", &[]),
                    if self.module_filter.is_some() {
                        s.t("list.empty_none_sub_one", &[])
                    } else {
                        s.t("list.empty_none_sub", &[])
                    },
                )
            } else if !scope_has_lines {
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
            if !self.scope_has_reports() && self.running.is_empty() {
                let (label, msg) = match self.module_filter.as_deref() {
                    Some(id) => (s.t("refresh.one", &[]), Message::Refresh(id.to_string())),
                    None => (s.t("refresh.all", &[]), Message::RefreshAll),
                };
                c = c.push(
                    button(text(label).size(12))
                        .style(button::primary)
                        .on_press(msg),
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
                text(if self.module_filter.is_some() {
                    s.t("list.subtitle_one", &[])
                } else {
                    s.t("list.subtitle", &[])
                })
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
                    text(format!("{}:", s.t(&format!("sev.{k}"), &[]))).size(13),
                    text(s.t(&format!("sev.{k}.desc"), &[]))
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

/// The header bar: the panel background, a thin border.
fn header_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            width: 1.0,
            color: p.background.strong.color,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// The left rail: the panel background, a thin border.
fn sidebar_style(theme: &Theme) -> container::Style {
    let p = theme.extended_palette();
    container::Style {
        background: Some(Background::Color(p.background.weak.color)),
        border: Border {
            width: 1.0,
            color: p.background.strong.color,
            ..Default::default()
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
