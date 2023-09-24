//! Displays a [`TabBar`](TabBar) to select the content to be displayed.
//!
//! You have to manage the logic to show the contend by yourself or you may want
//! to use the [`Tabs`](super::tabs::Tabs) widget instead.
//!
//! *This API requires the following crate features to be activated: `tab_bar`*
use iced_widget::{
    core::{
        self,
        alignment::{self, Horizontal, Vertical},
        event, layout,
        mouse::{self, Cursor},
        renderer, touch,
        widget::Tree,
        Alignment, Clipboard, Color, Element, Event, Layout, Length, Rectangle, Shell, Widget,
    },
    runtime::Font,
    text::{self, LineHeight},
    Column, Row, Text,
};

pub mod tab_label;
pub use crate::style::tab_bar::{Appearance, StyleSheet};
pub use tab_label::TabLabel;

use crate::graphics::icons;

use std::marker::PhantomData;
/// The default icon size.
const DEFAULT_ICON_SIZE: f32 = 32.0;
/// The default text size.
const DEFAULT_TEXT_SIZE: f32 = 16.0;
/// The default size of the close icon.
const DEFAULT_CLOSE_SIZE: f32 = 16.0;
/// The default padding between the tabs.
const DEFAULT_PADDING: f32 = 5.0;
/// The default spacing around the tabs.
const DEFAULT_SPACING: f32 = 0.0;

/// A tab bar to show tabs.
///
/// # Example
/// ```ignore
/// # use iced_aw::{TabLabel, TabBar};
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TabSelected(TabId),
/// }
///
/// #[derive(PartialEq, Hash)]
/// enum TabId {
///    One,
///    Two,
///    Three,
/// }
///
/// let tab_bar = TabBar::new(
///     Message::TabSelected,
/// )
/// .push(TabId::One, TabLabel::Text(String::from("One")))
/// .push(TabId::Two, TabLabel::Text(String::from("Two")))
/// .push(TabId::Three, TabLabel::Text(String::from("Three")))
/// .set_active_tab(&TabId::One);
/// ```
#[allow(missing_debug_implementations)]
pub struct TabBar<Message, TabId, Renderer = crate::Renderer>
where
    Renderer: core::Renderer + core::text::Renderer,
    Renderer::Theme: StyleSheet,
    TabId: Eq + Clone,
{
    /// The index of the currently active tab.
    active_tab: usize,
    /// The vector containing the labels of the tabs.
    tab_labels: Vec<TabLabel>,
    /// The vector containing the indices of the tabs.
    tab_indices: Vec<TabId>,
    /// The function that produces the message when a tab is selected.
    on_select: Box<dyn Fn(TabId) -> Message>,
    /// The function that produces the message when the close icon was pressed.
    on_close: Option<Box<dyn Fn(TabId) -> Message>>,
    /// The width of the [`TabBar`](TabBar).
    width: Length,
    /// The width of the tabs of the [`TabBar`](TabBar).
    tab_width: Length,
    /// The width of the [`TabBar`](TabBar).
    height: Length,
    /// The maximum height of the [`TabBar`](TabBar).
    max_height: f32,
    /// The icon size.
    icon_size: f32,
    /// The text size.
    text_size: f32,
    /// The size of the close icon.
    close_size: f32,
    /// The padding of the tabs of the [`TabBar`](TabBar).
    padding: f32,
    /// The spacing of the tabs of the [`TabBar`](TabBar).
    spacing: f32,
    /// The optional icon font of the [`TabBar`](TabBar).
    icon_font: Option<Font>,
    /// The optional text font of the [`TabBar`](TabBar).
    text_font: Option<Font>,
    /// The style of the [`TabBar`](TabBar).
    style: <Renderer::Theme as StyleSheet>::Style,
    #[allow(clippy::missing_docs_in_private_items)]
    _renderer: PhantomData<Renderer>,
}

