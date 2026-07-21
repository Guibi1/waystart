use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    Anchor, AnyElement, App, AppContext, Bounds, Context, DismissEvent, DispatchPhase, Element,
    ElementId, Entity, EventEmitter, FocusHandle, Focusable, GlobalElementId, Hitbox,
    InteractiveElement as _, IntoElement, KeyBinding, LayoutId, ManagedView, MouseButton,
    MouseDownEvent, ParentElement, Pixels, Refineable, Render, SharedString,
    StatefulInteractiveElement, Style, StyleRefinement, Styled, Window, actions, anchored,
    deferred, div, prelude::FluentBuilder as _, px,
};

use crate::config::Config;
use crate::ui::elements::Icon;

actions!(dropdown, [SelectPrev, SelectNext, Confirm, Cancel]);
const CONTEXT: &str = "Dropdown";

pub(super) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, Some(CONTEXT)),
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("shift-tab", SelectPrev, Some(CONTEXT)),
        KeyBinding::new("tab", SelectNext, Some(CONTEXT)),
        KeyBinding::new("enter", Confirm, Some(CONTEXT)),
        KeyBinding::new("escape", Cancel, Some(CONTEXT)),
    ])
}

pub struct DropdownContent {
    selected: usize,
    style: StyleRefinement,
    item_style: StyleRefinement,
    focus_handle: FocusHandle,
    items: Vec<DropdownItem>,
}

impl DropdownContent {
    pub fn new(cx: &mut App) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            selected: 0,
            style: StyleRefinement::default(),
            item_style: StyleRefinement::default(),
            focus_handle,
            items: Vec::new(),
        }
    }

    pub fn item<A>(
        mut self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        icon: Option<Icon>,
        action: A,
    ) -> Self
    where
        A: 'static + Fn(&mut Window, &mut App),
    {
        self.items.push(DropdownItem {
            id: id.into(),
            label: label.into(),
            icon,
            action: Rc::new(action),
            separate: false,
        });
        self
    }

    #[allow(dead_code)]
    fn item_style<S>(mut self, f: S) -> Self
    where
        S: FnOnce(&mut StyleRefinement),
    {
        f(&mut self.item_style);
        self
    }

    #[allow(dead_code)]
    pub fn separate(mut self) -> Self {
        if let Some(item) = self.items.last_mut() {
            item.separate = true;
        }
        self
    }
}

impl EventEmitter<DismissEvent> for DropdownContent {}

impl Focusable for DropdownContent {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Styled for DropdownContent {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Render for DropdownContent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.global::<Config>();

        div()
            .flex()
            .flex_col()
            .p_1()
            .bg(config.theme.background)
            .text_color(config.theme.foreground)
            .border_color(config.theme.accent)
            .border_1()
            .rounded_md()
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .key_context(CONTEXT)
            .map(|mut this| {
                this.style().refine(&self.style);
                this
            })
            .on_action(cx.listener(|_, _: &Cancel, _, cx| cx.emit(DismissEvent)))
            .on_action(cx.listener(|this, _: &SelectPrev, _, cx| {
                this.selected = if this.selected == 0 {
                    this.items.len().saturating_sub(1)
                } else {
                    this.selected.saturating_sub(1)
                };
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &SelectNext, _, cx| {
                this.selected = if this.selected == this.items.len().saturating_sub(1) {
                    0
                } else {
                    this.selected + 1
                };
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Confirm, window, cx| {
                if let Some(item) = this.items.get(this.selected) {
                    (item.action)(window, cx);
                    cx.emit(DismissEvent);
                }
            }))
            .children(self.items.iter().enumerate().map(|(i, item)| {
                div()
                    .id(item.id.clone())
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_2()
                    .border_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .map(|mut this| {
                        this.style().refine(&self.item_style);
                        this
                    })
                    .when(self.selected == i, |this| this.bg(config.theme.muted))
                    .on_mouse_move(cx.listener(move |this, _, _, cx| {
                        if this.selected != i {
                            this.selected = i;
                            cx.notify();
                        }
                    }))
                    .on_click({
                        let action = item.action.clone();
                        cx.listener(move |_, _, window, cx| {
                            action(window, cx);
                            cx.emit(DismissEvent);
                        })
                    })
                    .when_some(item.icon, |this, icon| {
                        this.child(icon.build(config.theme.foreground))
                    })
                    .child(item.label.clone())
            }))
    }
}

struct DropdownItem {
    id: ElementId,
    label: SharedString,
    icon: Option<Icon>,
    action: Rc<ItemAction>,
    separate: bool,
}
type ItemAction = dyn Fn(&mut Window, &mut App);

pub struct Dropdown<M: ManagedView> {
    id: ElementId,

