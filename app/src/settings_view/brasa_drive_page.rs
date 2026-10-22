use brasa_core::features::FeatureFlag;
use brasa_core::settings::ToggleableSetting as _;
use brasa_errors::report_if_error;
use brasaui::elements::{
    Container, Element, Flex, MouseStateHandle, ParentElement, Shrinkable, Text,
};
use brasaui::fonts::Weight;
use brasaui::keymap::ContextPredicate;
use brasaui::ui_components::button::ButtonVariant;
use brasaui::ui_components::components::{Coords, UiComponent, UiComponentStyles};
use brasaui::ui_components::switch::SwitchStateHandle;
use brasaui::{
    Action, AppContext, Entity, SingletonEntity, TypedActionView, View, ViewContext, ViewHandle, id,
};

use super::settings_page::{
    AdditionalInfo, MatchData, PageType, SettingsPageMeta, SettingsPageViewHandle, SettingsWidget,
    render_body_item,
};
use super::{
    LocalOnlyIconState, SettingActionPairContexts, SettingActionPairDescriptions, SettingsAction,
    SettingsSection, ToggleSettingActionPair, ToggleState, flags,
};
use crate::appearance::Appearance;
use crate::auth::AuthStateProvider;
use crate::auth::auth_manager::{AuthManager, AuthManagerEvent};
use crate::drive::settings::BrasaDriveSettings;

#[derive(Debug, Clone)]
pub enum BrasaDriveSettingsPageAction {
    ToggleShowBrasaDrive,
    SignUp,
    OpenUrl(String),
}

pub fn init_actions_from_parent_view<T: Action + Clone>(
    app: &mut AppContext,
    context: &ContextPredicate,
    builder: fn(SettingsAction) -> T,
) {
    ToggleSettingActionPair::add_toggle_setting_action_pairs_as_bindings(
        vec![ToggleSettingActionPair::custom(
            SettingActionPairDescriptions::new("Enable Brasa Drive", "Disable Brasa Drive"),
            builder(SettingsAction::BrasaDrive(
                BrasaDriveSettingsPageAction::ToggleShowBrasaDrive,
            )),
            SettingActionPairContexts::new(
                context.clone() & !id!(flags::ENABLE_BRASA_DRIVE) & !id!("IsAnonymousUser"),
                context.clone() & id!(flags::ENABLE_BRASA_DRIVE) & !id!("IsAnonymousUser"),
            ),
            None,
        )],
        app,
    );
}

pub enum BrasaDriveSettingsPageEvent {
    SignUp,
}

pub struct BrasaDriveSettingsPageView {
    page: PageType<Self>,
}

impl BrasaDriveSettingsPageView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        ctx.subscribe_to_model(&AuthManager::handle(ctx), |_, _, event, ctx| {
            if matches!(event, AuthManagerEvent::AuthComplete) {
                ctx.notify();
            }
        });
        Self {
            page: PageType::new_uncategorized(
                vec![
                    Box::new(BrasaDriveHeaderWidget::default()),
                    Box::new(BrasaDriveToggleWidget::default()),
                ],
                None,
            ),
        }
    }
}

impl Entity for BrasaDriveSettingsPageView {
    type Event = BrasaDriveSettingsPageEvent;
}

impl TypedActionView for BrasaDriveSettingsPageView {
    type Action = BrasaDriveSettingsPageAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            BrasaDriveSettingsPageAction::ToggleShowBrasaDrive => {
                BrasaDriveSettings::handle(ctx).update(ctx, |settings, ctx| {
                    report_if_error!(settings.enable_brasa_drive.toggle_and_save_value(ctx));
                });
                ctx.notify();
            }
            BrasaDriveSettingsPageAction::SignUp => {
                ctx.emit(BrasaDriveSettingsPageEvent::SignUp);
            }
            BrasaDriveSettingsPageAction::OpenUrl(url) => {
                ctx.open_url(url.as_str());
            }
        }
    }
}

impl View for BrasaDriveSettingsPageView {
    fn ui_name() -> &'static str {
        "BrasaDrivePage"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        self.page.render(self, app)
    }
}

impl SettingsPageMeta for BrasaDriveSettingsPageView {
    fn section() -> SettingsSection {
        SettingsSection::BrasaDrive
    }

    fn should_render(&self, _ctx: &AppContext) -> bool {
        true
    }

