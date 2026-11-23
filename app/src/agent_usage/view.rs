//! The "Agent usage" pane: how many tokens the AI agents on this computer spend, when and where,
//! and how much of their plans is left.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;

use brasaui::assets::asset_cache::AssetSource;
use brasaui::elements::{
    Align, Border, CacheOption, ClippedScrollStateHandle, ClippedScrollable, ConstrainedBox,
    Container, CornerRadius, CrossAxisAlignment, Empty, Expanded, Fill as UiFill, Flex, Hoverable,
    Image, MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement, Radius,
    ScrollbarWidth, Stack, Text, Wrap,
};
use brasaui::fonts::{FamilyId, Properties, Weight};
use brasaui::platform::Cursor;
use brasaui::text_layout::ClipConfig;
use brasaui::ui_components::components::UiComponent;
use brasaui::{
    AppContext, Element, Entity, ModelHandle, SingletonEntity, TypedActionView, View, ViewContext,
};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc};
use enum_iterator::{Sequence, all};
use pathfinder_color::ColorU;
use settings::Setting as _;

use super::collect::{UsageAgent, UsageData, UsagePlace, UsageSources, collect};
use super::equal_height_row::EqualHeightRow;
use super::limits::{LimitWindow, PlanLimits, codex_limits};
use super::stats::{
    DayStats, Provider, Totals, UsageRange, UsageSummary, format_count, format_duration,
    format_tokens, percent_change, summarize, token_total,
};
use crate::appearance::Appearance;
use crate::pane_group::focus_state::PaneFocusHandle;
use crate::pane_group::pane::view::{self, HeaderContent, StandardHeader, StandardHeaderOptions};
use crate::pane_group::{BackingView, PaneConfiguration, PaneEvent};
use crate::settings::{AISettings, AISettingsChangedEvent};
use crate::terminal::cli_agent::CLIAgent;
use crate::ui_components::blended_colors;
use crate::ui_components::buttons::icon_button_with_color;
use crate::ui_components::icons::Icon;

pub const AGENT_USAGE_TITLE: &str = "Agent usage";

/// Where Brasa keeps the Claude limit readings it has asked for, so the weekly chart has a history.
#[cfg(not(target_family = "wasm"))]
const CLAUDE_LIMITS_FILE: &str = "agent_usage_claude_limits.json";

const PAGE_PADDING: f32 = 24.;
const CARD_PADDING: f32 = 16.;
const GAP: f32 = 14.;
const ACTIVITY_CHART_HEIGHT: f32 = 150.;
const LIMIT_CHART_HEIGHT: f32 = 64.;
const HEATMAP_ROW_HEIGHT: f32 = 18.;
/// Below this width cards in a row stack vertically.
const MIN_CARD_WIDTH: f32 = 320.;
const ICON_SIZE: f32 = 14.;
const MAX_MODEL_ROWS: usize = 6;
/// Ranges longer than this are charted by week so the bars stay readable.
const MAX_DAILY_BARS: usize = 120;
const BRASA_ORANGE: ColorU = ColorU {
    r: 255,
    g: 122,
    b: 47,
    a: 255,
};
const BRASA_AMBER: ColorU = ColorU {
    r: 255,
    g: 183,
    b: 77,
    a: 255,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentUsageViewEvent {
    Pane(PaneEvent),
}

/// What the activity chart shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum ChartMetric {
    Input,
    Output,
    Tasks,
    Time,
}

