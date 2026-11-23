//! A row whose children all take the height of the tallest one. When the row is too narrow for
//! its children, they stack vertically instead.

use brasaui::elements::Point;
use brasaui::event::DispatchedEvent;
use brasaui::geometry::vector::{Vector2F, vec2f};
use brasaui::{
    AfterLayoutContext, AppContext, Element, EventContext, LayoutContext, PaintContext,
    SizeConstraint,
};

pub struct EqualHeightRow {
    /// Each child with its share of the row's width.
    children: Vec<(f32, Box<dyn Element>)>,
    spacing: f32,
    /// Below this width per child, the children stack vertically.
    min_child_width: f32,
    stacked: bool,
    size: Option<Vector2F>,
    origin: Option<Point>,
}

impl EqualHeightRow {
    pub fn new(spacing: f32, min_child_width: f32) -> Self {
        Self {
            children: Vec::new(),
            spacing,
            min_child_width,
            stacked: false,
            size: None,
            origin: None,
        }
    }

    pub fn with_child(mut self, flex: f32, child: Box<dyn Element>) -> Self {
        self.children.push((flex, child));
        self
    }

    pub fn finish(self) -> Box<dyn Element> {
        Box::new(self)
    }

    fn child_widths(&self, width: f32) -> Vec<f32> {
        let gaps = self.spacing * self.children.len().saturating_sub(1) as f32;
        let total_flex: f32 = self.children.iter().map(|(flex, _)| flex).sum();
        self.children
            .iter()
            .map(|(flex, _)| (width - gaps).max(0.) * flex / total_flex.max(f32::EPSILON))
            .collect()
    }
}

impl Element for EqualHeightRow {
    fn layout(
        &mut self,
        constraint: SizeConstraint,
        ctx: &mut LayoutContext,
        app: &AppContext,
    ) -> Vector2F {
        let width = constraint.max.x();
        if self.children.is_empty() || !width.is_finite() {
            self.size = Some(Vector2F::zero());
            return Vector2F::zero();
        }
        let widths = self.child_widths(width);
        self.stacked = widths
            .iter()
            .any(|child_width| *child_width < self.min_child_width);

        let size = if self.stacked {
            let mut height = 0.;
            for (index, (_, child)) in self.children.iter_mut().enumerate() {
                if index > 0 {
                    height += self.spacing;
                }
                let child_constraint =
                    SizeConstraint::new(vec2f(width, 0.), vec2f(width, f32::INFINITY));
                height += child.layout(child_constraint, ctx, app).y();
            }
            vec2f(width, height)
        } else {
            let mut height = 0f32;
            for ((_, child), child_width) in self.children.iter_mut().zip(&widths) {
                let child_constraint = SizeConstraint::new(
                    vec2f(*child_width, 0.),
                    vec2f(*child_width, f32::INFINITY),
                );
                height = height.max(child.layout(child_constraint, ctx, app).y());
            }
            for ((_, child), child_width) in self.children.iter_mut().zip(&widths) {
                let tight = vec2f(*child_width, height);
                child.layout(SizeConstraint::new(tight, tight), ctx, app);
            }
            vec2f(width, height)
        };
        self.size = Some(size);
        size
    }

    fn after_layout(&mut self, ctx: &mut AfterLayoutContext, app: &AppContext) {
        for (_, child) in &mut self.children {
            child.after_layout(ctx, app);
        }
    }

    fn paint(&mut self, origin: Vector2F, ctx: &mut PaintContext, app: &AppContext) {
        self.origin = Some(Point::from_vec2f(origin, ctx.scene.z_index()));
        let mut current = origin;
        for (_, child) in &mut self.children {
            child.paint(current, ctx, app);
            let child_size = child.size().unwrap_or_default();
            current += if self.stacked {
                vec2f(0., child_size.y() + self.spacing)
            } else {
                vec2f(child_size.x() + self.spacing, 0.)
            };
        }
    }

    fn size(&self) -> Option<Vector2F> {
        self.size
    }

    fn origin(&self) -> Option<Point> {
        self.origin
    }

    fn dispatch_event(
        &mut self,
        event: &DispatchedEvent,
        ctx: &mut EventContext,
        app: &AppContext,
    ) -> bool {
        let mut handled = false;
        for (_, child) in &mut self.children {
            handled |= child.dispatch_event(event, ctx, app);
        }
        handled
    }
}