impl<Message, TabId, Renderer> TabBar<Message, TabId, Renderer>
where
    Renderer: core::Renderer + core::text::Renderer<Font = core::Font>,
    Renderer::Theme: StyleSheet,
    TabId: Eq + Clone,
{
    /// Creates a new [`TabBar`](TabBar) with the index of the selected tab and a
    /// specified message which will be send when a tab is selected by the user.
    ///
    /// It expects:
    ///     * the index of the currently active tab.
    ///     * the function that will be called if a tab is selected by the user.
    ///         It takes the index of the selected tab.
    pub fn new<F>(on_select: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        Self::with_tab_labels(Vec::new(), on_select)
    }

    /// Similar to `new` but with a given Vector of the
    /// [`TabLabel`](crate::tab_bar::TabLabel)s.Alignment
    ///
    /// It expects:
    ///     * the index of the currently active tab.
    ///     * a vector containing the [`TabLabel`](TabLabel)s of the [`TabBar`](TabBar).
    ///     * the function that will be called if a tab is selected by the user.
    ///         It takes the index of the selected tab.
    pub fn with_tab_labels<F>(tab_labels: Vec<(TabId, TabLabel)>, on_select: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        Self {
            active_tab: 0,
            tab_indices: tab_labels.iter().map(|(id, _)| id.clone()).collect(),
            tab_labels: tab_labels.into_iter().map(|(_, label)| label).collect(),
            on_select: Box::new(on_select),
            on_close: None,
            width: Length::Fill,
            tab_width: Length::Fill,
            height: Length::Shrink,
            max_height: 4_294_967_295.0,
            icon_size: DEFAULT_ICON_SIZE,
            text_size: DEFAULT_TEXT_SIZE,
            close_size: DEFAULT_CLOSE_SIZE,
            padding: DEFAULT_PADDING,
            spacing: DEFAULT_SPACING,
            icon_font: None,
            text_font: None,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            _renderer: PhantomData,
        }
    }

    /// Gets the index of the currently active tab on the [`TabBar`](TabBar).
    #[must_use]
    pub fn get_active_tab_idx(&self) -> usize {
        self.active_tab
    }

    /// Gets the id of the currently active tab on the [`TabBar`](TabBar).
    #[must_use]
    pub fn get_active_tab_id(&self) -> Option<&TabId> {
        self.tab_indices.get(self.active_tab)
    }

    /// Gets the amount of tabs on the [`TabBar`](TabBar).
    #[must_use]
    pub fn size(&self) -> usize {
        self.tab_indices.len()
    }

    /// Sets the message that will be produced when the close icon of a tab
    /// on the [`TabBar`](TabBar) is pressed.
    ///
    /// Setting this enables the drawing of a close icon on the tabs.
    #[must_use]
    pub fn on_close<F>(mut self, on_close: F) -> Self
    where
        F: 'static + Fn(TabId) -> Message,
    {
        self.on_close = Some(Box::new(on_close));
        self
    }

    /// Sets the width of the [`TabBar`](TabBar).
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Gets the width of the [`TabBar`](TabBar).
    #[must_use]
    pub fn get_width(&self) -> Length {
        self.width
    }

    /// Sets the width of a tab on the [`TabBar`](TabBar).
    #[must_use]
    pub fn tab_width(mut self, width: Length) -> Self {
        self.tab_width = width;
        self
    }

    /// Sets the height of the [`TabBar`](TabBar).
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Gets the width of the [`TabBar`](TabBar).
    #[must_use]
    pub fn get_height(&self) -> Length {
        self.height
    }

    /// Sets the maximum height of the [`TabBar`](TabBar).
    #[must_use]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the icon size of the [`TabLabel`](crate::tab_bar::TabLabel)s of
    /// the [`TabBar`](TabBar).
    #[must_use]
    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size;
        self
    }

    /// Sets the text size of the [`TabLabel`](crate::tab_bar::TabLabel)s of the
    /// [`TabBar`](TabBar).
    #[must_use]
    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size;
        self
    }

    /// Sets the size of the close icon of the
    /// [`TabLabel`](crate::tab_bar::TabLabel)s of the [`TabBar`](TabBar).
    #[must_use]
    pub fn close_size(mut self, close_size: f32) -> Self {
        self.close_size = close_size;
        self
    }

    /// Sets the padding of the tabs of the [`TabBar`](TabBar).
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the spacing between the tabs of the [`TabBar`](TabBar).
    #[must_use]
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the font of the icons of the
    /// [`TabLabel`](crate::tab_bar::TabLabel)s of the [`TabBar`](TabBar).
    #[must_use]
    pub fn icon_font(mut self, icon_font: Font) -> Self {
        self.icon_font = Some(icon_font);
        self
    }

    /// Sets the font of the text of the
    /// [`TabLabel`](crate::tab_bar::TabLabel)s of the [`TabBar`](TabBar).
    #[must_use]
    pub fn text_font(mut self, text_font: Font) -> Self {
        self.text_font = Some(text_font);
        self
    }

    /// Sets the style of the [`TabBar`](TabBar).
    #[must_use]
    pub fn style(mut self, style: <Renderer::Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }

    /// Pushes a [`TabLabel`](crate::tab_bar::TabLabel) to the [`TabBar`](TabBar).
    #[must_use]
    pub fn push(mut self, id: TabId, tab_label: TabLabel) -> Self {
        self.tab_labels.push(tab_label);
        self.tab_indices.push(id);
        self
    }

    /// Sets up the active tab on the [`TabBar`](TabBar).
    #[must_use]
    pub fn set_active_tab(mut self, active_tab: &TabId) -> Self {
        self.active_tab = self
            .tab_indices
            .iter()
            .position(|id| id == active_tab)
            .map_or(0, |a| a);
        self
    }
}