impl ChartMetric {
    fn label(self) -> &'static str {
        match self {
            ChartMetric::Input => "Input",
            ChartMetric::Output => "Output",
            ChartMetric::Tasks => "Tasks",
            ChartMetric::Time => "Time",
        }
    }

    fn value(self, totals: &Totals) -> u64 {
        match self {
            ChartMetric::Input => totals.input_tokens,
            ChartMetric::Output => totals.output_tokens,
            ChartMetric::Tasks => totals.tasks,
            ChartMetric::Time => totals.agent_seconds.max(0) as u64,
        }
    }

    fn format(self, value: u64) -> String {
        match self {
            ChartMetric::Input | ChartMetric::Output => format_tokens(value),
            ChartMetric::Tasks => format_count(value),
            ChartMetric::Time => format_duration(value as i64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentUsageAction {
    SetRange(UsageRange),
    ToggleAgent(UsageAgent),
    SetChartMetric(ChartMetric),
    SetClaudeLimits(bool),
    ToggleCustomize,
    ToggleHidden(UsageAgent),
    Refresh,
}

enum ClaudeLimitsState {
    Off,
    Loading,
    Loaded(PlanLimits),
    Unavailable(String),
}

pub struct AgentUsageView {
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    data: Option<Arc<UsageData>>,
    loading: bool,
    range: UsageRange,
    agent_filter: Option<UsageAgent>,
    /// Agents left out of every number and card.
    hidden_agents: HashSet<UsageAgent>,
    /// Whether the card for choosing the hidden agents is open.
    customizing: bool,
    chart_metric: ChartMetric,
    summary: Option<UsageSummary>,
    codex_limits: Option<PlanLimits>,
    claude_limits: ClaudeLimitsState,
    scroll_state: ClippedScrollStateHandle,
    range_buttons: HashMap<UsageRange, MouseStateHandle>,
    metric_buttons: HashMap<ChartMetric, MouseStateHandle>,
    agent_rows: HashMap<UsageAgent, MouseStateHandle>,
    visibility_buttons: HashMap<UsageAgent, MouseStateHandle>,
    customize_button: MouseStateHandle,
    refresh_button: MouseStateHandle,
    claude_limits_button: MouseStateHandle,
    claude_limits_off_button: MouseStateHandle,
}

impl AgentUsageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let pane_configuration = ctx.add_model(|_ctx| PaneConfiguration::new(AGENT_USAGE_TITLE));
        ctx.subscribe_to_model(&AISettings::handle(ctx), |me, _, event, ctx| {
            if matches!(event, AISettingsChangedEvent::AgentUsageClaudeLimits { .. }) {
                me.refresh_claude_limits(ctx);
            } else if matches!(event, AISettingsChangedEvent::AgentUsageHiddenAgents { .. }) {
                let hidden = hidden_agents(ctx);
                if hidden != me.hidden_agents {
                    me.hidden_agents = hidden;
                    me.resummarize();
                    ctx.notify();
                }
            }
        });
        let mut view = Self {
            pane_configuration,
            focus_handle: None,
            data: None,
            loading: false,
            range: UsageRange::Month,
            agent_filter: None,
            hidden_agents: hidden_agents(ctx),
            customizing: false,
            chart_metric: ChartMetric::Input,
            summary: None,
            codex_limits: None,
            claude_limits: ClaudeLimitsState::Off,
            scroll_state: ClippedScrollStateHandle::default(),
            range_buttons: all::<UsageRange>()
                .map(|range| (range, MouseStateHandle::default()))
                .collect(),
            metric_buttons: all::<ChartMetric>()
                .map(|metric| (metric, MouseStateHandle::default()))
                .collect(),
            agent_rows: all::<UsageAgent>()
                .map(|agent| (agent, MouseStateHandle::default()))
                .collect(),
            visibility_buttons: all::<UsageAgent>()
                .map(|agent| (agent, MouseStateHandle::default()))
                .collect(),
            customize_button: MouseStateHandle::default(),
            refresh_button: MouseStateHandle::default(),
            claude_limits_button: MouseStateHandle::default(),
            claude_limits_off_button: MouseStateHandle::default(),
        };
        view.reload(ctx);
        view.refresh_claude_limits(ctx);
        view
    }

    pub fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    pub fn focus(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.focus_self();
    }

    /// Reads the agents' history again in the background.
    pub fn reload(&mut self, ctx: &mut ViewContext<Self>) {
        if self.loading {
            return;
        }
        self.loading = true;
        ctx.notify();
        #[cfg(not(target_family = "wasm"))]
        ctx.spawn(
            async {
                tokio::task::spawn_blocking(load_usage)
                    .await
                    .unwrap_or_default()
            },
            |me, data, ctx| me.set_data(data, ctx),
        );
        #[cfg(target_family = "wasm")]
        self.set_data(UsageData::default(), ctx);
    }

    fn set_data(&mut self, data: UsageData, ctx: &mut ViewContext<Self>) {
        self.codex_limits = codex_limits(&data.codex_limits, Utc::now());
        self.data = Some(Arc::new(data));
        self.loading = false;
        self.resummarize();
        ctx.notify();
    }

    fn resummarize(&mut self) {
        if self
            .agent_filter
            .is_some_and(|agent| self.hidden_agents.contains(&agent))
        {
            self.agent_filter = None;
        }
        self.summary = self.data.as_ref().map(|data| {
            let visible;
            let data = if self.hidden_agents.is_empty() {
                data.as_ref()
            } else {
                visible = data.without_agents(&self.hidden_agents);
                &visible
            };
            summarize(data, self.range, self.agent_filter, Local::now())
        });
    }

    fn is_hidden(&self, agent: UsageAgent) -> bool {
        self.hidden_agents.contains(&agent)
    }

    /// Asks Anthropic for the Claude plan limits when the user turned that on.
    fn refresh_claude_limits(&mut self, ctx: &mut ViewContext<Self>) {
        if !*AISettings::as_ref(ctx).agent_usage_claude_limits {
            self.claude_limits = ClaudeLimitsState::Off;
            ctx.notify();
            return;
        }
        self.claude_limits = ClaudeLimitsState::Loading;
        ctx.notify();
        #[cfg(not(target_family = "wasm"))]
        {
            let history_file = brasa_core::paths::state_dir().join(CLAUDE_LIMITS_FILE);
            ctx.spawn(
                async move {
                    tokio::task::spawn_blocking(move || {
                        super::limits::fetch_claude_limits(&history_file, Utc::now())
                    })
                    .await
                    .unwrap_or_else(|_| Err("Could not ask Anthropic.".to_string()))
                },
                |me, result, ctx| {
                    me.claude_limits = match result {
                        Ok(limits) => ClaudeLimitsState::Loaded(limits),
                        Err(message) => ClaudeLimitsState::Unavailable(message),
                    };
                    ctx.notify();
                },
            );
        }
    }

    fn render_header(&self, palette: &Palette, appearance: &Appearance) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let logo = ConstrainedBox::new(
            Image::new(
                AssetSource::Bundled {
                    path: "bundled/svg/brasita-color.svg",
                },
                CacheOption::BySize,
            )
            .finish(),
        )
        .with_width(34.)
        .with_height(34.)
        .finish();
        let subtitle = match &self.summary {
            Some(summary) => format!(
                "{} – today · read from this computer",
                summary.first_day.format("%b %-d")
            ),
            None => "Read from this computer".to_string(),
        };
        let title = Flex::column()
            .with_child(
                Text::new(AGENT_USAGE_TITLE, font, 18.)
                    .with_color(palette.text)
                    .with_style(bold())
                    .finish(),
            )
            .with_child(text(subtitle, font, 12., palette.sub))
            .finish();
        let left = Flex::row()
            .with_spacing(12.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(logo)
            .with_child(title)
            .finish();

        let mut ranges = Flex::row().with_spacing(2.);
        for range in all::<UsageRange>() {
            if let Some(mouse_state) = self.range_buttons.get(&range) {
                ranges.add_child(pill_button(
                    mouse_state.clone(),
                    range.label(),
                    self.range == range,
                    AgentUsageAction::SetRange(range),
                    palette,
                    font,
                ));
            }
        }
        let mut right = Flex::row()
            .with_spacing(8.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center);
        if self.loading {
            right.add_child(text("Reading history…", font, 12., palette.faint));
        }
        let group = |content: Box<dyn Element>| {
            Container::new(content)
                .with_uniform_padding(3.)
                .with_background_color(palette.card)
                .with_border(Border::all(1.).with_border_color(palette.border))
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(9.)))
                .finish()
        };
        right.add_child(group(ranges.finish()));
        right.add_child(group(pill_button(
            self.customize_button.clone(),
            "Customize",
            self.customizing,
            AgentUsageAction::ToggleCustomize,
            palette,
            font,
        )));
        right.add_child(
            icon_button_with_color(
                appearance,
                Icon::Refresh,
                false,
                self.refresh_button.clone(),
                palette.sub.into(),
            )
            .build()
            .on_click(|ctx, _, _| ctx.dispatch_typed_action(AgentUsageAction::Refresh))
            .finish(),
        );

        Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(left)
            .with_child(right.finish())
            .finish()
    }

    fn render_limits(
        &self,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Option<Box<dyn Element>> {
        let mut cards = Vec::new();
        if !self.is_hidden(UsageAgent::ClaudeCode) {
            cards.push(self.render_claude_limits(palette, appearance));
        }
        if let Some(limits) = &self.codex_limits
            && !self.is_hidden(UsageAgent::Codex)
        {
            cards.push(limit_card(
                UsageAgent::Codex,
                limits.plan.clone(),
                render_plan_limits(limits, palette, appearance),
                None,
                palette,
                appearance,
            ));
        }
        if cards.len() < 2 {
            return cards.pop();
        }
        let mut row = EqualHeightRow::new(GAP, MIN_CARD_WIDTH);
        for card in cards {
            row = row.with_child(1., card);
        }
        Some(row.finish())
    }

    /// The agents a user may want to hide: those with history, those with a plan card, and those
    /// already hidden.
    fn render_customize(&self, palette: &Palette, appearance: &Appearance) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let mut agents: BTreeSet<UsageAgent> = self.hidden_agents.iter().copied().collect();
        agents.insert(UsageAgent::ClaudeCode);
        if self.codex_limits.is_some() {
            agents.insert(UsageAgent::Codex);
        }
        if let Some(data) = &self.data {
            agents.extend(data.sessions.iter().map(|session| session.agent));
        }
        let mut chips = Wrap::row().with_spacing(8.).with_run_spacing(8.);
        for agent in agents {
            if let Some(mouse_state) = self.visibility_buttons.get(&agent) {
                chips.add_child(visibility_chip(
                    mouse_state.clone(),
                    agent,
                    !self.is_hidden(agent),
                    palette,
                    font,
                ));
            }
        }
        card(
            Flex::column()
                .with_spacing(12.)
                .with_child(card_header(
                    "Agents in this panel",
                    "click one to hide or show it".to_string(),
                    palette,
                    appearance,
                ))
                .with_child(chips.finish())
                .with_child(text(
                    "Hidden agents leave every number, chart and plan card.",
                    font,
                    11.,
                    palette.faint,
                ))
                .finish(),
            palette,
        )
    }

    fn render_claude_limits(&self, palette: &Palette, appearance: &Appearance) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let turn_off = || {
            link_button(
                self.claude_limits_off_button.clone(),
                "Turn off",
                AgentUsageAction::SetClaudeLimits(false),
                palette,
                font,
            )
        };
        let (plan, body, footer) = match &self.claude_limits {
            ClaudeLimitsState::Off => {
                let body = Flex::column()
                    .with_spacing(8.)
                    .with_child(text_wrapped(
                        "See how much of your Claude plan is left, like /usage in Claude Code.",
                        font,
                        13.,
                        palette.text,
                    ))
                    .with_child(text_wrapped(
                        "Brasa asks Anthropic using the Claude Code login on this computer. \
                         Everything else on this page stays on your PC.",
                        font,
                        12.,
                        palette.sub,
                    ))
                    .with_child(
                        Align::new(primary_button(
                            self.claude_limits_button.clone(),
                            "Show Claude limits",
                            AgentUsageAction::SetClaudeLimits(true),
                            font,
                        ))
                        .left()
                        .finish(),
                    )
                    .finish();
                (None, body, None)
            }
            ClaudeLimitsState::Loading => (
                None,
                text("Asking Anthropic…", font, 13., palette.sub),
                None,
            ),
            ClaudeLimitsState::Unavailable(message) => (
                None,
                text_wrapped(message, font, 13., palette.sub),
                Some(turn_off()),
            ),
            ClaudeLimitsState::Loaded(limits) => {
                let mut footer = Flex::row()
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .with_cross_axis_alignment(CrossAxisAlignment::Center);
                footer.add_child(match &limits.notice {
                    Some(notice) => text(notice.clone(), font, 11., palette.yellow),
                    None => Empty::new().finish(),
                });
                footer.add_child(turn_off());
                (
                    limits.plan.clone(),
                    render_plan_limits(limits, palette, appearance),
                    Some(footer.finish()),
                )
            }
        };
        limit_card(
            UsageAgent::ClaudeCode,
            plan,
            body,
            footer,
            palette,
            appearance,
        )
    }

    fn render_summary(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let totals = &summary.totals;
        let previous = summary.previous.as_ref();
        // Changes this large only mean the previous period was nearly empty.
        let change = |value: fn(&Totals) -> u64| {
            previous
                .and_then(|previous| percent_change(value(totals), value(previous)))
                .filter(|change| change.abs() < 500)
        };
        let per_turn = match totals.turns {
            0 => "no prompts yet".to_string(),
            turns => format!("{} per prompt", format_tokens(totals.output_tokens / turns)),
        };
        let cache_share = if totals.input_tokens > 0 {
            totals.cached_tokens * 100 / totals.input_tokens
        } else {
            0
        };
        let stats = [
            (
                "Tokens in",
                format_tokens(totals.input_tokens),
                format!("{cache_share}% reused from cache"),
                change(|totals| totals.input_tokens),
            ),
            (
                "Tokens out",
                format_tokens(totals.output_tokens),
                per_turn,
                change(|totals| totals.output_tokens),
            ),
            (
                "Tasks",
                format_count(totals.tasks),
                match totals.turns {
                    1 => "1 prompt".to_string(),
                    turns => format!("{} prompts", format_count(turns)),
                },
                change(|totals| totals.tasks),
            ),
            (
                "Agents working",
                format_duration(totals.agent_seconds),
                format!("up to {} at the same time", summary.max_concurrent),
                change(|totals| totals.agent_seconds.max(0) as u64),
            ),
        ];
        let mut row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);
        for (index, (label, value, detail, change)) in stats.into_iter().enumerate() {
            if index > 0 {
                row.add_child(
                    Container::new(
                        ConstrainedBox::new(swatch(palette.border, 0.))
                            .with_width(1.)
                            .with_height(48.)
                            .finish(),
                    )
                    .with_padding_left(GAP)
                    .with_padding_right(GAP)
                    .finish(),
                );
            }
            let mut top = Flex::row()
                .with_spacing(8.)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(overline(label, palette, appearance));
            if let Some(change) = change {
                top.add_child(change_badge(change, palette, font));
            }
            row.add_child(
                Expanded::new(
                    1.,
                    Flex::column()
                        .with_spacing(3.)
                        .with_child(top.finish())
                        .with_child(
                            Text::new(value, font, 22.)
                                .with_color(palette.text)
                                .with_style(bold())
                                .finish(),
                        )
                        .with_child(text(detail, font, 11., palette.sub))
                        .finish(),
                )
                .finish(),
            );
        }
        card(row.finish(), palette)
    }

    fn render_activity(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let metric = self.chart_metric;
        let buckets = chart_buckets(&summary.days, metric);
        let max = buckets.iter().copied().max().unwrap_or(0);
        let total: u64 = summary
            .days
            .iter()
            .map(|day| metric.value(&day.totals))
            .sum();
        let per_day = total / summary.days.len().max(1) as u64;
        let detail = format!(
            "{} of {} days with agents · {} a day on average",
            summary.days_with_activity(),
            summary.days.len(),
            metric.format(per_day)
        );

        let mut pills = Flex::row().with_spacing(2.);
        for option in all::<ChartMetric>() {
            if let Some(mouse_state) = self.metric_buttons.get(&option) {
                pills.add_child(pill_button(
                    mouse_state.clone(),
                    option.label(),
                    option == metric,
                    AgentUsageAction::SetChartMetric(option),
                    palette,
                    font,
                ));
            }
        }
        let header = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(overline("Activity", palette, appearance))
            .with_child(Expanded::new(1., Align::new(pills.finish()).right().finish()).finish())
            .finish();

        let spacing = match buckets.len() {
            0..=45 => 4.,
            46..=120 => 1.,
            _ => 0.,
        };
        let last = buckets.len().saturating_sub(1);
        let mut bars = Flex::row().with_spacing(spacing);
        for (index, value) in buckets.iter().enumerate() {
            let color = if index == last {
                BRASA_AMBER
            } else {
                with_alpha(BRASA_ORANGE, 190)
            };
            bars.add_child(
                Expanded::new(1., vertical_bar(fraction(*value, max), color, true)).finish(),
            );
        }
        let axis = ConstrainedBox::new(
            Flex::column()
                .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .with_main_axis_size(MainAxisSize::Max)
                .with_child(text(metric.format(max), font, 11., palette.sub))
                .with_child(text(metric.format(max / 2), font, 11., palette.faint))
                .with_child(text("0", font, 11., palette.faint))
                .finish(),
        )
        .with_width(52.)
        .finish();
        let gridline = || {
            ConstrainedBox::new(swatch(palette.track, 0.))
                .with_height(1.)
                .finish()
        };
        let gridlines = Flex::column()
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_main_axis_size(MainAxisSize::Max)
            .with_child(gridline())
            .with_child(gridline())
            .with_child(gridline())
            .finish();
        let plot = Stack::new()
            .with_child(gridlines)
            .with_child(bars.finish())
            .finish();
        let chart = ConstrainedBox::new(
            Flex::row()
                .with_child(axis)
                .with_child(Expanded::new(1., plot).finish())
                .finish(),
        )
        .with_height(ACTIVITY_CHART_HEIGHT)
        .finish();

        let middle = summary.days.get(summary.days.len() / 2);
        let dates = Flex::row()
            .with_main_axis_size(MainAxisSize::Max)
            .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .with_child(text(
                summary.first_day.format("%b %-d").to_string(),
                font,
                11.,
                palette.faint,
            ))
            .with_child(text(
                middle
                    .map(|day| day.date.format("%b %-d").to_string())
                    .unwrap_or_default(),
                font,
                11.,
                palette.faint,
            ))
            .with_child(text("today", font, 11., BRASA_AMBER))
            .finish();

        card(
            Flex::column()
                .with_spacing(10.)
                .with_child(header)
                .with_child(text(detail, font, 11., palette.faint))
                .with_child(chart)
                .with_child(Container::new(dates).with_padding_left(52.).finish())
                .finish(),
            palette,
        )
    }

    fn render_heatmap(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let max = summary.heatmap.iter().flatten().max().copied().unwrap_or(0);
        let detail = summary
            .busiest_slot()
            .map(|(weekday, hour)| format!("busiest: {} at {hour}:00", WEEKDAY_NAMES[weekday]))
            .unwrap_or_default();

        let mut grid = Flex::column().with_spacing(3.);
        for (weekday, hours) in summary.heatmap.iter().enumerate() {
            let mut cells = Flex::row().with_spacing(3.);
            for count in hours {
                cells.add_child(Expanded::new(1., swatch(palette.heat(*count, max), 3.)).finish());
            }
            grid.add_child(
                ConstrainedBox::new(
                    Flex::row()
                        .with_child(
                            ConstrainedBox::new(text(
                                WEEKDAY_LETTERS[weekday],
                                font,
                                10.,
                                palette.faint,
                            ))
                            .with_width(16.)
                            .finish(),
                        )
                        .with_child(Expanded::new(1., cells.finish()).finish())
                        .finish(),
                )
                .with_height(HEATMAP_ROW_HEIGHT)
                .finish(),
            );
        }

        let mut hours = Flex::row();
        for label in ["0:00", "6:00", "12:00", "18:00"] {
            hours.add_child(
                Expanded::new(
                    1.,
                    Align::new(text(label, font, 10., palette.faint))
                        .left()
                        .finish(),
                )
                .finish(),
            );
        }
        let mut legend = Flex::row()
            .with_spacing(3.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(text("quiet", font, 10., palette.faint));
        for level in 0..=4u64 {
            legend.add_child(
                ConstrainedBox::new(swatch(palette.heat(level, 4), 2.))
                    .with_width(9.)
                    .with_height(9.)
                    .finish(),
            );
        }
        legend.add_child(text("busy", font, 10., palette.faint));

        card(
            Flex::column()
                .with_spacing(10.)
                .with_child(card_header("Busiest hours", detail, palette, appearance))
                .with_child(grid.finish())
                .with_child(
                    Container::new(hours.finish())
                        .with_padding_left(16.)
                        .finish(),
                )
                .with_child(Align::new(legend.finish()).right().finish())
                .finish(),
            palette,
        )
    }

    fn render_models(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let total: u64 = summary.providers.iter().map(|(_, tokens)| tokens).sum();
        let segments: Vec<(ColorU, f32)> = summary
            .providers
            .iter()
            .map(|(provider, tokens)| (provider_color(*provider), share(*tokens, total)))
            .collect();
        let mut legend = Flex::row()
            .with_spacing(14.)
            .with_cross_axis_alignment(CrossAxisAlignment::Center);
        for (provider, tokens) in &summary.providers {
            legend.add_child(
                Flex::row()
                    .with_spacing(5.)
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(provider_icon(*provider))
                    .with_child(text(
                        format!(
                            "{} {}",
                            provider.display_name(),
                            percent_label(*tokens, total)
                        ),
                        font,
                        11.,
                        palette.sub,
                    ))
                    .finish(),
            );
        }

        let max_tokens = summary
            .models
            .first()
            .map(|model| token_total(&model.totals))
            .unwrap_or(0);
        let mut columns = [
            Flex::column().with_spacing(12.),
            Flex::column().with_spacing(12.),
        ];
        let per_column = summary.models.len().min(MAX_MODEL_ROWS).div_ceil(2);
        for (index, model) in summary.models.iter().take(MAX_MODEL_ROWS).enumerate() {
            let tokens = token_total(&model.totals);
            let mut detail = format!(
                "in {} · out {}",
                format_tokens(model.totals.input_tokens),
                format_tokens(model.totals.output_tokens)
            );
            // Models used only by subagents have no tasks of their own.
            if model.totals.tasks > 0 {
                detail.push_str(&format!(" · {}", tasks_label(model.totals.tasks)));
            }
            columns[usize::from(index >= per_column)].add_child(leaderboard_row(
                provider_icon(model.provider),
                model.name.clone(),
                detail,
                percent_label(tokens, total),
                fraction(tokens, max_tokens),
                provider_color(model.provider),
                palette,
                font,
            ));
        }
        let [left, right] = columns;
        let mut rows = Flex::column().with_spacing(12.).with_child(
            Flex::row()
                .with_spacing(28.)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(Expanded::new(1., left.finish()).finish())
                .with_child(Expanded::new(1., right.finish()).finish())
                .finish(),
        );
        let rest = &summary.models[summary.models.len().min(MAX_MODEL_ROWS)..];
        if !rest.is_empty() {
            let tokens: u64 = rest.iter().map(|model| token_total(&model.totals)).sum();
            rows.add_child(text(
                format!(
                    "+ {} more models · {} tokens",
                    rest.len(),
                    format_tokens(tokens)
                ),
                font,
                11.,
                palette.faint,
            ));
        }

        let models_label = match summary.models.len() {
            1 => "1 model".to_string(),
            count => format!("{count} models"),
        };
        card(
            Flex::column()
                .with_spacing(12.)
                .with_child(card_header("Models", models_label, palette, appearance))
                .with_child(segmented_bar(&segments, palette.track))
                .with_child(legend.finish())
                .with_child(rows.finish())
                .finish(),
            palette,
        )
    }

    fn render_agents(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let total: u64 = summary
            .agents
            .iter()
            .map(|agent| token_total(&agent.totals))
            .sum();
        let max_tokens = summary
            .agents
            .iter()
            .map(|agent| token_total(&agent.totals))
            .max()
            .unwrap_or(0);
        let max_seconds = summary
            .agents
            .iter()
            .map(|agent| agent.totals.agent_seconds.max(0) as u64)
            .max()
            .unwrap_or(0);
        let mut rows = Flex::column().with_spacing(4.);
        for stats in &summary.agents {
            let Some(mouse_state) = self.agent_rows.get(&stats.agent).cloned() else {
                continue;
            };
            let agent = stats.agent;
            let selected = self.agent_filter == Some(agent);
            let dimmed = self.agent_filter.is_some() && !selected;
            let tokens = token_total(&stats.totals);
            // Local AI has no token counts, so it is measured by time.
            let (detail, share_label, bar) = if agent == UsageAgent::LocalAi {
                (
                    tasks_label(stats.totals.tasks),
                    format_duration(stats.totals.agent_seconds),
                    fraction(stats.totals.agent_seconds.max(0) as u64, max_seconds),
                )
            } else {
                (
                    format!(
                        "in {} · out {} · {}",
                        format_tokens(stats.totals.input_tokens),
                        format_tokens(stats.totals.output_tokens),
                        format_duration(stats.totals.agent_seconds)
                    ),
                    percent_label(tokens, total),
                    fraction(tokens, max_tokens),
                )
            };
            let (icon, color) = agent_icon(agent, palette);
            let row_palette = if dimmed { palette.dimmed() } else { *palette };
            let row = leaderboard_row(
                icon_element(icon, if dimmed { palette.faint } else { color }),
                agent.display_name().to_string(),
                detail,
                share_label,
                bar,
                if dimmed { palette.track } else { color },
                &row_palette,
                font,
            );
            let palette = *palette;
            rows.add_child(
                Hoverable::new(mouse_state, move |state| {
                    let background = if selected {
                        with_alpha(BRASA_ORANGE, 36)
                    } else if state.is_hovered() {
                        palette.track
                    } else {
                        ColorU::transparent_black()
                    };
                    Container::new(row)
                        .with_uniform_padding(8.)
                        .with_background_color(background)
                        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
                        .finish()
                })
                .with_cursor(Cursor::PointingHand)
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(AgentUsageAction::ToggleAgent(agent))
                })
                .finish(),
            );
        }
        let detail = match self.agent_filter {
            Some(agent) => format!("only {} · click again for all", agent.display_name()),
            None => "click one to focus on it".to_string(),
        };
        card(
            Flex::column()
                .with_spacing(10.)
                .with_child(card_header("Agents", detail, palette, appearance))
                .with_child(rows.finish())
                .finish(),
            palette,
        )
    }

    fn render_places(
        &self,
        summary: &UsageSummary,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        let total: u64 = summary
            .places
            .iter()
            .map(|place| token_total(&place.totals))
            .sum();
        let segments: Vec<(ColorU, f32)> = summary
            .places
            .iter()
            .map(|place| {
                (
                    place_icon(place.place, palette).1,
                    share(token_total(&place.totals), total),
                )
            })
            .collect();
        let mut chips = Flex::row().with_spacing(10.);
        for stats in &summary.places {
            let (icon, color) = place_icon(stats.place, palette);
            let tokens = token_total(&stats.totals);
            let chip = Flex::column()
                .with_spacing(4.)
                .with_child(
                    Flex::row()
                        .with_spacing(6.)
                        .with_cross_axis_alignment(CrossAxisAlignment::Center)
                        .with_child(icon_element(icon, color))
                        .with_child(text(stats.place.display_name(), font, 12., palette.text))
                        .finish(),
                )
                .with_child(
                    Text::new(percent_label(tokens, total), font, 20.)
                        .with_color(color)
                        .with_style(bold())
                        .finish(),
                )
                .with_child(text(
                    format!(
                        "{} tokens · {}",
                        format_tokens(tokens),
                        format_duration(stats.totals.agent_seconds)
                    ),
                    font,
                    11.,
                    palette.sub,
                ))
                .with_child(text(
                    tasks_label(stats.totals.tasks),
                    font,
                    11.,
                    palette.faint,
                ))
                .finish();
            chips.add_child(
                Expanded::new(
                    1.,
                    Container::new(chip)
                        .with_uniform_padding(12.)
                        .with_background_color(palette.track)
                        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
                        .finish(),
                )
                .finish(),
            );
        }
        card(
            Flex::column()
                .with_spacing(12.)
                .with_child(card_header(
                    "Where they run",
                    "share of tokens".to_string(),
                    palette,
                    appearance,
                ))
                .with_child(segmented_bar(&segments, palette.track))
                .with_child(chips.finish())
                .with_child(text(
                    "Brasa counts agents started in its terminal, and Brasa Code the agent \
                     extensions in folders it opened.",
                    font,
                    11.,
                    palette.faint,
                ))
                .finish(),
            palette,
        )
    }

    fn render_message(
        &self,
        title: &str,
        body: &str,
        palette: &Palette,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        let font = appearance.ui_font_family();
        card(
            Flex::column()
                .with_spacing(6.)
                .with_child(
                    Text::new(title.to_string(), font, 14.)
                        .with_color(palette.text)
                        .with_style(bold())
                        .finish(),
                )
                .with_child(text_wrapped(body, font, 12., palette.sub))
                .finish(),
            palette,
        )
    }
}

