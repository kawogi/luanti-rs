//! Contains parsers and serializers for Luanti's Formspec-strings.

#![expect(unused, reason = "// TODO")]

use std::fmt::Display;

struct Formspec {
    elements: Vec<FormspecElement>,
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

struct Dimensions {
    pub pos: Position,
    pub size: Size,
}

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

enum FormspecError {
    NameIsQuit,
    NameIsKey,
    NameIsEmpty,
    NameInvalidChar(String),
}

impl Display for FormspecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameIsQuit => f.write_str("'quit' is a reserved formspec element name"),
            Self::NameIsKey => f.write_str(
                "formspec element names starting with 'key_' are reserved to pass key press events",
            ),
            Self::NameIsEmpty => f.write_str("formspec element names may not be empty"),
            Self::NameInvalidChar(name) => write!(
                f,
                "formspec element name contains at least one illegal character: {name}"
            ),
        }
    }
}
