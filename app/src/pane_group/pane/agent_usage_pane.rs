use brasaui::{AppContext, ModelHandle, SingletonEntity, View, ViewContext, ViewHandle};

use super::view::PaneView;
use super::{
    DetachType, PaneConfiguration, PaneContent, PaneGroup, PaneId, ShareableLink,
    ShareableLinkError,
};
use crate::agent_usage::{AgentUsagePaneManager, AgentUsageView, AgentUsageViewEvent};
use crate::app_state::LeafContents;
use crate::workspace::PaneViewLocator;

pub struct AgentUsagePane {
    view: ViewHandle<PaneView<AgentUsageView>>,
    pane_configuration: ModelHandle<PaneConfiguration>,
}

impl AgentUsagePane {
    pub fn from_view(agent_usage_view: ViewHandle<AgentUsageView>, ctx: &mut AppContext) -> Self {
        let pane_configuration = agent_usage_view.as_ref(ctx).pane_configuration();

        let view = ctx.add_typed_action_view(agent_usage_view.window_id(ctx), |ctx| {
            let pane_id = PaneId::from_agent_usage_pane_ctx(ctx);
            PaneView::new(
                pane_id,
                agent_usage_view,
                (),
                pane_configuration.clone(),
                ctx,
            )
        });

        Self {
            view,
            pane_configuration,
        }
    }

    pub fn new<V: View>(ctx: &mut ViewContext<V>) -> Self {
        let view = ctx.add_typed_action_view(AgentUsageView::new);
        Self::from_view(view, ctx)
    }

    pub fn agent_usage_view(&self, ctx: &AppContext) -> ViewHandle<AgentUsageView> {
        self.view.as_ref(ctx).child(ctx)
    }
}

impl PaneContent for AgentUsagePane {
    fn id(&self) -> PaneId {
        PaneId::from_agent_usage_pane_view(&self.view)
    }

    fn attach(
        &self,
        _group: &PaneGroup,
        focus_handle: crate::pane_group::focus_state::PaneFocusHandle,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        self.view
            .update(ctx, |view, ctx| view.set_focus_handle(focus_handle, ctx));

        let agent_usage_view = self.agent_usage_view(ctx);
        let pane_id = self.id();
        let pane_group_id = ctx.view_id();
        let window_id = ctx.window_id();

        ctx.subscribe_to_view(&agent_usage_view, move |pane_group, _, event, ctx| {
            let AgentUsageViewEvent::Pane(pane_event) = event;
            pane_group.handle_pane_event(pane_id, pane_event, ctx)
        });
        ctx.subscribe_to_view(&self.view, move |group, _, event, ctx| {
            group.handle_pane_view_event(pane_id, event, ctx);
        });

        AgentUsagePaneManager::handle(ctx).update(ctx, |manager, _ctx| {
            manager.register_pane(
                window_id,
                PaneViewLocator {
                    pane_group_id,
                    pane_id,
                },
            );
        });
    }

    fn detach(
        &self,
        _group: &PaneGroup,
        _detach_type: DetachType,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        // Always unsubscribe from views.
        let agent_usage_view = self.agent_usage_view(ctx);
        ctx.unsubscribe_to_view(&agent_usage_view);
        ctx.unsubscribe_to_view(&self.view);

        // Always deregister from the manager.
        let window_id = ctx.window_id();
        AgentUsagePaneManager::handle(ctx).update(ctx, |manager, _| {
            manager.deregister_pane(&window_id);
        });
    }

    fn snapshot(&self, _app: &AppContext) -> LeafContents {
        LeafContents::AgentUsage
    }

    fn has_application_focus(&self, ctx: &mut ViewContext<PaneGroup>) -> bool {
        self.view.is_self_or_child_focused(ctx)
    }

    fn focus(&self, ctx: &mut ViewContext<PaneGroup>) {
        self.agent_usage_view(ctx)
            .update(ctx, |view, ctx| view.focus(ctx));
    }

    fn shareable_link(
        &self,
        _ctx: &mut ViewContext<PaneGroup>,
    ) -> Result<ShareableLink, ShareableLinkError> {
        Ok(ShareableLink::Base)
    }

    fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    fn is_pane_being_dragged(&self, ctx: &AppContext) -> bool {
        self.view.as_ref(ctx).is_being_dragged()
    }
}