    anchor: Anchor,
    trigger: Option<AnyElement>,
    content_builder: Option<Rc<ContentBuilder<M>>>,
}
type ContentBuilder<M> = dyn Fn(&mut Context<M>) -> M;

impl<M> Dropdown<M>
where
    M: ManagedView,
{
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            anchor: Anchor::TopLeft,
            trigger: None,
            content_builder: None,
        }
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn trigger<T>(mut self, trigger: T) -> Self
    where
        T: IntoElement + 'static,
    {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    /// Set the content of the popover.
    pub fn content<C>(mut self, content: C) -> Self
    where
        C: Fn(&mut Context<M>) -> M + 'static,
    {
        self.content_builder = Some(Rc::new(content));
        self
    }

    fn with_element_state<R>(
        &mut self,
        id: &GlobalElementId,
        window: &mut Window,
        cx: &mut App,
        f: impl FnOnce(&mut Self, &mut DropdownLayoutState<M>, &mut Window, &mut App) -> R,
    ) -> R {
        window.with_element_state::<DropdownLayoutState<M>, _>(id, |element_state, window| {
            let mut element_state = element_state.unwrap_or_default();
            let result = f(self, &mut element_state, window, cx);
            (result, element_state)
        })
    }
}

impl<M> IntoElement for Dropdown<M>
where
    M: ManagedView,
{
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct DropdownLayoutState<M> {
    content_view: Rc<RefCell<Option<Entity<M>>>>,
    previously_focused: Rc<RefCell<Option<FocusHandle>>>,
    popover: Option<(LayoutId, AnyElement)>,
    trigger: Option<(LayoutId, AnyElement)>,
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl<M> Default for DropdownLayoutState<M> {
    fn default() -> Self {
        Self {
            content_view: Rc::new(RefCell::new(None)),
            previously_focused: Rc::new(RefCell::new(None)),
            popover: None,
            trigger: None,
            trigger_bounds: None,
        }
    }
}

pub struct DropdownPrepaintState {
    hitbox: Hitbox,
    /// Trigger bounds for limit a rect to handle mouse click.
    trigger_bounds: Option<Bounds<Pixels>>,
}

impl<M: ManagedView> Element for Dropdown<M> {
    type RequestLayoutState = DropdownLayoutState<M>;
    type PrepaintState = DropdownPrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        Some(std::panic::Location::caller())
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.with_element_state(
            id.unwrap(),
            window,
            cx,
            |view: &mut Dropdown<M>, element_state, window, cx| {
                let popover =
                    if let Some(content_view) = element_state.content_view.borrow_mut().as_mut() {
                        let popover = anchored()
                            .snap_to_window_with_margin(px(8.))
                            .anchor(match view.anchor {
                                Anchor::TopLeft => Anchor::BottomLeft,
                                Anchor::TopRight => Anchor::BottomRight,
                                Anchor::BottomLeft => Anchor::TopLeft,
                                Anchor::BottomRight => Anchor::TopRight,
                                Anchor::TopCenter => Anchor::BottomCenter,
                                Anchor::BottomCenter => Anchor::TopCenter,
                                Anchor::LeftCenter => Anchor::RightCenter,
                                Anchor::RightCenter => Anchor::LeftCenter,
                            })
                            .when_some(element_state.trigger_bounds, |anchored, trigger_bounds| {
                                anchored.position(trigger_bounds.corner(view.anchor))
                            })
                            .child(
                                div()
                                    .size_full()
                                    .occlude()
                                    .map(|this| match view.anchor {
                                        Anchor::TopLeft | Anchor::TopCenter | Anchor::TopRight => {
                                            this.top_0p5()
                                        }
                                        Anchor::BottomLeft
                                        | Anchor::BottomCenter
                                        | Anchor::BottomRight => this.bottom_0p5(),
                                        Anchor::LeftCenter => this.left_0p5(),
                                        Anchor::RightCenter => this.right_0p5(),
                                    })
                                    .child(content_view.clone())
                                    .on_mouse_down_out({
                                        let content_view = element_state.content_view.clone();
                                        let previously_focused =
                                            element_state.previously_focused.clone();
                                        move |_, window, cx| {
                                            cx.stop_propagation();
                                            window.prevent_default();
                                            if let Some(previous_focus_handle) =
                                                previously_focused.borrow_mut().take()
                                            {
                                                window.focus(&previous_focus_handle, cx);
                                            }
                                            *content_view.borrow_mut() = None;
                                            window.refresh();
                                        }
                                    }),
                            );
                        let mut element = deferred(popover).with_priority(1).into_any();

                        Some((element.request_layout(window, cx), element))
                    } else {
                        None
                    };

                let trigger = {
                    let mut trigger_element = match view.trigger.take() {
                        Some(element) => element,
                        None => div().into_any_element(),
                    };
                    Some((trigger_element.request_layout(window, cx), trigger_element))
                };

                let layout_id: LayoutId = window.request_layout(
                    Style::default(),
                    trigger
                        .as_ref()
                        .into_iter()
                        .chain(popover.as_ref())
                        .map(|(id, _)| *id),
                    cx,
                );

                (
                    layout_id,
                    DropdownLayoutState {
                        popover,
                        trigger,
                        ..Default::default()
                    },
                )
            },
        )
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let mut trigger_bounds = None;
        if let Some((id, element)) = &mut request_layout.trigger {
            element.prepaint(window, cx);
            trigger_bounds = Some(window.layout_bounds(*id));
        }
        if let Some((id, element)) = &mut request_layout.popover {
            element.prepaint(window, cx);
            window.layout_bounds(*id);
        }

        let hitbox = window.insert_hitbox(
            trigger_bounds.unwrap_or_default(),
            gpui::HitboxBehavior::Normal,
        );

        DropdownPrepaintState {
            trigger_bounds,
            hitbox,
        }
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.with_element_state(
            id.unwrap(),
            window,
            cx,
            |this, element_state, window, cx| {
                element_state.trigger_bounds = prepaint.trigger_bounds;

                if let Some((_, mut element)) = request_layout.trigger.take() {
                    element.paint(window, cx);
                }

                if let Some((_, mut element)) = request_layout.popover.take() {
                    element.paint(window, cx);
                    return; // Popover is painted, no need to handle trigger click.
                }

                let Some(content_builder) = this.content_builder.take() else {
                    return; // No popover content.
                };

                let content_view = element_state.content_view.clone();
                let previously_focused = element_state.previously_focused.clone();
                let hitbox_id = prepaint.hitbox.id;

                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                    if phase == DispatchPhase::Bubble
                        && event.button == MouseButton::Left
                        && hitbox_id.is_hovered(window)
                    {
                        cx.stop_propagation();
                        window.prevent_default();

                        *previously_focused.borrow_mut() = window.focused(cx);
                        let new_content_view = cx.new(|cx| content_builder(cx));

                        window
                            .subscribe(&new_content_view, cx, {
                                let content_view = content_view.clone();
                                let previously_focused = previously_focused.clone();
                                move |popover, _: &DismissEvent, window, cx| {
                                    if popover.focus_handle(cx).contains_focused(window, cx)
                                        && let Some(previous_focus_handle) =
                                            previously_focused.borrow_mut().take()
                                    {
                                        window.focus(&previous_focus_handle, cx);
                                    }
                                    *content_view.borrow_mut() = None;
                                    window.refresh();
                                }
                            })
                            .detach();

                        window.focus(&new_content_view.focus_handle(cx), cx);
                        *content_view.borrow_mut() = Some(new_content_view);
                        window.refresh();
                    }
                });
            },
        );
    }
}