fn load_usage() -> UsageData {
    #[cfg(feature = "local_fs")]
    {
        let sources = UsageSources::detect(Some(
            crate::persistence::database_file_path_for_current_scope(),
        ));
        let commands = sources
            .brasa_databases
            .iter()
            .flat_map(|database| super::collect::read_brasa_commands(database))
            .collect();
        collect(&sources, commands)
    }
    #[cfg(not(feature = "local_fs"))]
    collect(&UsageSources::detect(None), Vec::new())
}

const WEEKDAY_LETTERS: [&str; 7] = ["M", "T", "W", "T", "F", "S", "S"];
const WEEKDAY_NAMES: [&str; 7] = [
    "Mondays",
    "Tuesdays",
    "Wednesdays",
    "Thursdays",
    "Fridays",
    "Saturdays",
    "Sundays",
];

#[derive(Clone, Copy)]
struct Palette {
    card: ColorU,
    border: ColorU,
    text: ColorU,
    sub: ColorU,
    faint: ColorU,
    track: ColorU,
    neutral: ColorU,
    green: ColorU,
    yellow: ColorU,
    red: ColorU,
}

impl Palette {
    fn new(appearance: &Appearance) -> Self {
        let theme = appearance.theme();
        let card = theme.surface_1().into_solid();
        let text = blended_colors::text_main(theme, card);
        Self {
            card,
            border: theme.outline().into_solid(),
            text,
            sub: blended_colors::text_sub(theme, card),
            faint: blended_colors::text_disabled(theme, card),
            track: with_alpha(text, 16),
            neutral: with_alpha(text, 110),
            green: theme.ui_green_color(),
            yellow: theme.ui_yellow_color(),
            red: theme.ui_error_color(),
        }
    }

