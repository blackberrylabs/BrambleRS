//! Permitt's view in BrambleRS: one screen for the sheet of hosts × capabilities, the detail
//! of the selected cell with the evidence behind its state, and the findings underneath.
//! It calls `bramblers-permitt-core` and reads no file itself; it writes only its own
//! settings (the shell owns the window, the language and the theme).
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, mouse_area, pick_list,
    row, scrollable, text, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Task, Theme};

use bramblers_hostrules::SourceKind;
use bramblers_permitt_core::model::{
    Capability, CapabilityState, Evidence, Finding, FindingKind, HostSheet, Level, State,
};
use bramblers_permitt_core::rules::ShellRules;
use bramblers_permitt_core::{inventory, sheet, Env, Inventory, Options, Rules};

use bramblers_ui::palette::{alert, amber, good, muted as unknown, neutral};
use bramblers_ui::widgets::{card, inner, pill, small_button};
use bramblers_ui::{Lang, Prefs, Strings};

const EN: &str = include_str!("../../locales/en.json");
const FR: &str = include_str!("../../locales/fr.json");

/// Permitt's own preference; the language and the theme belong to the shell.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Extra {
    /// `false` to leave `~/.bashrc` and friends unread (BRIEF R4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    read_shell: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Result<Arc<Inventory>, String>),
    Rescan,
    Filter(String),
    Scope(String),
    ToggleInstalledOnly(bool),
    ToggleFindingsOnly(bool),
    ToggleReadShell(bool),
    SelectCell(String, Capability),
    SelectFinding(usize),
    Copy(String),
    OpenFile(PathBuf),
    OpenUrl(String),
}

pub struct View {
    prefs: Prefs<Extra>,
    lang: Lang,
    strings: Strings,
    dark: bool,
    env: Env,
    rules: Arc<Rules>,
    shell_rules: Arc<ShellRules>,
    inventory: Option<Arc<Inventory>>,
    loading: bool,
    error: Option<String>,
    filter: String,
    /// `None`: user level; `Some(root)`: that project.
    scope: Option<PathBuf>,
    installed_only: bool,
    findings_only: bool,
    read_shell: bool,
    selected: Option<(String, Capability)>,
    status: String,
}

impl View {
    pub fn new() -> (View, Task<Message>) {
        // The old app's `~/.config/permitt/settings.json` is read once, then this file is
        // the view's own under the shell's directory.
        let prefs: Prefs<Extra> = Prefs::load_with_fallback("bramblers/permitt", "permitt");
        let lang = prefs.initial_lang();
        let dark = prefs.initial_dark();
        let strings = Strings::load(EN, FR, lang);
        let env = Env::current();
        let rules = Arc::new(bramblers_permitt_core::rules::embedded());
        let shell_rules = Arc::new(bramblers_permitt_core::rules::embedded_shell());
        let read_shell = prefs.extra.read_shell.unwrap_or(true);
        let status = strings.t("header.scanning", &[]);
        let task = load(rules.clone(), shell_rules.clone(), env.clone(), read_shell);
        let view = View {
            prefs,
            lang,
            strings,
            dark,
            env,
            rules,
            shell_rules,
            inventory: None,
            loading: true,
            error: None,
            filter: String::new(),
            scope: None,
            installed_only: true,
            findings_only: false,
            read_shell,
            selected: None,
            status,
        };
        (view, task)
    }

    /// The shell's language, pushed in; the view never owns it.
    pub fn set_lang(&mut self, lang: Lang) {
        self.lang = lang;
        self.strings = Strings::load(EN, FR, lang);
        self.refresh_status();
    }

    /// The shell's theme, pushed in.
    pub fn set_dark(&mut self, dark: bool) {
        self.dark = dark;
    }

    /// Read every settings file again, on a thread, as the first load does.
    pub fn refresh(&mut self) -> Task<Message> {
        self.loading = true;
        self.status = self.strings.t("header.scanning", &[]);
        load(
            self.rules.clone(),
            self.shell_rules.clone(),
            self.env.clone(),
            self.read_shell,
        )
    }

    fn remember(&mut self) {
        self.prefs.extra.read_shell = Some(self.read_shell);
        self.prefs.save();
    }

