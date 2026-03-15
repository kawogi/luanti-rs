//! Contains parsers and serializers for Luanti's Formspec-strings.

#![expect(unused, reason = "// TODO")]

pub(crate) mod arguments;
pub(crate) mod error;

use std::{collections::VecDeque, fmt::Display, num::ParseIntError};

use crate::{arguments::RawArgs, error::FormspecError};

const ELEMENT_START: char = '[';
const ELEMENT_END: char = ']';
const ITEM_SEPARATOR: char = ';';
const SUB_ITEM_SEPARATOR: char = ',';

struct Formspec {
    elements: Vec<FormspecElement>,
}

impl Formspec {
    #[expect(clippy::too_many_lines, reason = "too many variants")]
    fn from_str(mut str: &str) -> Result<Self, FormspecError> {
        while let Some((name, args)) = FormspecElement::read_raw(&mut str)? {
            #[expect(clippy::match_same_arms, reason = "// TODO")]
            match name {
                elements::FormspecVersion::NAME => {
                    //
                }
                elements::Size::NAME => {
                    //
                }
                elements::Position::NAME => {
                    //
                }
                elements::Anchor::NAME => {
                    //
                }
                elements::Padding::NAME => {
                    //
                }
                elements::NoPrepend::NAME => {
                    //
                }
                elements::RealCoordinates::NAME => {
                    //
                }
                elements::AllowClose::NAME => {
                    //
                }
                elements::Container::NAME => {
                    //
                }
                elements::ContainerEnd::NAME => {
                    //
                }
                elements::ScrollContainer::NAME => {
                    //
                }
                elements::ScrollContainerEnd::NAME => {
                    //
                }
                elements::List::NAME => {
                    //
                }
                elements::ListRing::NAME => {
                    //
                }
                elements::ListColors::NAME => {
                    //
                }
                elements::Tooltip::NAME => {
                    //
                }
                elements::Image::NAME => {
                    //
                }
                elements::AnimatedImage::NAME => {
                    //
                }
                elements::Model::NAME => {
                    //
                }
                elements::ItemImage::NAME => {
                    //
                }
                elements::BackgroundColor::NAME => {
                    //
                }
                elements::Background::NAME => {
                    //
                }
                elements::Background9::NAME => {
                    //
                }
                elements::PasswordField::NAME => {
                    //
                }
                elements::Field::NAME => {
                    //
                }
                elements::FieldEnterAfterEdit::NAME => {
                    //
                }
                elements::FieldCloseOnEnter::NAME => {
                    //
                }
                elements::Textarea::NAME => {
                    //
                }
                elements::Label::NAME => {
                    //
                }
                elements::Hypertext::NAME => {
                    //
                }
                elements::VerticalLabel::NAME => {
                    //
                }
                elements::Button::NAME => {
                    //
                }
                elements::ButtonUrl::NAME => {
                    //
                }
                elements::ImageButton::NAME => {
                    //
                }
                elements::ItemImageButton::NAME => {
                    //
                }
                elements::ButtonExit::NAME => {
                    //
                }
                elements::ButtonUrlExit::NAME => {
                    //
                }
                elements::ImageButtonExit::NAME => {
                    //
                }
                elements::TextList::NAME => {
                    //
                }
                elements::TabHeader::NAME => {
                    //
                }
                elements::Box::NAME => {
                    //
                }
                elements::Dropdown::NAME => {
                    //
                }
                elements::Checkbox::NAME => {
                    //
                }
                elements::Scrollbar::NAME => {
                    //
                }
                elements::ScrollbarOptions::NAME => {
                    //
                }
                elements::Table::NAME => {
                    //
                }
                elements::TableOptions::NAME => {
                    //
                }
                elements::TableColumns::NAME => {
                    //
                }
                elements::Style::NAME => {
                    //
                }
                elements::StyleType::NAME => {
                    //
                }
                elements::SetFocus::NAME => {
                    //
                }
                unknown => return Err(FormspecError::UnknownElement(unknown.to_owned())),
            }
        }

        todo!();
    }
}

struct ElementName(String);