    /// The same palette with every text color faded, for rows that are not selected.
    fn dimmed(&self) -> Self {
        Self {
            text: self.faint,
            sub: self.faint,
            ..*self
        }
    }

    /// Heatmap color for `count` out of `max`, on a square-root scale so quiet hours still show.
    fn heat(&self, count: u64, max: u64) -> ColorU {
        if count == 0 || max == 0 {
            return self.track;
        }
        let level = ((count as f32 / max as f32).sqrt() * 4.)
            .ceil()
            .clamp(1., 4.) as usize;
        [
            self.track,
            with_alpha(BRASA_ORANGE, 80),
            with_alpha(BRASA_ORANGE, 150),
            BRASA_ORANGE,
            BRASA_AMBER,
        ][level]
    }

    /// Green while plenty is left, yellow when it runs low, red when it is nearly gone.
    fn limit_color(&self, left_percent: f32) -> ColorU {
        if left_percent >= 50. {
            self.green
        } else if left_percent >= 20. {
            self.yellow
        } else {
            self.red
        }
    }
}

fn limit_card(
    agent: UsageAgent,
    plan: Option<String>,
    body: Box<dyn Element>,
    footer: Option<Box<dyn Element>>,
    palette: &Palette,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let font = appearance.ui_font_family();
    let (icon, color) = agent_icon(agent, palette);
    let mut header = Flex::row()
        .with_main_axis_size(MainAxisSize::Max)
        .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(
            Flex::row()
                .with_spacing(8.)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(
                    ConstrainedBox::new(icon.to_brasaui_icon(color.into()).finish())
                        .with_width(18.)
                        .with_height(18.)
                        .finish(),
                )
                .with_child(
                    Text::new(format!("{} plan", agent.display_name()), font, 14.)
                        .with_color(palette.text)
                        .with_style(bold())
                        .finish(),
                )
                .finish(),
        );
    if let Some(plan) = plan {
        header.add_child(
            Container::new(text(plan, font, 11., palette.sub))
                .with_padding_left(8.)
                .with_padding_right(8.)
                .with_padding_top(2.)
                .with_padding_bottom(2.)
                .with_border(Border::all(1.).with_border_color(palette.border))
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(10.)))
                .finish(),
        );
    }
    let mut column = Flex::column()
        .with_spacing(12.)
        .with_child(header.finish())
        .with_child(body);
    if let Some(footer) = footer {
        column.add_child(footer);
    }
    card(column.finish(), palette)
}