    fn refresh_status(&mut self) {
        self.status = match &self.inventory {
            None => self.strings.t("header.scanning", &[]),
            Some(inv) => self.strings.t(
                "footer.summary",
                &[
                    (
                        "files",
                        &inv.files.iter().filter(|f| f.exists).count().to_string(),
                    ),
                    ("shell", &inv.shell_files.len().to_string()),
                    ("roots", &inv.roots.len().to_string()),
                    ("known", &self.rules.hosts.len().to_string()),
                    ("not_read", &inv.not_read.len().to_string()),
                ],
            ),
        };
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(Ok(inv)) => {
                self.inventory = Some(inv);
                self.loading = false;
                self.error = None;
                self.refresh_status();
            }
            Message::Loaded(Err(e)) => {
                self.loading = false;
                self.error = Some(e);
            }
            Message::Rescan => return self.refresh(),
            Message::Filter(f) => self.filter = f,
            Message::Scope(choice) => {
                self.scope = self
                    .scope_options()
                    .into_iter()
                    .find(|(_, label)| *label == choice)
                    .and_then(|(root, _)| root);
            }
            Message::ToggleInstalledOnly(b) => self.installed_only = b,
            Message::ToggleFindingsOnly(b) => self.findings_only = b,
            Message::ToggleReadShell(b) => {
                self.read_shell = b;
                self.remember();
                return self.refresh();
            }
            Message::SelectCell(host, cap) => self.selected = Some((host, cap)),
            Message::SelectFinding(fi) => {
                if let Some(inv) = &self.inventory {
                    if let Some(f) = inv.findings.get(fi) {
                        self.scope = f.project.clone();
                        self.selected =
                            Some((f.host.clone(), f.capability.unwrap_or(Capability::Commands)));
                    }
                }
            }
            Message::Copy(s) => {
                self.status = self.strings.t("footer.copied", &[]);
                return iced::clipboard::write(s);
            }
            Message::OpenFile(p) => {
                let _ = std::process::Command::new("xdg-open").arg(&p).spawn();
                self.status = self
                    .strings
                    .t("footer.opened", &[("file", &sheet::short(&p, &self.env))]);
            }
            Message::OpenUrl(u) => {
                let _ = std::process::Command::new("xdg-open").arg(&u).spawn();
            }
        }
        Task::none()
    }

    // ------------------------------------------------------------------ data

    /// The sheets of the current scope: the user level, or a project's (hosts whose state
    /// does not differ there fall back to their user-level sheet).
    fn sheets(&self, inv: &Inventory) -> Vec<HostSheet> {
        let project = match &self.scope {
            None => return self.all_sheets(inv),
            Some(p) => p,
        };
        let base = self.all_sheets(inv);
        let overlay = inv.projects.iter().find(|p| &p.root == project);
        base.into_iter()
            .map(|h| {
                overlay
                    .and_then(|p| p.hosts.iter().find(|x| x.host == h.host).cloned())
                    .unwrap_or(h)
            })
            .collect()
    }

    /// Every host known to the rules, installed or not, in the inventory's order first.
    fn all_sheets(&self, inv: &Inventory) -> Vec<HostSheet> {
        let mut out: Vec<HostSheet> = inv.hosts.clone();
        if !self.installed_only {
            for rule in &self.rules.hosts {
                if !out.iter().any(|h| h.host == rule.id) {
                    let built = sheet::build(rule, &self.env, None);
                    out.push(built.sheet);
                }
            }
        }
        out
    }

    fn scope_options(&self) -> Vec<(Option<PathBuf>, String)> {
        let mut v = vec![(None, self.strings.t("filter.scope_user", &[]))];
        if let Some(inv) = &self.inventory {
            for p in &inv.projects {
                let name = p
                    .root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.root.display().to_string());
                v.push((
                    Some(p.root.clone()),
                    self.strings.t("filter.scope_project", &[("name", &name)]),
                ));
            }
        }
        v
    }

    fn visible(&self, inv: &Inventory, sheets: &[HostSheet]) -> Vec<usize> {
        visible_indices(
            sheets,
            &inv.findings,
            &self.filter,
            self.scope.as_deref(),
            self.installed_only,
            self.findings_only,
        )
    }

    fn state_color(&self, s: State) -> Color {
        match s {
            State::Auto | State::Denylist(_) => alert(self.dark),
            State::Judged => amber(self.dark),
            State::Asks | State::Allowlist(_) | State::Sandboxed => good(self.dark),
            State::Off => neutral(),
            State::Unknown => unknown(),
        }
    }

    fn state_label(&self, s: State) -> String {
        let base = match s {
            State::Asks => self.strings.t("state.asks", &[]),
            State::Auto => self.strings.t("state.auto", &[]),
            State::Judged => self.strings.t("state.judged", &[]),
            State::Allowlist(_) => self.strings.t("state.allowlist", &[]),
            State::Denylist(_) => self.strings.t("state.denylist", &[]),
            State::Sandboxed => self.strings.t("state.sandboxed", &[]),
            State::Off => self.strings.t("state.off", &[]),
            State::Unknown => self.strings.t("state.unknown", &[]),
        };
        match s {
            State::Allowlist(n) | State::Denylist(n) if n > 0 => format!("{base}({n})"),
            _ => base,
        }
    }

    fn cap_label(&self, c: Capability) -> String {
        match c {
            Capability::Commands => self.strings.t("table.commands", &[]),
            Capability::Files => self.strings.t("table.files", &[]),
            Capability::Network => self.strings.t("table.network", &[]),
            Capability::Mcp => self.strings.t("table.mcp", &[]),
        }
    }

    // ------------------------------------------------------------------ views

    pub fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = match (&self.inventory, &self.error) {
            (Some(inv), _) => {
                let sheets = self.sheets(inv);
                row![
                    column![self.view_filters(), self.view_table(inv, &sheets)]
                        .spacing(8)
                        .width(Length::FillPortion(13))
                        .height(Length::Fill),
                    container(self.view_side(inv, &sheets))
                        .width(Length::FillPortion(9))
                        .height(Length::Fill),
                ]
                .spacing(14)
                .height(Length::Fill)
                .into()
            }
            (None, Some(e)) => container(text(e.clone()).size(16))
                .center(Length::Fill)
                .into(),
            (None, None) => container(
                text(self.strings.t("header.scanning", &[]))
                    .size(16)
                    .style(text::secondary),
            )
            .center(Length::Fill)
            .into(),
        };
        column![
            self.view_header(),
            body,
            text(self.status.clone()).size(11).style(text::secondary),
        ]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let inv = self.inventory.as_deref();
        let n = |v: Option<usize>| v.map(|x| x.to_string()).unwrap_or_else(|| "—".into());
        let installed = inv.map(|i| i.hosts.len());
        let known = self.rules.hosts.len();
        let auto = inv.map(|i| {
            i.hosts
                .iter()
                .flat_map(|h| &h.capabilities)
                .filter(|c| c.state == State::Auto)
                .count()
        });
        let total = inv.map(|i| i.hosts.len() * 4).unwrap_or(0);
        let findings = inv.map(|i| i.findings.len());
        let shell = inv.map(|i| i.hosts.iter().map(|h| h.shell.len()).sum::<usize>());
        let dot = |v: Option<usize>| {
            v.map(|x| {
                if x == 0 {
                    good(self.dark)
                } else {
                    alert(self.dark)
                }
            })
        };
        row![
            bramblers_ui::widgets::kpi(
                s.t("kpi.hosts", &[]),
                n(installed),
                s.t("kpi.hosts_known", &[("n", &known.to_string())]),
                None
            ),
            bramblers_ui::widgets::kpi(
                s.t("kpi.auto", &[]),
                n(auto),
                s.t("kpi.auto_sub", &[("n", &total.to_string())]),
                dot(auto)
            ),
            bramblers_ui::widgets::kpi(s.t("kpi.findings", &[]), n(findings), "", dot(findings)),
            bramblers_ui::widgets::kpi(
                s.t("kpi.shell", &[]),
                n(shell),
                s.t(
                    "kpi.shell_sub",
                    &[(
                        "n",
                        &inv.map(|i| i.shell_files.len()).unwrap_or(0).to_string()
                    )]
                ),
                dot(shell)
            ),
        ]
        .spacing(12)
        .into()
    }

    fn view_filters(&self) -> Element<'_, Message> {
        let s = &self.strings;
        let options: Vec<String> = self.scope_options().into_iter().map(|(_, l)| l).collect();
        let selected = self
            .scope_options()
            .into_iter()
            .find(|(root, _)| *root == self.scope)
            .map(|(_, l)| l)
            .unwrap_or_else(|| s.t("filter.scope_user", &[]));
        let scope = pick_list(options, Some(selected), Message::Scope)
            .text_size(12)
            .padding([4, 8])
            .width(Length::Fixed(200.0));
        row![
            text_input(&s.t("filter.placeholder", &[]), &self.filter)
                .on_input(Message::Filter)
                .size(13)
                .width(Length::Fill),
            scope,
            checkbox(s.t("filter.installed_only", &[]), self.installed_only)
                .on_toggle(Message::ToggleInstalledOnly)
                .size(16)
                .text_size(12),
            checkbox(s.t("filter.findings_only", &[]), self.findings_only)
                .on_toggle(Message::ToggleFindingsOnly)
                .size(16)
                .text_size(12),
            checkbox(s.t("filter.read_shell", &[]), self.read_shell)
                .on_toggle(Message::ToggleReadShell)
                .size(16)
                .text_size(12),
            if self.loading {
                button(text(s.t("header.scanning", &[])).size(12)).style(button::secondary)
            } else {
                button(text(s.t("header.rescan", &[])).size(12))
                    .style(button::secondary)
                    .on_press(Message::Rescan)
            },
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
    }

    fn view_table(&self, inv: &Inventory, sheets: &[HostSheet]) -> Element<'_, Message> {
        let s = &self.strings;
        let visible = self.visible(inv, sheets);
        let head = column![
            text(s.t("table.title", &[])).size(16),
            text(s.t("table.subtitle", &[]))
                .size(11)
                .style(text::secondary),
        ]
        .spacing(2);
        if inv.hosts.is_empty() && self.installed_only {
            return card(
                column![
                    head,
                    container(text(
                        s.t("table.none", &[("n", &self.rules.hosts.len().to_string())])
                    ))
                    .padding(20)
                ]
                .spacing(10)
                .into(),
            )
            .height(Length::Fill)
            .into();
        }
        let header_cell =
            |label: String, w: Length| text(label).size(12).style(text::secondary).width(w);
        let header = row![
            header_cell(s.t("table.host", &[]), Length::FillPortion(3)),
            header_cell(self.cap_label(Capability::Commands), Length::FillPortion(3)),
            header_cell(self.cap_label(Capability::Files), Length::FillPortion(3)),
            header_cell(self.cap_label(Capability::Network), Length::FillPortion(3)),
            header_cell(self.cap_label(Capability::Mcp), Length::FillPortion(3)),
        ]
        .spacing(6)
        .padding([4, 6]);
        let mut rows = column![header].spacing(2);
        if visible.is_empty() {
            rows = rows
                .push(container(text(s.t("table.empty", &[])).style(text::secondary)).padding(12));
        }
        for i in visible {
            let h = &sheets[i];
            // Context line under the host name.
            let mut ctx: Vec<String> = Vec::new();
            if !h.installed {
                ctx.push(s.t("table.not_installed", &[]));
            }
            for hit in &h.shell {
                ctx.push(format!(
                    "▲ {}:{} `{}`",
                    sheet::short(&hit.file, &self.env),
                    hit.line,
                    hit.flag
                ));
            }
            if !h.guards.is_empty() {
                let names: Vec<&str> = h.guards.iter().map(|g| g.name.as_str()).collect();
                ctx.push(s.t("table.guards", &[("list", &names.join(", "))]));
            }
            if h.capabilities.iter().any(|c| c.state == State::Unknown) && !h.unknown.is_empty() {
                ctx.push(s.t("table.unknown", &[]));
            }
            let mut name_col = column![text(h.name.clone()).size(13)].spacing(2);
            if !ctx.is_empty() {
                name_col = name_col.push(text(ctx.join(" · ")).size(10).style(text::secondary));
            }
            let mut r = row![name_col.width(Length::FillPortion(3))]
                .spacing(6)
                .padding([6, 6])
                .align_y(Alignment::Center);
            for c in Capability::ALL {
                let cs = h.state(c).unwrap();
                let selected = self.selected.as_ref() == Some(&(h.host.clone(), c));
                let mut cell = row![pill(self.state_label(cs.state), self.state_color(cs.state))]
                    .spacing(4)
                    .align_y(Alignment::Center);
                if cs.is_default {
                    cell = cell.push(
                        text(s.t("table.default", &[]))
                            .size(10)
                            .style(text::secondary),
                    );
                }
                let cell = container(cell)
                    .padding([3, 4])
                    .width(Length::FillPortion(3))
                    .style(move |theme: &Theme| {
                        let p = theme.extended_palette();
                        container::Style {
                            background: if selected {
                                Some(p.primary.weak.color.into())
                            } else {
                                None
                            },
                            border: Border::default().rounded(5),
                            ..container::Style::default()
                        }
                    });
                r = r.push(mouse_area(cell).on_press(Message::SelectCell(h.host.clone(), c)));
            }
            let line = container(r).width(Length::Fill).style(|theme: &Theme| {
                let p = theme.extended_palette();
                container::Style {
                    background: Some(p.background.weak.color.into()),
                    border: Border::default().rounded(3),
                    ..container::Style::default()
                }
            });
            rows = rows.push(line);
        }
        card(
            column![
                head,
                scrollable(rows).height(Length::Fill),
                text(s.t("legend", &[])).size(10).style(text::secondary),
            ]
            .spacing(10)
            .into(),
        )
        .height(Length::Fill)
        .into()
    }

    fn view_side(&self, inv: &Inventory, sheets: &[HostSheet]) -> Element<'_, Message> {
        let s = &self.strings;
        let head = column![
            text(s.t("panel.title", &[])).size(16),
            text(s.t("panel.subtitle", &[]))
                .size(11)
                .style(text::secondary),
        ]
        .spacing(2);
        let detail: Element<'_, Message> = match &self.selected {
            Some((host, cap)) => match sheets.iter().find(|h| &h.host == host) {
                Some(h) => self.view_detail(h, *cap),
                None => container(
                    text(s.t("detail.pick", &[]))
                        .size(12)
                        .style(text::secondary),
                )
                .padding([8, 0])
                .into(),
            },
            None => container(
                text(s.t("detail.pick", &[]))
                    .size(12)
                    .style(text::secondary),
            )
            .padding([8, 0])
            .into(),
        };
        card(
            column![
                head,
                scrollable(detail).height(Length::FillPortion(3)),
                horizontal_rule(1),
                column![
                    text(s.t("findings.title", &[])).size(14),
                    text(s.t("findings.subtitle", &[]))
                        .size(11)
                        .style(text::secondary),
                ]
                .spacing(2),
                scrollable(self.view_findings(inv)).height(Length::FillPortion(2)),
            ]
            .spacing(10)
            .into(),
        )
        .height(Length::Fill)
        .into()
    }

    fn view_detail(&self, h: &HostSheet, cap: Capability) -> Element<'_, Message> {
        let s = &self.strings;
        let cs: &CapabilityState = h.state(cap).unwrap();
        let status = match h.precedence.status {
            bramblers_permitt_core::model::PrecedenceStatus::Documented => {
                s.t("detail.precedence.documented", &[])
            }
            bramblers_permitt_core::model::PrecedenceStatus::Undocumented => {
                s.t("detail.precedence.undocumented", &[])
            }
            bramblers_permitt_core::model::PrecedenceStatus::Contradictory => {
                s.t("detail.precedence.contradictory", &[])
            }
        };
        let order: Vec<&str> = h.precedence.order.iter().map(|l| l.label()).collect();
        let sub = format!(
            "{} · {} · {}",
            h.kind,
            if h.installed {
                s.t("detail.installed", &[])
            } else {
                s.t("detail.not_installed", &[])
            },
            s.t(
                "detail.precedence",
                &[("order", &order.join(" > ")), ("status", &status)]
            )
        );
        let mut col = column![
            row![
                text(format!("{} — {}", h.name, self.cap_label(cap))).size(18),
                horizontal_space(),
                pill(self.state_label(cs.state), self.state_color(cs.state)),
            ]
            .align_y(Alignment::Center),
            text(sub).size(11).style(text::secondary),
        ]
        .spacing(6);
        if let Some(p) = &h.project {
            col = col.push(
                text(format!("↳ {}", sheet::short(p, &self.env)))
                    .size(11)
                    .style(text::secondary),
            );
        }

        // Evidence, strongest first (the core applied weakest first).
        col = col.push(
            text(s.t("detail.evidence", &[]))
                .size(12)
                .style(text::secondary),
        );
        for e in cs.evidence.iter().rev() {
            col = col.push(self.view_evidence(e));
        }
        if !cs.ignored.is_empty() {
            col = col.push(
                text(s.t("detail.ignored", &[]))
                    .size(12)
                    .style(text::secondary),
            );
            for e in &cs.ignored {
                col = col.push(self.view_evidence(e));
            }
        }
        if !h.shell.is_empty() {
            col = col.push(
                text(s.t("detail.shell", &[]))
                    .size(12)
                    .style(text::secondary),
            );
            for hit in &h.shell {
                let block = column![
                    row![
                        text(format!(
                            "{}:{}",
                            sheet::short(&hit.file, &self.env),
                            hit.line
                        ))
                        .size(12)
                        .width(Length::Fill),
                        small_button(s.t("detail.open", &[]), Message::OpenFile(hit.file.clone())),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    text(hit.text.clone()).size(11).font(iced::Font::MONOSPACE),
                    text(format!("`{}` — {}", hit.flag, hit.effect))
                        .size(11)
                        .color(alert(self.dark)),
                ]
                .spacing(3);
                col = col.push(inner(block.into()));
            }
        }
        if let Some(act) = self.rules.host(&h.host).and_then(|r| r.hint("act")) {
            col = col.push(text(s.t("detail.act", &[])).size(12).style(text::secondary));
            col = col.push(inner(text(act.to_string()).size(12).into()));
        }
        // Other capabilities, one click away.
        let mut others = row![text(s.t("detail.other", &[("host", &h.name)]))
            .size(11)
            .style(text::secondary)]
        .spacing(6)
        .align_y(Alignment::Center);
        for c in Capability::ALL {
            if c == cap {
                continue;
            }
            let st = h.state(c).unwrap().state;
            others = others.push(
                mouse_area(
                    row![
                        text(self.cap_label(c)).size(11),
                        pill(self.state_label(st), self.state_color(st))
                    ]
                    .spacing(3)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::SelectCell(h.host.clone(), c)),
            );
        }
        col = col.push(others);
        container(col)
            .padding(iced::Padding {
                top: 4.0,
                right: 14.0,
                bottom: 4.0,
                left: 0.0,
            })
            .into()
    }

    fn view_evidence(&self, e: &Evidence) -> Element<'_, Message> {
        let s = &self.strings;
        let where_ = match (&e.file, e.line) {
            (Some(p), Some(l)) => format!("{}:{l}", sheet::short(p, &self.env)),
            (Some(p), None) => sheet::short(p, &self.env),
            (None, _) if e.level == Level::Default => s.t("detail.default", &[]),
            (None, _) => e.level.to_string(),
        };
        let kind = match e.source.kind {
            SourceKind::Documentation => s.t("detail.source.documentation", &[]),
            SourceKind::SourceCode => s.t("detail.source.source-code", &[]),
            SourceKind::Observed => s.t("detail.source.observed", &[]),
        };
        let value_line = if e.value.is_empty() {
            e.key.clone()
        } else {
            format!("{} = {}", e.key, e.value)
        };
        let place = if e.level == Level::Default {
            where_
        } else {
            format!("{} · {}", e.level, where_)
        };
        let mut head = row![
            text(place).size(11).style(text::secondary),
            horizontal_space()
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        if let Some(p) = &e.file {
            head = head.push(small_button(
                s.t("detail.open", &[]),
                Message::OpenFile(p.clone()),
            ));
            head = head.push(small_button(
                s.t("detail.copy_path", &[]),
                Message::Copy(p.display().to_string()),
            ));
        }
        let mut block = column![
            text(value_line)
                .size(12)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fill),
            head
        ]
        .spacing(3);
        if let Some(n) = &e.note {
            block = block.push(text(n.clone()).size(11));
        }
        let mut src = row![text(format!("{kind} — “{}”", e.source.quote))
            .size(11)
            .style(text::secondary)
            .width(Length::Fill)]
        .spacing(6)
        .align_y(Alignment::Center);
        if let Some(u) = &e.source.url {
            src = src.push(small_button(
                s.t("detail.open_doc", &[]),
                Message::OpenUrl(u.clone()),
            ));
        }
        block = block.push(src);
        inner(block.into())
    }

    fn view_findings(&self, inv: &Inventory) -> Element<'_, Message> {
        let s = &self.strings;
        if inv.findings.is_empty() {
            return container(
                text(s.t("findings.none", &[]))
                    .size(12)
                    .color(good(self.dark)),
            )
            .padding([6, 0])
            .into();
        }
        let mut col = column![].spacing(6);
        for (fi, f) in inv.findings.iter().enumerate() {
            let color = match f.kind {
                FindingKind::JudgedByModel => amber(self.dark),
                FindingKind::Unreadable | FindingKind::Contradictory | FindingKind::NotHonoured => {
                    neutral()
                }
                _ => alert(self.dark),
            };
            let where_ = finding_where(f, &self.env, s);
            let mut block = column![row![
                text(s.t(&format!("kind.{}", kind_slug(f.kind)), &[]))
                    .size(10)
                    .color(color),
                text(
                    f.message
                        .split(" — ")
                        .next()
                        .unwrap_or(&f.message)
                        .to_string()
                )
                .size(12),
            ]
            .spacing(8)
            .align_y(Alignment::Center),]
            .spacing(2);
            if let Some(w) = where_ {
                block = block.push(text(w).size(11).style(text::secondary));
            }
            if let Some(act) = self.rules.host(&f.host).and_then(|r| r.hint("act")) {
                block = block.push(text(format!("→ {act}")).size(11).style(text::secondary));
            }
            let c = container(block)
                .padding(8)
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
                });
            col = col.push(mouse_area(c).on_press(Message::SelectFinding(fi)));
        }
        container(col)
            .padding(iced::Padding {
                top: 4.0,
                right: 14.0,
                bottom: 4.0,
                left: 0.0,
            })
            .into()
    }
}