impl<Message, TabId, Renderer> Widget<Message, Renderer> for TabBar<Message, TabId, Renderer>
where
    Renderer: core::Renderer + core::text::Renderer<Font = core::Font>,
    Renderer::Theme: StyleSheet + text::StyleSheet,
    TabId: Eq + Clone,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.tab_labels
            .iter()
            .fold(Row::<Message, Renderer>::new(), |row, tab_label| {
                let label = match tab_label {
                    TabLabel::Icon(icon) => Column::new().align_items(Alignment::Center).push(
                        Row::new()
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .push(
                                Text::new(icon.to_string())
                                    .size(self.icon_size)
                                    .font(self.icon_font.unwrap_or_default())
                                    .horizontal_alignment(alignment::Horizontal::Center)
                                    .vertical_alignment(alignment::Vertical::Center),
                            ),
                    ),
                    TabLabel::Text(text) => Column::new().align_items(Alignment::Center).push(
                        Text::new(text)
                            .size(self.text_size)
                            .width(self.tab_width)
                            .font(self.text_font.unwrap_or_default())
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .vertical_alignment(alignment::Vertical::Center),
                    ),
                    TabLabel::IconText(icon, text) => Column::new()
                        .align_items(Alignment::Center)
                        .push(
                            Row::new()
                                .width(Length::Shrink)
                                .height(Length::Shrink)
                                .push(
                                    Text::new(icon.to_string())
                                        .size(self.icon_size)
                                        .font(self.icon_font.unwrap_or_default())
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                        .vertical_alignment(alignment::Vertical::Center),
                                ),
                        )
                        .push(
                            Text::new(text)
                                .size(self.text_size)
                                .width(self.tab_width)
                                .font(self.text_font.unwrap_or_default()),
                        ),
                }
                .width(self.tab_width)
                .height(self.height);

                let mut label_row = Row::new()
                    .align_items(Alignment::Center)
                    .padding(self.padding)
                    .width(self.tab_width)
                    .push(label);

                if self.on_close.is_some() {
                    label_row = label_row.push(
                        Row::new()
                            .width(Length::Fixed(self.close_size + 1.0))
                            .height(Length::Fixed(self.close_size + 1.0))
                            .align_items(Alignment::Center),
                    );
                }

                row.push(label_row)
            })
            .width(self.width)
            .height(self.height)
            .spacing(self.spacing)
            .layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if layout
                    .bounds()
                    .contains(cursor.position().unwrap_or_default())
                {
                    let tabs_map: Vec<bool> = layout
                        .children()
                        .map(|layout| {
                            layout
                                .bounds()
                                .contains(cursor.position().unwrap_or_default())
                        })
                        .collect();

                    if let Some(new_selected) = tabs_map.iter().position(|b| *b) {
                        shell.publish(
                            self.on_close
                                .as_ref()
                                .filter(|_on_close| {
                                    let tab_layout = layout.children().nth(new_selected).expect("Native: Layout should have a tab layout at the selected index");
                                    let cross_layout = tab_layout.children().nth(1).expect("Native: Layout should have a close layout");

                                    cross_layout.bounds().contains(cursor.position().unwrap_or_default())
                                })
                                .map_or_else(
                                    || (self.on_select)(self.tab_indices[new_selected].clone()),
                                    |on_close| (on_close)(self.tab_indices[new_selected].clone()),
                                ),
                        );
                        return event::Status::Captured;
                    }
                }
                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let children = layout.children();
        let mut mouse_interaction = mouse::Interaction::default();

        for layout in children {
            let is_mouse_over = layout
                .bounds()
                .contains(cursor.position().unwrap_or_default());
            let new_mouse_interaction = if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            };

            if new_mouse_interaction > mouse_interaction {
                mouse_interaction = new_mouse_interaction;
            }
        }

        mouse_interaction
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let children = layout.children();
        let is_mouse_over = bounds.contains(cursor.position().unwrap_or_default());
        let style_sheet = if is_mouse_over {
            theme.hovered(self.style, false)
        } else {
            theme.active(self.style, false)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: (0.0).into(),
                border_width: style_sheet.border_width,
                border_color: style_sheet.border_color.unwrap_or(Color::TRANSPARENT),
            },
            style_sheet
                .background
                .unwrap_or_else(|| Color::TRANSPARENT.into()),
        );

        for ((i, tab), layout) in self.tab_labels.iter().enumerate().zip(children) {
            draw_tab(
                renderer,
                tab,
                layout,
                theme,
                self.style,
                i == self.get_active_tab_idx(),
                cursor,
                (
                    self.icon_font.unwrap_or(icons::AW_ICON_FONT),
                    self.icon_size,
                ),
                (self.text_font.unwrap_or_default(), self.text_size),
                self.close_size,
            );
        }
    }
}