/// The weekly window in detail, with the short session window underneath.
fn render_plan_limits(
    limits: &PlanLimits,
    palette: &Palette,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let font = appearance.ui_font_family();
    let now = Utc::now();
    let Some(main) = limits.weekly.as_ref().or(limits.session.as_ref()) else {
        return text("No limits reported yet.", font, 12., palette.sub);
    };
    let used = main.used_percent.clamp(0., 100.);
    let left = 100. - used;
    let color = palette.limit_color(left);
    let period = if main.length >= Duration::days(6) {
        "left this week"
    } else {
        "left in this window"
    };
    if main.expired {
        return Flex::column()
            .with_spacing(8.)
            .with_child(
                Flex::row()
                    .with_spacing(8.)
                    .with_cross_axis_alignment(CrossAxisAlignment::End)
                    .with_child(
                        Text::new("100%", font, 34.)
                            .with_color(palette.text)
                            .with_style(bold())
                            .finish(),
                    )
                    .with_child(
                        Container::new(text(period, font, 13., palette.sub))
                            .with_padding_bottom(6.)
                            .finish(),
                    )
                    .finish(),
            )
            .with_child(progress_bar(1., palette.green, palette.track, 8.))
            .with_child(text(
                format!(
                    "Not used since its last reset ({}), so the whole limit is free.",
                    main.resets_at.with_timezone(&Local).format("%b %-d")
                ),
                font,
                12.,
                palette.sub,
            ))
            .finish();
    }

    let mut column = Flex::column()
        .with_spacing(8.)
        .with_child(
            Flex::row()
                .with_spacing(8.)
                .with_cross_axis_alignment(CrossAxisAlignment::End)
                .with_child(
                    Text::new(format!("{left:.0}%"), font, 34.)
                        .with_color(palette.text)
                        .with_style(bold())
                        .finish(),
                )
                .with_child(
                    Container::new(text(period, font, 13., palette.sub))
                        .with_padding_bottom(6.)
                        .finish(),
                )
                .finish(),
        )
        .with_child(progress_bar(left / 100., color, palette.track, 8.))
        .with_child(
            Flex::row()
                .with_main_axis_size(MainAxisSize::Max)
                .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .with_child(text(format!("{used:.0}% used"), font, 12., palette.sub))
                .with_child(text(
                    format!("Resets {}", format_reset(main.resets_at, now)),
                    font,
                    12.,
                    palette.sub,
                ))
                .finish(),
        )
        .with_child(render_window_chart(main, now, color, palette, font));

    let forecast = match main.runs_out_at(now) {
        Some(time) => (
            format!("At this pace it runs out {}", format_reset(time, now)),
            palette.red,
        ),
        None => (
            format!(
                "At this pace, {:.0}% used by the reset",
                main.projected_percent(now).min(100.)
            ),
            palette.sub,
        ),
    };
    column.add_child(text(forecast.0, font, 12., forecast.1));

    if limits.weekly.is_some()
        && let Some(session) = limits.session.as_ref().filter(|session| !session.expired)
    {
        let session_used = session.used_percent.clamp(0., 100.);
        let session_color = palette.limit_color(100. - session_used);
        column.add_child(
            Container::new(
                Flex::row()
                    .with_spacing(10.)
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(text(
                        format!("{} h window", session.length.num_hours()),
                        font,
                        11.,
                        palette.faint,
                    ))
                    .with_child(
                        Expanded::new(
                            1.,
                            progress_bar(
                                1. - session_used / 100.,
                                session_color,
                                palette.track,
                                5.,
                            ),
                        )
                        .finish(),
                    )
                    .with_child(text(
                        format!(
                            "{session_used:.0}% used · resets {}",
                            format_reset(session.resets_at, now)
                        ),
                        font,
                        11.,
                        palette.sub,
                    ))
                    .finish(),
            )
            .with_padding_top(4.)
            .finish(),
        );
    }
    column.finish()
}