/// The one line under a finding: the file and key that set it, or the default's quote.
fn finding_where(f: &Finding, env: &Env, s: &Strings) -> Option<String> {
    if let Some(e) = f.evidence.iter().rev().find(|e| e.level != Level::Default) {
        let loc = match (&e.file, e.line) {
            (Some(p), Some(l)) => format!("{}:{l}", sheet::short(p, env)),
            (Some(p), None) => sheet::short(p, env),
            _ => e.level.to_string(),
        };
        return Some(format!("{loc} · {} = {}", e.key, e.value));
    }
    if let Some(e) = f.evidence.iter().find(|e| e.level == Level::Default) {
        return Some(format!(
            "{} — “{}”",
            s.t("findings.by_default", &[]),
            e.source.quote
        ));
    }
    f.message.split(" — ").nth(1).map(String::from)
}

/// Does this host carry a finding for the current scope (or one that applies everywhere)?
fn host_has_finding(findings: &[Finding], scope: Option<&Path>, host: &str) -> bool {
    findings
        .iter()
        .any(|f| f.host == host && f.project.as_deref() == scope)
        || findings
            .iter()
            .any(|f| f.host == host && f.project.is_none())
}

/// The indexes of the sheets to show: the search, the scope and the two checkboxes.
fn visible_indices(
    sheets: &[HostSheet],
    findings: &[Finding],
    filter: &str,
    scope: Option<&Path>,
    installed_only: bool,
    findings_only: bool,
) -> Vec<usize> {
    let needle = filter.trim().to_lowercase();
    (0..sheets.len())
        .filter(|&i| {
            let h = &sheets[i];
            if installed_only && !h.installed {
                return false;
            }
            if findings_only && !host_has_finding(findings, scope, &h.host) {
                return false;
            }
            if !needle.is_empty() && !h.name.to_lowercase().contains(&needle) {
                return false;
            }
            true
        })
        .collect()
}