struct Position {
    pub x: f32,
    pub y: f32,
}

struct Size {
    pub x: f32,
    pub y: f32,
}

struct SizeSlots {
    pub x: u32,
    pub y: u32,
}

enum Color {
    Named(String),
    Rgb(u8, u8, u8),
}

// struct Dimensions {
//     pub pos: Position,
//     pub size: Size,
// }

impl ElementName {
    fn new(name: String) -> Result<Self, FormspecError> {
        if name.is_empty() {
            return Err(FormspecError::NameIsEmpty);
        }
        if name == "quit" {
            return Err(FormspecError::NameIsQuit);
        }
        if name.starts_with("key_") {
            return Err(FormspecError::NameIsKey);
        }
        if !name
            .chars()
            .all(|char| char.is_alphanumeric() || char == '_')
        {
            return Err(FormspecError::NameInvalidChar(name));
        }

        Ok(Self(name))
    }
}

struct FormspecString(pub String);

impl Display for FormspecString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // str_replace(str, "\\", "\\\\");
        // str_replace(str, "]", "\\]");
        // str_replace(str, "[", "\\[");
        // str_replace(str, ";", "\\;");
        // str_replace(str, ",", "\\,");
        // str_replace(str, "$", "\\$");
        todo!()
    }
}

/// [Formspec documentation](https://api.luanti.org/formspec/)
enum FormspecElement {
    /// `formspec_version[<version>]`
    FormspecVersion,
    /// `size[<W>,<H>,<fixed_size>]`
    Size,
    /// `position[<X>,<Y>]`
    Position,
    /// `anchor[<X>,<Y>]`
    Anchor,
    /// `padding[<X>,<Y>]`
    Padding,
    /// `no_prepend[]`
    NoPrepend,
    /// `real_coordinates[<bool>]`
    RealCoordinates,
    /// `allow_close[<bool>]`
    AllowClose,
    /// `container[<X>,<Y>]`
    Container,
    /// `container_end[]`
    ContainerEnd,
    /// `scroll_container[<X>,<Y>;<W>,<H>;<scrollbar name>;<orientation>;<scroll factor>;<content padding>]`
    ScrollContainer,
    /// `scroll_container_end[]`
    ScrollContainerEnd,
    /// `list[<inventory location>;<list name>;<X>,<Y>;<W>,<H>;<starting item index>]`
    List,
    /// `listring[<inventory location>;<list name>]`
    /// `listring[]`
    ListRing,
    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>]`
    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>;<slot_border>]`
    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>;<slot_border>;<tooltip_bgcolor>;<tooltip_fontcolor>]`
    ListColors,
    /// `tooltip[<gui_element_name>;<tooltip_text>;<bgcolor>;<fontcolor>]`
    /// `tooltip[<X>,<Y>;<W>,<H>;<tooltip_text>;<bgcolor>;<fontcolor>]`
    Tooltip,
    /// `image[<X>,<Y>;<W>,<H>;<texture name>;<middle>]`
    Image,
    /// `animated_image[<X>,<Y>;<W>,<H>;<name>;<texture name>;<frame count>;<frame duration>;<frame start>;<middle>]`
    AnimatedImage,
    /// `model[<X>,<Y>;<W>,<H>;<name>;<mesh>;<textures>;<rotation>;<continuous>;<mouse control>;<frame loop range>;<animation speed>]`
    Model,
    /// `item_image[<X>,<Y>;<W>,<H>;<item name>]`
    ItemImage,
    /// `bgcolor[<bgcolor>;<fullscreen>;<fbgcolor>]`
    BackgroundColor,
    /// `background[<X>,<Y>;<W>,<H>;<texture name>]`
    /// `background[<X>,<Y>;<W>,<H>;<texture name>;<auto_clip>]`
    Background,
    /// `background9[<X>,<Y>;<W>,<H>;<texture name>;<auto_clip>;<middle>]`
    Background9,
    /// `pwdfield[<X>,<Y>;<W>,<H>;<name>;<label>]`
    PasswordField,
    /// `field[<X>,<Y>;<W>,<H>;<name>;<label>;<default>]`
    /// `field[<name>;<label>;<default>]`
    Field,
    /// `field_enter_after_edit[<name>;<enter_after_edit>]`
    FieldEnterAfterEdit,
    /// `field_close_on_enter[<name>;<close_on_enter>]`
    FieldCloseOnEnter,
    /// `textarea[<X>,<Y>;<W>,<H>;<name>;<label>;<default>]`
    Textarea,
    /// `label[<X>,<Y>;<label>]`
    /// `label[<X>,<Y>;<W>,<H>;<label>]`
    Label,
    /// `hypertext[<X>,<Y>;<W>,<H>;<name>;<text>]`
    Hypertext,
    /// `vertlabel[<X>,<Y>;<label>]`
    VerticalLabel,
    /// `button[<X>,<Y>;<W>,<H>;<name>;<label>]`
    Button,
    /// `button_url[<X>,<Y>;<W>,<H>;<name>;<label>;<url>]`
    ButtonUrl,
    /// `image_button[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>]`
    /// `image_button[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>;<noclip>;<drawborder>;<pressed texture name>]`
    ImageButton,
    /// `item_image_button[<X>,<Y>;<W>,<H>;<item name>;<name>;<label>]`
    ItemImageButton,
    /// `button_exit[<X>,<Y>;<W>,<H>;<name>;<label>]`
    ButtonExit,
    /// `button_url_exit[<X>,<Y>;<W>,<H>;<name>;<label>;<url>]`
    ButtonUrlExit,
    /// `image_button_exit[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>]`
    ImageButtonExit,
    /// `textlist[<X>,<Y>;<W>,<H>;<name>;<listelem 1>,<listelem 2>,...,<listelem n>]`
    /// `textlist[<X>,<Y>;<W>,<H>;<name>;<listelem 1>,<listelem 2>,...,<listelem n>;<selected idx>;<transparent>]`
    TextList,
    /// `tabheader[<X>,<Y>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]`
    /// `tabheader[<X>,<Y>;<H>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]`
    /// `tabheader[<X>,<Y>;<W>,<H>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]¶`
    TabHeader,
    /// `box[<X>,<Y>;<W>,<H>;<color>]`
    Box,
    /// `dropdown[<X>,<Y>;<W>;<name>;<item 1>,<item 2>, ...,<item n>;<selected idx>;<index event>]`
    /// `dropdown[<X>,<Y>;<W>,<H>;<name>;<item 1>,<item 2>, ...,<item n>;<selected idx>;<index event>]`
    Dropdown,
    /// `checkbox[<X>,<Y>;<name>;<label>;<selected>]`
    Checkbox,
    /// `scrollbar[<X>,<Y>;<W>,<H>;<orientation>;<name>;<value>]`
    Scrollbar,
    /// `scrollbaroptions[opt1;opt2;...]`
    ScrollbarOptions,
    /// `table[<X>,<Y>;<W>,<H>;<name>;<cell 1>,<cell 2>,...,<cell n>;<selected idx>]`
    Table,
    /// `tableoptions[<opt 1>;<opt 2>;...]`
    TableOptions,
    /// `tablecolumns[<type 1>,<opt 1a>,<opt 1b>,...;<type 2>,<opt 2a>,<opt 2b>;...]`
    TableColumns,
    /// `style[<selector 1>,<selector 2>,...;<prop1>;<prop2>;...]`
    Style,
    /// `style_type[<selector 1>,<selector 2>,...;<prop1>;<prop2>;...]`
    StyleType,
    /// `set_focus[<name>;<force>]`
    SetFocus,
}

impl FormspecElement {
    #[expect(
        clippy::string_slice,
        reason = "all indices are guaranteed to fall onto a character boundary"
    )]
    fn read_raw<'str>(
        str: &mut &'str str,
    ) -> Result<Option<(&'str str, RawArgs<'str>)>, FormspecError> {
        let Some((name, remainder)) = str.split_once(ELEMENT_START) else {
            return if str.chars().all(|char| char.is_ascii_whitespace()) {
                Ok(None)
            } else {
                Err(FormspecError::PrematureEnd)
            };
        };

        let name = name.trim();

        let mut is_escaped = false;
        let mut items = VecDeque::new();
        let mut sub_items = Vec::new();
        let mut start_index = 0;
        let mut empty_sub_item_count = 0;
        let mut empty_item_count = 0;

        'next_char: for (index, char) in remainder.char_indices() {
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            enum Completion {
                None,
                SubItem,
                Item,
                Element,
            }

            // handle escaped characters and reset flag
            if std::mem::replace(&mut is_escaped, false) {
                continue;
            }

            let completion = match char {
                ELEMENT_END => Completion::Element,
                ITEM_SEPARATOR => Completion::Item,
                SUB_ITEM_SEPARATOR => Completion::SubItem,
                '\\' => {
                    is_escaped = true;
                    Completion::None
                }
                _ => Completion::None,
            };

            if completion >= Completion::SubItem {
                let sub_item = remainder[start_index..index].trim();
                if sub_item.is_empty() {
                    // to not add empty sub-items, but remember them
                    empty_sub_item_count += 1;
                } else {
                    // insert empty sub-items to retain positional index of non-empty entries
                    while empty_sub_item_count > 0 {
                        sub_items.push("");
                        empty_sub_item_count -= 1;
                    }
                    sub_items.push(sub_item);
                }
                // next sub-item starts after the current separator
                start_index = index + char.len_utf8();
            }

            if completion >= Completion::Item {
                empty_sub_item_count = 0;
                if sub_items.is_empty() {
                    // to not add empty items, but remember them
                    empty_item_count += 1;
                } else {
                    // insert empty items to retain positional index of non-empty entries
                    while empty_item_count > 0 {
                        items.push_back(vec![]);
                        empty_item_count -= 1;
                    }
                    items.push_back(std::mem::take(&mut sub_items));
                }
            }

            if completion >= Completion::Element {
                // officially consume all characters of this element
                *str = &remainder[start_index..];
                let args = RawArgs { args: items };
                return Ok(Some((name, args)));
            }
        }

        // we've reached the end of input before hitting the element's delimiter
        Err(FormspecError::PrematureEnd)
    }
}

pub mod elements {
    use std::num::NonZeroU32;

    use crate::{Color, FormspecError, SizeSlots, arguments::RawArgs, error::ArgumentError};

    /// `formspec_version[<version>]`
    pub struct FormspecVersion {
        version: u32,
    }

    impl FormspecVersion {
        pub(crate) const NAME: &str = "formspec_version";

        fn from_args(mut args: RawArgs<'_>) -> Result<Self, FormspecError> {
            const VERSION_NUMBER: &str = "version number";

            let Some(version) = args
                .read_u32()
                .map_err(|error| FormspecError::argument(Self::NAME, VERSION_NUMBER, error))?
            else {
                return Err(FormspecError::argument(
                    Self::NAME,
                    VERSION_NUMBER,
                    ArgumentError::Missing,
                ));
            };
            args.check_empty()
                .map_err(|args| FormspecError::ExcessiveArguments(Self::NAME, args))?;

            Ok(Self { version })
        }
    }

    /// `size[<W>,<H>,<fixed_size>]`
    pub struct Size {
        dimensions: super::Size,
        fixed_size: bool,
    }

    impl Size {
        pub(crate) const NAME: &str = "size";
    }

    /// `position[<X>,<Y>]`
    pub struct Position {
        position: super::Position,
    }

    impl Position {
        pub(crate) const NAME: &str = "position";
    }

    /// `anchor[<X>,<Y>]`
    pub struct Anchor {
        position: super::Position,
    }

    impl Anchor {
        pub(crate) const NAME: &str = "anchor";
    }

    /// `padding[<X>,<Y>]`
    pub struct Padding {
        padding: super::Position,
    }

    impl Padding {
        pub(crate) const NAME: &str = "padding";
    }

    /// `no_prepend[]`
    pub struct NoPrepend;

    impl NoPrepend {
        pub(crate) const NAME: &str = "no_prepend";
    }

    /// `real_coordinates[<bool>]`
    pub struct RealCoordinates {
        is_real: bool,
    }

    impl RealCoordinates {
        pub(crate) const NAME: &str = "real_coordinates";
    }

    /// `allow_close[<bool>]`
    pub struct AllowClose {
        allow_close: bool,
    }

    impl AllowClose {
        pub(crate) const NAME: &str = "allow_close";
    }

    /// `container[<X>,<Y>]`
    pub struct Container {
        position: super::Position,
    }

    impl Container {
        pub(crate) const NAME: &str = "container";
    }

    /// `container_end[]`
    pub struct ContainerEnd;

    impl ContainerEnd {
        pub(crate) const NAME: &str = "container_end";
    }

    /// `scroll_container[<X>,<Y>;<W>,<H>;<scrollbar name>;<orientation>;<scroll factor>;<content padding>]`
    pub struct ScrollContainer {
        position: super::Position,
        dimensions: super::Size,
        scrollbar_name: String,
        orientation: Orientation,
        scroll_factor: Option<f32>,
        content_padding: Option<f32>,
    }

    impl ScrollContainer {
        pub(crate) const NAME: &str = "scroll_container";
    }

    /// `scroll_container_end[]`
    pub struct ScrollContainerEnd;

    impl ScrollContainerEnd {
        pub(crate) const NAME: &str = "scroll_container_end";
    }

    /// `list[<inventory location>;<list name>;<X>,<Y>;<W>,<H>;<starting item index>]`
    pub struct List {
        inventory_location: String,
        name: String,
        position: super::Position,
        size: SizeSlots,
        starting_item_index: Option<u32>,
    }

    impl List {
        pub(crate) const NAME: &str = "list";
    }

    /// `listring[<inventory location>;<list name>]`
    /// `listring[]`
    pub struct ListRing {
        inventory_location: String,
        list_name: String,
    }

    impl ListRing {
        pub(crate) const NAME: &str = "listring";
    }

    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>]`
    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>;<slot_border>]`
    /// `listcolors[<slot_bg_normal>;<slot_bg_hover>;<slot_border>;<tooltip_bgcolor>;<tooltip_fontcolor>]`
    pub struct ListColors {
        slot_background_normal: Color,
        slot_background_hover: Color,
        slot_border: Option<Color>,
        tooltip_background: Option<Color>,
        tooltip_font: Option<Color>,
    }

    impl ListColors {
        pub(crate) const NAME: &str = "listcolors";
    }

    /// `tooltip[<gui_element_name>;<tooltip_text>;<bgcolor>;<fontcolor>]`
    /// `tooltip[<X>,<Y>;<W>,<H>;<tooltip_text>;<bgcolor>;<fontcolor>]`
    pub struct Tooltip {
        anchor: TooltipAnchor,
        text: String,
        background_color: Color,
        font_color: Color,
    }

    enum TooltipAnchor {
        Element {
            gui_element_name: String,
        },
        Position {
            position: super::Position,
            size: super::Size,
        },
    }

    impl Tooltip {
        pub(crate) const NAME: &str = "tooltip";
    }

    /// `image[<X>,<Y>;<W>,<H>;<texture name>;<middle>]`
    pub struct Image {
        position: super::Position,
        size: super::Size,
        texture_name: String,
        middle: Option<Middle>,
    }

    impl Image {
        pub(crate) const NAME: &str = "image";
    }

    /// `animated_image[<X>,<Y>;<W>,<H>;<name>;<texture name>;<frame count>;<frame duration>;<frame start>;<middle>]`
    pub struct AnimatedImage {
        position: super::Position,
        size: super::Size,
        name: String,
        texture_name: String,
        frame_count: u32,
        frame_duration_ms: u32,
        frame_start: Option<NonZeroU32>,
        middle: Option<Middle>,
    }

    impl AnimatedImage {
        pub(crate) const NAME: &str = "animated_image";
    }

    /// `model[<X>,<Y>;<W>,<H>;<name>;<mesh>;<textures>;<rotation>;<continuous>;<mouse control>;<frame loop range>;<animation speed>]`
    pub struct Model {}

    impl Model {
        pub(crate) const NAME: &str = "model";
    }

    /// `item_image[<X>,<Y>;<W>,<H>;<item name>]`
    pub struct ItemImage {}

    impl ItemImage {
        pub(crate) const NAME: &str = "item_image";
    }

    /// `bgcolor[<bgcolor>;<fullscreen>;<fbgcolor>]`
    pub struct BackgroundColor {}

    impl BackgroundColor {
        pub(crate) const NAME: &str = "bgcolor";
    }

    /// `background[<X>,<Y>;<W>,<H>;<texture name>]`
    /// `background[<X>,<Y>;<W>,<H>;<texture name>;<auto_clip>]`
    pub struct Background {}

    impl Background {
        pub(crate) const NAME: &str = "background";
    }

    /// `background9[<X>,<Y>;<W>,<H>;<texture name>;<auto_clip>;<middle>]`
    pub struct Background9 {}

    impl Background9 {
        pub(crate) const NAME: &str = "background9";
    }

    /// `pwdfield[<X>,<Y>;<W>,<H>;<name>;<label>]`
    pub struct PasswordField {}

    impl PasswordField {
        pub(crate) const NAME: &str = "pwdfield";
    }

    /// `field[<X>,<Y>;<W>,<H>;<name>;<label>;<default>]`
    /// `field[<name>;<label>;<default>]`
    pub struct Field {}

    impl Field {
        pub(crate) const NAME: &str = "field";
    }

    /// `field_enter_after_edit[<name>;<enter_after_edit>]`
    pub struct FieldEnterAfterEdit {}

    impl FieldEnterAfterEdit {
        pub(crate) const NAME: &str = "field_enter_after_edit";
    }

    /// `field_close_on_enter[<name>;<close_on_enter>]`
    pub struct FieldCloseOnEnter {}

    impl FieldCloseOnEnter {
        pub(crate) const NAME: &str = "field_close_on_enter";
    }

    /// `textarea[<X>,<Y>;<W>,<H>;<name>;<label>;<default>]`
    pub struct Textarea {}

    impl Textarea {
        pub(crate) const NAME: &str = "textarea";
    }

    /// `label[<X>,<Y>;<label>]`
    /// `label[<X>,<Y>;<W>,<H>;<label>]`
    pub struct Label {}

    impl Label {
        pub(crate) const NAME: &str = "label";
    }

    /// `hypertext[<X>,<Y>;<W>,<H>;<name>;<text>]`
    pub struct Hypertext {}

    impl Hypertext {
        pub(crate) const NAME: &str = "hypertext";
    }

    /// `vertlabel[<X>,<Y>;<label>]`
    pub struct VerticalLabel {}

    impl VerticalLabel {
        pub(crate) const NAME: &str = "vertlabel";
    }

    /// `button[<X>,<Y>;<W>,<H>;<name>;<label>]`
    pub struct Button {}

    impl Button {
        pub(crate) const NAME: &str = "button";
    }

    /// `button_url[<X>,<Y>;<W>,<H>;<name>;<label>;<url>]`
    pub struct ButtonUrl {}

    impl ButtonUrl {
        pub(crate) const NAME: &str = "button_url";
    }

    /// `image_button[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>]`
    /// `image_button[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>;<noclip>;<drawborder>;<pressed texture name>]`
    pub struct ImageButton {}

    impl ImageButton {
        pub(crate) const NAME: &str = "image_button";
    }

    /// `item_image_button[<X>,<Y>;<W>,<H>;<item name>;<name>;<label>]`
    pub struct ItemImageButton {}

    impl ItemImageButton {
        pub(crate) const NAME: &str = "item_image_button";
    }

    /// `button_exit[<X>,<Y>;<W>,<H>;<name>;<label>]`
    pub struct ButtonExit {}

    impl ButtonExit {
        pub(crate) const NAME: &str = "button_exit";
    }

    /// `button_url_exit[<X>,<Y>;<W>,<H>;<name>;<label>;<url>]`
    pub struct ButtonUrlExit {}

    impl ButtonUrlExit {
        pub(crate) const NAME: &str = "button_url_exit";
    }

    /// `image_button_exit[<X>,<Y>;<W>,<H>;<texture name>;<name>;<label>]`
    pub struct ImageButtonExit {}

    impl ImageButtonExit {
        pub(crate) const NAME: &str = "image_button_exit";
    }

    /// `textlist[<X>,<Y>;<W>,<H>;<name>;<listelem 1>,<listelem 2>,...,<listelem n>]`
    /// `textlist[<X>,<Y>;<W>,<H>;<name>;<listelem 1>,<listelem 2>,...,<listelem n>;<selected idx>;<transparent>]`
    pub struct TextList {}

    impl TextList {
        pub(crate) const NAME: &str = "textlist";
    }

    /// `tabheader[<X>,<Y>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]`
    /// `tabheader[<X>,<Y>;<H>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]`
    /// `tabheader[<X>,<Y>;<W>,<H>;<name>;<caption 1>,<caption 2>,...,<caption n>;<current_tab>;<transparent>;<draw_border>]¶`
    pub struct TabHeader {}

    impl TabHeader {
        pub(crate) const NAME: &str = "tabheader";
    }

    /// `box[<X>,<Y>;<W>,<H>;<color>]`
    pub struct Box {}

    impl Box {
        pub(crate) const NAME: &str = "box";
    }

    /// `dropdown[<X>,<Y>;<W>;<name>;<item 1>,<item 2>, ...,<item n>;<selected idx>;<index event>]`
    /// `dropdown[<X>,<Y>;<W>,<H>;<name>;<item 1>,<item 2>, ...,<item n>;<selected idx>;<index event>]`
    pub struct Dropdown {}

    impl Dropdown {
        pub(crate) const NAME: &str = "dropdown";
    }

    /// `checkbox[<X>,<Y>;<name>;<label>;<selected>]`
    pub struct Checkbox {}

    impl Checkbox {
        pub(crate) const NAME: &str = "checkbox";
    }

    /// `scrollbar[<X>,<Y>;<W>,<H>;<orientation>;<name>;<value>]`
    pub struct Scrollbar {}

    impl Scrollbar {
        pub(crate) const NAME: &str = "scrollbar";
    }

    /// `scrollbaroptions[opt1;opt2;...]`
    pub struct ScrollbarOptions {}

    impl ScrollbarOptions {
        pub(crate) const NAME: &str = "scrollbaroptions";
    }

    /// `table[<X>,<Y>;<W>,<H>;<name>;<cell 1>,<cell 2>,...,<cell n>;<selected idx>]`
    pub struct Table {}

    impl Table {
        pub(crate) const NAME: &str = "table";
    }

    /// `tableoptions[<opt 1>;<opt 2>;...]`
    pub struct TableOptions {}

    impl TableOptions {
        pub(crate) const NAME: &str = "tableoptions";
    }

    /// `tablecolumns[<type 1>,<opt 1a>,<opt 1b>,...;<type 2>,<opt 2a>,<opt 2b>;...]`
    pub struct TableColumns {}

    impl TableColumns {
        pub(crate) const NAME: &str = "tablecolumns";
    }

    /// `style[<selector 1>,<selector 2>,...;<prop1>;<prop2>;...]`
    pub struct Style {}

    impl Style {
        pub(crate) const NAME: &str = "style";
    }

    /// `style_type[<selector 1>,<selector 2>,...;<prop1>;<prop2>;...]`
    pub struct StyleType {}

    impl StyleType {
        pub(crate) const NAME: &str = "style_type";
    }

    /// `set_focus[<name>;<force>]`
    pub struct SetFocus {}

    impl SetFocus {
        pub(crate) const NAME: &str = "set_focus";
    }

    enum Orientation {
        Vertical,
        Horizontal,
    }

    /// Defines a rectangle within a 9-sliced texture
    enum Middle {
        All(u32),
        Centered { horizontal: u32, vertical: u32 },
        Free { x1: u32, y1: u32, x2: i32, y2: i32 },
    }
}

#[cfg(test)]
mod test {
    #![expect(clippy::unwrap_used, clippy::too_many_lines, reason = "ok in tests")]
    use crate::{FormspecElement, FormspecError};

    #[test]
    fn read_raw() {
        {
            let mut input = "";
            assert!(FormspecElement::read_raw(&mut input).unwrap().is_none());
            assert!(input.is_empty());
        }
        {
            let mut input = " \t\r\n ";
            assert!(FormspecElement::read_raw(&mut input).unwrap().is_none());
            assert_eq!(input, " \t\r\n ");
        }
        {
            let mut input = "name[]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert!(args.args.is_empty());
            assert!(input.is_empty());
        }
        {
            let mut input = " \t\r\n name \t\r\n []";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert!(args.args.is_empty());
            assert!(input.is_empty());
        }
        {
            let mut input = "name[arg]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(args.args, vec![vec!["arg"]]);
            assert!(input.is_empty());
        }
        {
            let mut input = "name[arg,]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(args.args, vec![vec!["arg"]]);
            assert!(input.is_empty());
        }
        {
            let mut input = "name[arg,;]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(args.args, vec![vec!["arg"]]);
            assert!(input.is_empty());
        }
        {
            let mut input = "name[,,;;,,;,,arg,;,,,]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(args.args, vec![vec![], vec![], vec![], vec!["", "", "arg"]]);
            assert!(input.is_empty());
        }
        {
            let mut input = "name[,,;;,,;,,arg,,arg,,;,,,]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(
                args.args,
                vec![vec![], vec![], vec![], vec!["", "", "arg", "", "arg"]]
            );
            assert!(input.is_empty());
        }
        {
            let mut input = "name[,,;;,,;,,arg;arg,,;,,,]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(
                args.args,
                vec![vec![], vec![], vec![], vec!["", "", "arg"], vec!["arg"]]
            );
            assert!(input.is_empty());
        }
        {
            let mut input = r"name[\,\;\\\[\]]";
            let (name, args) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name, "name");
            assert_eq!(args.args, vec![vec![r"\,\;\\\[\]"]]);
            assert!(input.is_empty());
        }
        {
            let mut input = "name1[]name2[]";

            let (name1, args1) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name1, "name1");
            assert!(args1.args.is_empty());

            let (name2, args2) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name2, "name2");
            assert!(args2.args.is_empty());

            assert!(input.is_empty());
        }
        {
            let mut input = " name1[] name2[] ";

            let (name1, args1) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name1, "name1");
            assert!(args1.args.is_empty());

            let (name2, args2) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name2, "name2");
            assert!(args2.args.is_empty());

            assert_eq!(input, " ");
        }
        {
            let mut input = " \t\r\n name1[] \t\r\n name2[] \t\r\n ";
            let (name1, args1) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name1, "name1");
            assert!(args1.args.is_empty());

            let (name2, args2) = FormspecElement::read_raw(&mut input).unwrap().unwrap();
            assert_eq!(name2, "name2");
            assert!(args2.args.is_empty());

            assert_eq!(input, " \t\r\n ");
        }
        {
            let mut input = " \t\r\n x";
            assert!(matches!(
                FormspecElement::read_raw(&mut input).unwrap_err(),
                FormspecError::PrematureEnd
            ));
            assert_eq!(input, " \t\r\n x");
        }
        {
            let mut input = "x";
            assert!(matches!(
                FormspecElement::read_raw(&mut input).unwrap_err(),
                FormspecError::PrematureEnd
            ));
            assert_eq!(input, "x");
        }
        {
            let mut input = "x[";
            assert!(matches!(
                FormspecElement::read_raw(&mut input).unwrap_err(),
                FormspecError::PrematureEnd
            ));
            assert_eq!(input, "x[");
        }
        {
            let mut input = r"x[\]";
            assert!(matches!(
                FormspecElement::read_raw(&mut input).unwrap_err(),
                FormspecError::PrematureEnd
            ));
            assert_eq!(input, r"x[\]");
        }
    }
}