/// One bar per day of the window: usage reached so far, and the forecast for the days left.
fn render_window_chart(
    window: &LimitWindow,
    now: DateTime<Utc>,
    color: ColorU,
    palette: &Palette,
    font: FamilyId,
) -> Box<dyn Element> {
    let today = now.with_timezone(&Local).date_naive();
    let mut bars = Flex::row().with_spacing(6.);
    let mut labels = Flex::row().with_spacing(6.);
    for day in window.days(now) {
        let (value, bar_color) = match day.used_percent {
            Some(used) => (used, color),
            None => (day.projected_percent, with_alpha(color, 60)),
        };
        bars.add_child(
            Expanded::new(
                1.,
                vertical_bar(value.clamp(0., 100.) / 100., bar_color, true),
            )
            .finish(),
        );
        let label_color = if day.date == today {
            palette.text
        } else {
            palette.faint
        };
        labels.add_child(
            Expanded::new(
                1.,
                Align::new(text(weekday_letter(day.date), font, 10., label_color)).finish(),
            )
            .finish(),
        );
    }
    Flex::column()
        .with_spacing(4.)
        .with_child(
            Container::new(
                ConstrainedBox::new(bars.finish())
                    .with_height(LIMIT_CHART_HEIGHT)
                    .finish(),
            )
            .with_padding_top(4.)
            .with_border(Border::bottom(1.).with_border_color(palette.border))
            .finish(),
        )
        .with_child(labels.finish())
        .finish()
}

fn weekday_letter(date: NaiveDate) -> &'static str {
    WEEKDAY_LETTERS[date.weekday().num_days_from_monday() as usize]
}

/// `14:30` for resets within a day, otherwise `Fri 28 · 11:50`.
fn format_reset(time: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let local = time.with_timezone(&Local);
    if time - now < Duration::hours(20) {
        local.format("%H:%M").to_string()
    } else {
        local.format("%a %-d · %H:%M").to_string()
    }
}

fn pill_button(
    mouse_state: MouseStateHandle,
    label: &'static str,
    selected: bool,
    action: AgentUsageAction,
    palette: &Palette,
    font: FamilyId,
) -> Box<dyn Element> {
    let palette = *palette;
    Hoverable::new(mouse_state, move |state| {
        let (background, color) = if selected {
            (with_alpha(BRASA_ORANGE, 46), BRASA_AMBER)
        } else if state.is_hovered() {
            (palette.track, palette.text)
        } else {
            (ColorU::transparent_black(), palette.sub)
        };
        Container::new(text(label, font, 12., color))
            .with_padding_left(10.)
            .with_padding_right(10.)
            .with_padding_top(4.)
            .with_padding_bottom(4.)
            .with_background_color(background)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(7.)))
            .finish()
    })
    .with_cursor(Cursor::PointingHand)
    .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
    .finish()
}

fn visibility_chip(
    mouse_state: MouseStateHandle,
    agent: UsageAgent,
    shown: bool,
    palette: &Palette,
    font: FamilyId,
) -> Box<dyn Element> {
    let palette = *palette;
    let (icon, color) = agent_icon(agent, &palette);
    Hoverable::new(mouse_state, move |state| {
        let hovered = state.is_hovered();
        let (background, border, label_color, mark) = if shown {
            (
                with_alpha(BRASA_ORANGE, if hovered { 60 } else { 36 }),
                with_alpha(BRASA_ORANGE, 110),
                palette.text,
                icon_element(Icon::Check, BRASA_AMBER),
            )
        } else {
            (
                if hovered {
                    palette.track
                } else {
                    ColorU::transparent_black()
                },
                palette.border,
                palette.faint,
                icon_element(Icon::Eye, palette.faint),
            )
        };
        Container::new(
            Flex::row()
                .with_spacing(6.)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(mark)
                .with_child(icon_element(
                    icon,
                    if shown { color } else { palette.faint },
                ))
                .with_child(text(agent.display_name(), font, 12., label_color))
                .finish(),
        )
        .with_padding_left(10.)
        .with_padding_right(12.)
        .with_padding_top(5.)
        .with_padding_bottom(5.)
        .with_background_color(background)
        .with_border(Border::all(1.).with_border_color(border))
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(14.)))
        .finish()
    })
    .with_cursor(Cursor::PointingHand)
    .on_click(move |ctx, _, _| ctx.dispatch_typed_action(AgentUsageAction::ToggleHidden(agent)))
    .finish()
}

fn primary_button(
    mouse_state: MouseStateHandle,
    label: &'static str,
    action: AgentUsageAction,
    font: FamilyId,
) -> Box<dyn Element> {
    Hoverable::new(mouse_state, move |state| {
        let background = if state.is_hovered() {
            BRASA_AMBER
        } else {
            BRASA_ORANGE
        };
        Container::new(
            Text::new(label, font, 12.)
                .with_color(ColorU::new(30, 16, 10, 255))
                .with_style(bold())
                .finish(),
        )
        .with_padding_left(14.)
        .with_padding_right(14.)
        .with_padding_top(6.)
        .with_padding_bottom(6.)
        .with_background_color(background)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(7.)))
        .finish()
    })
    .with_cursor(Cursor::PointingHand)
    .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
    .finish()
}

fn link_button(
    mouse_state: MouseStateHandle,
    label: &'static str,
    action: AgentUsageAction,
    palette: &Palette,
    font: FamilyId,
) -> Box<dyn Element> {
    let palette = *palette;
    Hoverable::new(mouse_state, move |state| {
        let color = if state.is_hovered() {
            palette.text
        } else {
            palette.faint
        };
        text(label, font, 11., color)
    })
    .with_cursor(Cursor::PointingHand)
    .on_click(move |ctx, _, _| ctx.dispatch_typed_action(action.clone()))
    .finish()
}