fn load(rules: Arc<Rules>, shell: Arc<ShellRules>, env: Env, read_shell: bool) -> Task<Message> {
    let (tx, rx) = iced::futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let inv = inventory::run(
            &rules,
            &shell,
            &env,
            Options {
                no_shell: !read_shell,
                all_hosts: false,
            },
        );
        let _ = tx.send(Arc::new(inv));
    });
    Task::perform(
        async move {
            rx.await
                .map_err(|_| "the inventory thread stopped unexpectedly".to_string())
        },
        Message::Loaded,
    )
}

fn kind_slug(k: FindingKind) -> &'static str {
    match k {
        FindingKind::RunsWithoutAsking => "runs-without-asking",
        FindingKind::JudgedByModel => "judged-by-model",
        FindingKind::DenylistOnly => "denylist-only",
        FindingKind::ShellBypass => "shell-bypass",
        FindingKind::NotHonoured => "not-honoured",
        FindingKind::Unreadable => "unreadable",
        FindingKind::Contradictory => "contradictory",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bramblers_hostrules::SourceKind;
    use bramblers_permitt_core::model::{PrecedenceInfo, PrecedenceStatus, SourceRef};

    fn sheet(host: &str, name: &str, installed: bool) -> HostSheet {
        HostSheet {
            host: host.to_string(),
            name: name.to_string(),
            kind: "cli".to_string(),
            installed,
            project: None,
            capabilities: Capability::ALL
                .iter()
                .map(|&capability| CapabilityState {
                    capability,
                    state: State::Asks,
                    is_default: true,
                    evidence: Vec::new(),
                    ignored: Vec::new(),
                })
                .collect(),
            guards: Vec::new(),
            shell: Vec::new(),
            precedence: PrecedenceInfo {
                order: vec![Level::User],
                status: PrecedenceStatus::Documented,
                note: None,
                source: None,
            },
            unknown: Vec::new(),
        }
    }

    fn finding(host: &str, project: Option<&str>) -> Finding {
        Finding {
            kind: FindingKind::RunsWithoutAsking,
            host: host.to_string(),
            host_name: host.to_string(),
            capability: Some(Capability::Commands),
            project: project.map(PathBuf::from),
            message: "Claude Code runs shell commands without asking".to_string(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn locales_have_the_same_keys() {
        bramblers_ui::i18n::same_keys(EN, FR).unwrap();
    }

    #[test]
    fn the_filters_combine() {
        let sheets = vec![
            sheet("alpha", "Alpha", true),
            sheet("beta", "Beta", false),
            sheet("gamma", "Gamma", true),
        ];
        let findings = vec![finding("gamma", None)];
        assert_eq!(
            visible_indices(&sheets, &findings, "", None, true, false),
            vec![0, 2]
        );
        assert_eq!(
            visible_indices(&sheets, &findings, "", None, true, true),
            vec![2]
        );
        assert_eq!(
            visible_indices(&sheets, &findings, " bet ", None, false, false),
            vec![1]
        );
        assert!(visible_indices(&sheets, &findings, "delta", None, false, false).is_empty());
    }

    #[test]
    fn a_project_finding_only_counts_in_its_scope() {
        let sheets = vec![sheet("alpha", "Alpha", true)];
        let findings = vec![finding("alpha", Some("/work/project"))];
        assert!(visible_indices(&sheets, &findings, "", None, true, true).is_empty());
        assert_eq!(
            visible_indices(
                &sheets,
                &findings,
                "",
                Some(Path::new("/work/project")),
                true,
                true
            ),
            vec![0]
        );
    }

    #[test]
    fn finding_where_prefers_the_setting_over_the_default() {
        let s = Strings::load(EN, FR, Lang::En);
        let env = Env::fixture(Path::new("/home/x"), bramblers_hostrules::Os::Linux);
        let source = SourceRef {
            kind: SourceKind::Documentation,
            url: None,
            quote: "quoted".to_string(),
        };
        let evidence = |level, key: &str, value: &str| Evidence {
            level,
            file: Some(PathBuf::from("/home/x/.claude/settings.json")),
            line: Some(3),
            key: key.to_string(),
            value: value.to_string(),
            note: None,
            source: source.clone(),
        };
        let f = Finding {
            kind: FindingKind::RunsWithoutAsking,
            host: "claude-code".to_string(),
            host_name: "Claude Code".to_string(),
            capability: Some(Capability::Commands),
            project: None,
            message: "Claude Code runs shell commands without asking — by default".to_string(),
            evidence: vec![
                evidence(Level::User, "permissions.defaultMode", "acceptEdits"),
                evidence(Level::Default, "permissions.defaultMode", "ask"),
            ],
        };
        assert_eq!(
            finding_where(&f, &env, &s).unwrap(),
            "~/.claude/settings.json:3 · permissions.defaultMode = acceptEdits"
        );
        let default_only = Finding {
            evidence: vec![evidence(Level::Default, "permissions.defaultMode", "ask")],
            ..f
        };
        assert_eq!(
            finding_where(&default_only, &env, &s).unwrap(),
            "by default, nothing changed it — “quoted”"
        );
    }

    #[test]
    fn every_finding_kind_has_a_locale_key() {
        let s = Strings::load(EN, FR, Lang::En);
        for kind in [
            FindingKind::RunsWithoutAsking,
            FindingKind::JudgedByModel,
            FindingKind::DenylistOnly,
            FindingKind::ShellBypass,
            FindingKind::NotHonoured,
            FindingKind::Unreadable,
            FindingKind::Contradictory,
        ] {
            let label = s.t(&format!("kind.{}", kind_slug(kind)), &[]);
            assert!(!label.starts_with('⟨'), "{kind:?} → {label}");
        }
    }
}