/// Draws a tab.
#[allow(
    clippy::borrowed_box,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
fn draw_tab<Renderer>(
    renderer: &mut Renderer,
    tab: &TabLabel,
    layout: Layout<'_>,
    theme: &Renderer::Theme,
    style: <Renderer::Theme as StyleSheet>::Style,
    is_selected: bool,
    cursor: Cursor,
    icon_data: (Font, f32),
    text_data: (Font, f32),
    close_size: f32,
) where
    Renderer: core::Renderer + core::text::Renderer<Font = core::Font>,
    Renderer::Theme: StyleSheet + text::StyleSheet,
{
    let is_mouse_over = layout
        .bounds()
        .contains(cursor.position().unwrap_or_default());
    let style = if is_mouse_over {
        theme.hovered(style, is_selected)
    } else {
        theme.active(style, is_selected)
    };

    let bounds = layout.bounds();
    let mut children = layout.children();
    let label_layout = children
        .next()
        .expect("Graphics: Layout should have a label layout");
    let mut label_layout_children = label_layout.children();

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border_radius: (0.0).into(),
            border_width: style.tab_label_border_width,
            border_color: style.tab_label_border_color,
        },
        style.tab_label_background,
    );

    match tab {
        TabLabel::Icon(icon) => {
            let icon_bounds = label_layout_children
                .next()
                .expect("Graphics: Layout should have an icon layout for an Icon")
                .bounds();

            renderer.fill_text(core::text::Text {
                content: &icon.to_string(),
                bounds: Rectangle {
                    x: icon_bounds.center_x(),
                    y: icon_bounds.center_y(),
                    ..icon_bounds
                },
                size: icon_data.1,
                color: style.icon_color,
                font: icon_data.0,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping: iced_widget::text::Shaping::Advanced,
            });
        }
        TabLabel::Text(text) => {
            let text_bounds = label_layout_children
                .next()
                .expect("Graphics: Layout should have a text layout for a Text")
                .bounds();

            renderer.fill_text(core::text::Text {
                content: &text[..],
                bounds: Rectangle {
                    x: text_bounds.center_x(),
                    y: text_bounds.center_y(),
                    ..text_bounds
                },
                size: text_data.1,
                color: style.text_color,
                font: text_data.0,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping: iced_widget::text::Shaping::Advanced,
            });
        }
        TabLabel::IconText(icon, text) => {
            let icon_bounds = label_layout_children
                .next()
                .expect("Graphics: Layout should have an icons layout for an IconText")
                .bounds();
            let text_bounds = label_layout_children
                .next()
                .expect("Graphics: Layout should have a text layout for an IconText")
                .bounds();

            renderer.fill_text(core::text::Text {
                content: &icon.to_string(),
                bounds: Rectangle {
                    x: icon_bounds.center_x(),
                    y: icon_bounds.center_y(),
                    ..icon_bounds
                },
                size: icon_data.1,
                color: style.icon_color,
                font: icon_data.0,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping: iced_widget::text::Shaping::Advanced,
            });

            renderer.fill_text(core::text::Text {
                content: &text[..],
                bounds: Rectangle {
                    x: text_bounds.center_x(),
                    y: text_bounds.center_y(),
                    ..text_bounds
                },
                size: text_data.1,
                color: style.text_color,
                font: text_data.0,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                line_height: LineHeight::Relative(1.3),
                shaping: iced_widget::text::Shaping::Advanced,
            });
        }
    };

    if let Some(cross_layout) = children.next() {
        let cross_bounds = cross_layout.bounds();
        let is_mouse_over_cross = cursor.is_over(cross_bounds);

        renderer.fill_text(core::text::Text {
            content: &icons::icon_to_char(icons::Icon::X).to_string(),
            bounds: Rectangle {
                x: cross_bounds.center_x(),
                y: cross_bounds.center_y(),
                ..cross_bounds
            },
            size: close_size + if is_mouse_over_cross { 1.0 } else { 0.0 },
            color: style.icon_color,
            font: icons::AW_ICON_FONT,
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Center,
            line_height: LineHeight::Relative(1.3),
            shaping: iced_widget::text::Shaping::Basic,
        });
    };
}

impl<'a, Message, TabId, Renderer> From<TabBar<Message, TabId, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + core::Renderer + core::text::Renderer<Font = core::Font>,
    Renderer::Theme: StyleSheet + text::StyleSheet,
    Message: 'a,
    TabId: 'a + Eq + Clone,
{
    fn from(tab_bar: TabBar<Message, TabId, Renderer>) -> Self {
        Element::new(tab_bar)
    }
}