fn change_badge(change: i64, palette: &Palette, font: FamilyId) -> Box<dyn Element> {
    let (color, arrow) = if change >= 0 {
        (palette.green, "▲")
    } else {
        (palette.red, "▼")
    };
    text(format!("{arrow} {}%", change.abs()), font, 10., color)
}

fn bold() -> Properties {
    Properties {
        weight: Weight::Bold,
        ..Default::default()
    }
}

fn with_alpha(color: ColorU, alpha: u8) -> ColorU {
    ColorU { a: alpha, ..color }
}

fn text(value: impl Into<String>, font: FamilyId, size: f32, color: ColorU) -> Box<dyn Element> {
    Text::new(value.into(), font, size)
        .with_color(color)
        .soft_wrap(false)
        .with_clip(ClipConfig::end())
        .finish()
}

fn text_wrapped(value: &str, font: FamilyId, size: f32, color: ColorU) -> Box<dyn Element> {
    Text::new(value.to_string(), font, size)
        .with_color(color)
        .finish()
}

fn overline(label: &str, palette: &Palette, appearance: &Appearance) -> Box<dyn Element> {
    Text::new(
        label.to_uppercase(),
        appearance.overline_font_family(),
        appearance.overline_font_size() + 1.,
    )
    .with_color(palette.faint)
    .soft_wrap(false)
    .finish()
}

fn card(content: Box<dyn Element>, palette: &Palette) -> Box<dyn Element> {
    Container::new(content)
        .with_uniform_padding(CARD_PADDING)
        .with_background_color(palette.card)
        .with_border(Border::all(1.).with_border_color(palette.border))
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(12.)))
        .finish()
}

fn card_header(
    label: &str,
    detail: String,
    palette: &Palette,
    appearance: &Appearance,
) -> Box<dyn Element> {
    Flex::row()
        .with_main_axis_size(MainAxisSize::Max)
        .with_cross_axis_alignment(CrossAxisAlignment::Center)
        .with_child(overline(label, palette, appearance))
        .with_child(
            Expanded::new(
                1.,
                Align::new(text(
                    detail,
                    appearance.ui_font_family(),
                    11.,
                    palette.faint,
                ))
                .right()
                .finish(),
            )
            .finish(),
        )
        .finish()
}

/// A row of a ranking: name and details, its share, and a bar.
#[allow(clippy::too_many_arguments)]
fn leaderboard_row(
    icon: Box<dyn Element>,
    name: String,
    detail: String,
    share_label: String,
    bar_fraction: f32,
    color: ColorU,
    palette: &Palette,
    font: FamilyId,
) -> Box<dyn Element> {
    Flex::column()
        .with_spacing(6.)
        .with_child(
            Flex::row()
                .with_spacing(8.)
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_child(ConstrainedBox::new(icon).with_width(ICON_SIZE).finish())
                .with_child(text(name, font, 13., palette.text))
                .with_child(Expanded::new(1., text(detail, font, 11., palette.faint)).finish())
                .with_child(
                    Text::new(share_label, font, 12.)
                        .with_color(palette.text)
                        .with_style(bold())
                        .finish(),
                )
                .finish(),
        )
        .with_child(progress_bar(bar_fraction, color, palette.track, 5.))
        .finish()
}

fn swatch(color: ColorU, radius: f32) -> Box<dyn Element> {
    Container::new(Empty::new().finish())
        .with_background_color(color)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(radius)))
        .finish()
}

/// Share of `max`, with a visible minimum so small but non-zero values still show.
fn fraction(value: u64, max: u64) -> f32 {
    if value == 0 || max == 0 {
        0.
    } else {
        (value as f32 / max as f32).clamp(0.03, 1.)
    }
}

fn share(value: u64, total: u64) -> f32 {
    if total == 0 {
        0.
    } else {
        value as f32 / total as f32
    }
}

fn percent_of(value: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        (value * 100 + total / 2) / total
    }
}

/// `42%`, or `<1%` for a share too small to round up to 1%.
fn percent_label(value: u64, total: u64) -> String {
    match percent_of(value, total) {
        0 if value > 0 => "<1%".to_string(),
        percent => format!("{percent}%"),
    }
}

/// `1 task` or `12 tasks`.
fn tasks_label(tasks: u64) -> String {
    match tasks {
        1 => "1 task".to_string(),
        tasks => format!("{} tasks", format_count(tasks)),
    }
}

/// A bar filling `fraction` of its parent's height, from the bottom or from the top.
fn vertical_bar(fraction: f32, color: ColorU, from_bottom: bool) -> Box<dyn Element> {
    let mut column = Flex::column();
    if fraction <= 0. {
        column.add_child(Expanded::new(1., Empty::new().finish()).finish());
        return column.finish();
    }
    let bar = Expanded::new(fraction, swatch(color, 3.)).finish();
    let gap = (fraction < 1.).then(|| Expanded::new(1. - fraction, Empty::new().finish()).finish());
    match (gap, from_bottom) {
        (Some(gap), true) => column.extend([gap, bar]),
        (Some(gap), false) => column.extend([bar, gap]),
        (None, _) => column.add_child(bar),
    }
    column.finish()
}

fn progress_bar(fraction: f32, color: ColorU, track: ColorU, height: f32) -> Box<dyn Element> {
    let fraction = fraction.clamp(0., 1.);
    let mut row = Flex::row();
    if fraction > 0. {
        row.add_child(Expanded::new(fraction, swatch(color, height / 2.)).finish());
    }
    if fraction < 1. {
        row.add_child(Expanded::new(1. - fraction, swatch(track, height / 2.)).finish());
    }
    ConstrainedBox::new(row.finish())
        .with_height(height)
        .finish()
}

fn segmented_bar(segments: &[(ColorU, f32)], track: ColorU) -> Box<dyn Element> {
    let mut row = Flex::row().with_spacing(3.);
    let visible: Vec<_> = segments.iter().filter(|(_, share)| *share > 0.).collect();
    if visible.is_empty() {
        row.add_child(Expanded::new(1., swatch(track, 4.)).finish());
    }
    for (color, share) in visible {
        row.add_child(Expanded::new(share.max(0.01), swatch(*color, 4.)).finish());
    }
    ConstrainedBox::new(row.finish()).with_height(8.).finish()
}

fn icon_element(icon: Icon, color: ColorU) -> Box<dyn Element> {
    ConstrainedBox::new(icon.to_brasaui_icon(color.into()).finish())
        .with_width(ICON_SIZE)
        .with_height(ICON_SIZE)
        .finish()
}

/// The chosen metric per bar; long ranges are grouped by week.
fn chart_buckets(days: &[DayStats], metric: ChartMetric) -> Vec<u64> {
    let size = if days.len() > MAX_DAILY_BARS { 7 } else { 1 };
    days.chunks(size)
        .map(|chunk| chunk.iter().map(|day| metric.value(&day.totals)).sum())
        .collect()
}

fn provider_color(provider: Provider) -> ColorU {
    match provider {
        Provider::Anthropic => ColorU::new(217, 119, 87, 255),
        Provider::OpenAi => ColorU::new(16, 163, 127, 255),
        Provider::Google => ColorU::new(66, 133, 244, 255),
        Provider::Xai => ColorU::new(150, 150, 150, 255),
        Provider::Other => ColorU::new(160, 120, 220, 255),
    }
}