    fn update_filter(&mut self, query: &str, ctx: &mut ViewContext<Self>) -> MatchData {
        self.page.update_filter(query, ctx)
    }

    fn scroll_to_widget(&mut self, widget_id: &'static str) {
        self.page.scroll_to_widget(widget_id)
    }

    fn clear_highlighted_widget(&mut self) {
        self.page.clear_highlighted_widget();
    }
}

impl From<ViewHandle<BrasaDriveSettingsPageView>> for SettingsPageViewHandle {
    fn from(view_handle: ViewHandle<BrasaDriveSettingsPageView>) -> Self {
        SettingsPageViewHandle::BrasaDrive(view_handle)
    }
}

#[derive(Default)]
struct BrasaDriveHeaderWidget {
    sign_up_button: MouseStateHandle,
}

impl SettingsWidget for BrasaDriveHeaderWidget {
    type View = BrasaDriveSettingsPageView;

    fn search_terms(&self) -> &str {
        "brasa drive sign up"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        FeatureFlag::SkipFirebaseAnonymousUser.is_enabled()
            && AuthStateProvider::as_ref(app)
                .get()
                .is_anonymous_or_logged_out()
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        _app: &AppContext,
    ) -> Box<dyn Element> {
        let ui_builder = appearance.ui_builder();

        let message = Container::new(
            Text::new_inline(
                "To use Brasa Drive, please create an account.".to_string(),
                appearance.ui_font_family(),
                14.,
            )
            .with_color(
                appearance
                    .theme()
                    .sub_text_color(appearance.theme().surface_2())
                    .into_solid(),
            )
            .finish(),
        )
        .with_margin_right(16.)
        .finish();

        let button = Container::new(
            ui_builder
                .button(ButtonVariant::Accent, self.sign_up_button.clone())
                .with_style(UiComponentStyles {
                    font_size: Some(14.),
                    font_weight: Some(Weight::Semibold),
                    border_radius: Some(brasaui::elements::CornerRadius::with_all(
                        brasaui::elements::Radius::Pixels(4.),
                    )),
                    padding: Some(Coords {
                        top: 8.,
                        bottom: 8.,
                        left: 24.,
                        right: 24.,
                    }),
                    ..Default::default()
                })
                .with_text_label("Sign up".to_owned())
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(BrasaDriveSettingsPageAction::SignUp);
                })
                .finish(),
        )
        .finish();

        Container::new(
            Flex::row()
                .with_cross_axis_alignment(brasaui::elements::CrossAxisAlignment::Center)
                .with_child(Shrinkable::new(1., message).finish())
                .with_child(button)
                .finish(),
        )
        .with_padding_bottom(15.)
        .finish()
    }
}

#[derive(Default)]
struct BrasaDriveToggleWidget {
    switch_state: SwitchStateHandle,
    info_icon_mouse_state: MouseStateHandle,
}

impl SettingsWidget for BrasaDriveToggleWidget {
    type View = BrasaDriveSettingsPageView;

    fn search_terms(&self) -> &str {
        "brasa drive tools panel command palette search workflows prompts notebooks environment variables"
    }

    fn should_render(&self, app: &AppContext) -> bool {
        BrasaDriveSettings::is_brasa_drive_available(app)
    }

    fn render(
        &self,
        _view: &Self::View,
        appearance: &Appearance,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let settings = BrasaDriveSettings::as_ref(app);

        render_body_item::<BrasaDriveSettingsPageAction>(
            "Brasa Drive".into(),
            Some(AdditionalInfo {
                mouse_state: self.info_icon_mouse_state.clone(),
                on_click_action: Some(BrasaDriveSettingsPageAction::OpenUrl(
                    "https://github.com/SifG22/brasa".to_string(),
                )),
                secondary_text: None,
                tooltip_override_text: None,
            }),
            LocalOnlyIconState::Hidden,
            ToggleState::Enabled,
            appearance,
            appearance
                .ui_builder()
                .switch(self.switch_state.clone())
                .check(*settings.enable_brasa_drive)
                .build()
                .on_click(|ctx, _, _| {
                    ctx.dispatch_typed_action(BrasaDriveSettingsPageAction::ToggleShowBrasaDrive);
                })
                .finish(),
            Some("Brasa Drive is a workspace in your terminal where you can save Workflows, Notebooks, Prompts, and Environment Variables for personal use or to share with a team.".into()),
        )
    }
}
