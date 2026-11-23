//! A dashboard of the tokens the AI agents on this computer spend, when and where.

pub mod collect;
mod equal_height_row;
pub mod limits;
pub mod stats;
mod view;

use std::collections::HashMap;

use brasaui::{Entity, SingletonEntity, WindowId};
pub use view::{AgentUsageView, AgentUsageViewEvent};

use crate::workspace::PaneViewLocator;

/// Tracks the agent usage pane open in each window, so opening it again focuses that one.
#[derive(Default)]
pub struct AgentUsagePaneManager {
    panes: HashMap<WindowId, PaneViewLocator>,
}

impl AgentUsagePaneManager {
    pub fn find_pane(&self, window_id: WindowId) -> Option<PaneViewLocator> {
        self.panes.get(&window_id).copied()
    }

    pub fn register_pane(&mut self, window_id: WindowId, locator: PaneViewLocator) {
        self.panes.insert(window_id, locator);
    }

    pub fn deregister_pane(&mut self, window_id: &WindowId) {
        self.panes.remove(window_id);
    }
}

impl Entity for AgentUsagePaneManager {
    type Event = ();
}

impl SingletonEntity for AgentUsagePaneManager {}