fn provider_icon(provider: Provider) -> Box<dyn Element> {
    let color = provider_color(provider);
    match provider {
        Provider::Anthropic => icon_element(Icon::ClaudeLogo, color),
        Provider::OpenAi => icon_element(Icon::OpenAILogo, color),
        Provider::Google => icon_element(Icon::GeminiEnterpriseAgentPlatform, color),
        Provider::Xai => icon_element(Icon::GrokLogo, color),
        Provider::Other => ConstrainedBox::new(swatch(color, 4.))
            .with_width(8.)
            .with_height(8.)
            .finish(),
    }
}

fn hidden_agents(ctx: &AppContext) -> HashSet<UsageAgent> {
    AISettings::as_ref(ctx)
        .agent_usage_hidden_agents
        .iter()
        .filter_map(|id| UsageAgent::from_id(id))
        .collect()
}

fn agent_icon(agent: UsageAgent, palette: &Palette) -> (Icon, ColorU) {
    let cli_agent = match agent {
        UsageAgent::ClaudeCode => CLIAgent::Claude,
        UsageAgent::Codex => CLIAgent::Codex,
        UsageAgent::GeminiCli => CLIAgent::Gemini,
        UsageAgent::OpenCode => CLIAgent::OpenCode,
        UsageAgent::Antigravity => CLIAgent::Antigravity,
        UsageAgent::GrokBuild => CLIAgent::Grok,
        UsageAgent::LocalAi => return (Icon::Brasita, BRASA_ORANGE),
    };
    let color = match agent {
        UsageAgent::ClaudeCode | UsageAgent::GeminiCli => {
            cli_agent.brand_color().unwrap_or(palette.text)
        }
        UsageAgent::Codex
        | UsageAgent::OpenCode
        | UsageAgent::Antigravity
        | UsageAgent::GrokBuild
        | UsageAgent::LocalAi => palette.text,
    };
    (cli_agent.icon().unwrap_or(Icon::Terminal), color)
}

fn place_icon(place: UsagePlace, palette: &Palette) -> (Icon, ColorU) {
    match place {
        UsagePlace::Brasa => (Icon::Brasita, BRASA_ORANGE),
        UsagePlace::BrasaCode => (Icon::InlineCode, BRASA_AMBER),
        UsagePlace::OtherTerminal => (Icon::Terminal, palette.neutral),
        UsagePlace::OtherEditor => (Icon::Code2, palette.neutral),
        UsagePlace::DesktopApp => (Icon::Laptop, palette.neutral),
    }
}

impl Entity for AgentUsageView {
    type Event = AgentUsageViewEvent;
}

impl View for AgentUsageView {
    fn ui_name() -> &'static str {
        "AgentUsageView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();
        let palette = Palette::new(appearance);
        let mut page = Flex::column()
            .with_spacing(GAP)
            .with_child(self.render_header(&palette, appearance));
        if self.customizing {
            page.add_child(self.render_customize(&palette, appearance));
        }
        if let Some(limits) = self.render_limits(&palette, appearance) {
            page.add_child(limits);
        }

        let has_history = self
            .data
            .as_ref()
            .is_some_and(|data| !data.sessions.is_empty());
        let has_visible_history = self.data.as_ref().is_some_and(|data| {
            data.sessions
                .iter()
                .any(|session| !self.is_hidden(session.agent))
        });
        match &self.summary {
            None => page.add_child(self.render_message(
                "Reading your agents' history…",
                "Brasa reads the files your agents keep on this computer.",
                &palette,
                appearance,
            )),
            Some(_) if !has_history => page.add_child(self.render_message(
                "No agent history yet",
                "Brasa reads what Claude Code, Codex, Gemini CLI, Antigravity, Grok Build, opencode \
                 and its local AI keep on this computer. Use any of them and come back.",
                &palette,
                appearance,
            )),
            Some(_) if !has_visible_history => page.add_child(self.render_message(
                "Every agent is hidden",
                "Open Customize to show some of them again.",
                &palette,
                appearance,
            )),
            Some(summary) => {
                page.add_child(self.render_summary(summary, &palette, appearance));
                page.add_child(
                    EqualHeightRow::new(GAP, MIN_CARD_WIDTH)
                        .with_child(3., self.render_activity(summary, &palette, appearance))
                        .with_child(2., self.render_heatmap(summary, &palette, appearance))
                        .finish(),
                );
                page.add_child(
                    EqualHeightRow::new(GAP, MIN_CARD_WIDTH)
                        .with_child(1., self.render_agents(summary, &palette, appearance))
                        .with_child(1., self.render_places(summary, &palette, appearance))
                        .finish(),
                );
                page.add_child(self.render_models(summary, &palette, appearance));
            }
        }

        let content = Container::new(page.finish())
            .with_uniform_padding(PAGE_PADDING)
            .finish();
        Container::new(
            ClippedScrollable::vertical(
                self.scroll_state.clone(),
                content,
                ScrollbarWidth::Auto,
                theme.nonactive_ui_detail().into(),
                theme.active_ui_detail().into(),
                UiFill::None,
            )
            .finish(),
        )
        .with_background(theme.background())
        .finish()
    }
}

impl TypedActionView for AgentUsageView {
    type Action = AgentUsageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            AgentUsageAction::SetRange(range) => self.range = *range,
            AgentUsageAction::ToggleAgent(agent) => {
                self.agent_filter = (self.agent_filter != Some(*agent)).then_some(*agent);
            }
            AgentUsageAction::SetChartMetric(metric) => {
                self.chart_metric = *metric;
                ctx.notify();
                return;
            }
            AgentUsageAction::SetClaudeLimits(enabled) => {
                let enabled = *enabled;
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings.agent_usage_claude_limits.set_value(enabled, ctx) {
                        log::warn!("Could not save the Claude limits setting: {err}");
                    }
                });
                return;
            }
            AgentUsageAction::ToggleCustomize => {
                self.customizing = !self.customizing;
                ctx.notify();
                return;
            }
            AgentUsageAction::ToggleHidden(agent) => {
                let agent = *agent;
                if !self.hidden_agents.remove(&agent) {
                    self.hidden_agents.insert(agent);
                }
                let ids: Vec<String> = all::<UsageAgent>()
                    .filter(|agent| self.hidden_agents.contains(agent))
                    .map(|agent| agent.id().to_string())
                    .collect();
                AISettings::handle(ctx).update(ctx, |settings, ctx| {
                    if let Err(err) = settings.agent_usage_hidden_agents.set_value(ids, ctx) {
                        log::warn!("Could not save the hidden agents: {err}");
                    }
                });
            }
            AgentUsageAction::Refresh => {
                self.reload(ctx);
                self.refresh_claude_limits(ctx);
                return;
            }
        }
        self.resummarize();
        ctx.notify();
    }
}

impl BackingView for AgentUsageView {
    type PaneHeaderOverflowMenuAction = AgentUsageAction;
    type CustomAction = AgentUsageAction;
    type AssociatedData = ();

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        action: &Self::PaneHeaderOverflowMenuAction,
        ctx: &mut ViewContext<Self>,
    ) {
        self.handle_action(action, ctx);
    }

    fn handle_custom_action(&mut self, action: &Self::CustomAction, ctx: &mut ViewContext<Self>) {
        self.handle_action(action, ctx);
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.emit(AgentUsageViewEvent::Pane(PaneEvent::Close));
    }

    fn focus_contents(&mut self, ctx: &mut ViewContext<Self>) {
        self.focus(ctx);
    }

    fn render_header_content(
        &self,
        _ctx: &view::HeaderRenderContext<'_>,
        _app: &AppContext,
    ) -> HeaderContent {
        HeaderContent::Standard(StandardHeader {
            title: AGENT_USAGE_TITLE.to_string(),
            title_secondary: None,
            title_style: None,
            title_clip_config: ClipConfig::start(),
            title_max_width: None,
            left_of_title: None,
            right_of_title: None,
            left_of_overflow: None,
            options: StandardHeaderOptions::default(),
        })
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, _ctx: &mut ViewContext<Self>) {
        self.focus_handle = Some(focus_handle);
    }
}
