import { FC, JSXElementConstructor, ReactElement, ReactNode } from "react";
/**@internal*/
declare global {
    namespace JSX {
        interface IntrinsicElements {
            ["gauntlet:action"]: {
                id?: string;
                label: string;
                onAction: (id: string | undefined) => void;
            };
            ["gauntlet:action_panel_section"]: {
                children?: ElementComponent<typeof Action>;
                title?: string;
            };
            ["gauntlet:action_panel"]: {
                children?: ElementComponent<typeof Action | typeof ActionPanelSection>;
                title?: string;
            };
            ["gauntlet:metadata_link"]: {
                children?: StringComponent;
                label: string;
                href: string;
            };
            ["gauntlet:metadata_tag_item"]: {
                children?: StringComponent;
                onClick?: () => void;
            };
            ["gauntlet:metadata_tag_list"]: {
                children?: ElementComponent<typeof MetadataTagItem>;
                label: string;
            };
            ["gauntlet:metadata_separator"]: {};
            ["gauntlet:metadata_value"]: {
                children?: StringComponent;
                label: string;
            };
            ["gauntlet:metadata_icon"]: {
                icon: Icons;
                label: string;
            };
            ["gauntlet:metadata"]: {
                children?: ElementComponent<typeof MetadataTagList | typeof MetadataLink | typeof MetadataValue | typeof MetadataIcon | typeof MetadataSeparator>;
            };
            ["gauntlet:image"]: {
                source: ImageLike;
            };
            ["gauntlet:svg"]: {
                source: DataSource;
            };
            ["gauntlet:h1"]: {
                children?: StringComponent;
            };
            ["gauntlet:h2"]: {
                children?: StringComponent;
            };
            ["gauntlet:h3"]: {
                children?: StringComponent;
            };
            ["gauntlet:h4"]: {
                children?: StringComponent;
            };
            ["gauntlet:h5"]: {
                children?: StringComponent;
            };
            ["gauntlet:h6"]: {
                children?: StringComponent;
            };
            ["gauntlet:horizontal_break"]: {};
            ["gauntlet:code_block"]: {
                children?: StringComponent;
            };
            ["gauntlet:paragraph"]: {
                children?: StringComponent;
            };
            ["gauntlet:content"]: {
                children?: ElementComponent<typeof Paragraph | typeof Image | typeof Svg | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock>;
            };
            ["gauntlet:detail"]: {
                children?: ElementComponent<typeof ActionPanel | typeof Metadata | typeof Content>;
                isLoading?: boolean;
            };
            ["gauntlet:text_field"]: {
                label?: string;
                value?: string;
                onChange?: (value: string | undefined) => void;
            };
            ["gauntlet:password_field"]: {
                label?: string;
                value?: string;
                onChange?: (value: string | undefined) => void;
            };
            ["gauntlet:checkbox"]: {
                label?: string;
                title?: string;
                value?: boolean;
                onChange?: (value: boolean) => void;
            };
            ["gauntlet:select_item"]: {
                children?: StringComponent;
                value: string;
            };
            ["gauntlet:select"]: {
                children?: ElementComponent<typeof SelectItem>;
                label?: string;
                value?: string;
                onChange?: (value: string | undefined) => void;
            };
            ["gauntlet:separator"]: {};
            ["gauntlet:form"]: {
                children?: ElementComponent<typeof ActionPanel | typeof TextField | typeof PasswordField | typeof Checkbox | typeof Select | typeof Separator>;
                isLoading?: boolean;
            };
            ["gauntlet:inline_separator"]: {
                icon?: Icons;
            };
            ["gauntlet:inline"]: {
                children?: ElementComponent<typeof ActionPanel | typeof Content | typeof InlineSeparator | typeof Content | typeof Content>;
            };
            ["gauntlet:empty_view"]: {
                title: string;
                description?: string;
                image?: ImageLike;
            };
            ["gauntlet:accessory_icon"]: {
                icon: ImageLike;
                tooltip?: string;
            };
            ["gauntlet:accessory_text"]: {
                text: string;
                icon?: ImageLike;
                tooltip?: string;
            };
            ["gauntlet:search_bar"]: {
                value?: string;
                placeholder?: string;
                onChange?: (value: string | undefined) => void;
            };
            ["gauntlet:list_item"]: {
                children?: ElementComponent<typeof TextAccessory | typeof IconAccessory>;
                id: string;
                title: string;
                subtitle?: string;
                icon?: ImageLike;
            };
            ["gauntlet:list_section"]: {
                children?: ElementComponent<typeof ListItem>;
                title: string;
                subtitle?: string;
            };
            ["gauntlet:list"]: {
                children?: ElementComponent<typeof ActionPanel | typeof ListItem | typeof ListSection | typeof SearchBar | typeof EmptyView | typeof Detail>;
                isLoading?: boolean;
                onItemFocusChange?: (itemId: string | undefined) => void;
            };
            ["gauntlet:grid_item"]: {
                children?: ElementComponent<typeof IconAccessory | typeof Content>;
                id: string;
                title?: string;
                subtitle?: string;
            };
            ["gauntlet:grid_section"]: {
                children?: ElementComponent<typeof GridItem>;
                title: string;
                subtitle?: string;
                columns?: number;
            };
            ["gauntlet:grid"]: {
                children?: ElementComponent<typeof ActionPanel | typeof GridItem | typeof GridSection | typeof SearchBar | typeof EmptyView>;
                isLoading?: boolean;
                columns?: number;
                onItemFocusChange?: (itemId: string | undefined) => void;
            };
        }
    }
}
export type ElementParams<Comp extends FC<any>> = Comp extends FC<infer Params> ? Params : never;
export type Element<Comp extends FC<any>> = ReactElement<ElementParams<Comp>, JSXElementConstructor<ElementParams<Comp>>>;
export type StringNode = string | number;
export type EmptyNode = boolean | null | undefined;
export type ElementComponent<Comp extends FC<any>> = Element<Comp> | EmptyNode | Iterable<ElementComponent<Comp>>;
export type StringComponent = StringNode | EmptyNode | Iterable<StringComponent>;
export type StringOrElementComponent<Comp extends FC<any>> = StringNode | EmptyNode | Element<Comp> | Iterable<StringOrElementComponent<Comp>>;
export enum Icons {
    PersonAdd = "PersonAdd",
    Airplane = "Airplane",
    Alarm = "Alarm",
    AlignCentre = "AlignCentre",
    AlignLeft = "AlignLeft",
    AlignRight = "AlignRight",
    ArrowClockwise = "ArrowClockwise",
    ArrowCounterClockwise = "ArrowCounterClockwise",
    ArrowDown = "ArrowDown",
    ArrowLeft = "ArrowLeft",
    ArrowRight = "ArrowRight",
    ArrowUp = "ArrowUp",
    ArrowLeftRight = "ArrowLeftRight",
    ArrowsContract = "ArrowsContract",
    ArrowsExpand = "ArrowsExpand",
    AtSymbol = "AtSymbol",
    Cash = "Cash",
    Battery = "Battery",
    BatteryCharging = "BatteryCharging",
    Bell = "Bell",
    BellDisabled = "BellDisabled",
    Document = "Document",
    DocumentAdd = "DocumentAdd",
    DocumentDelete = "DocumentDelete",
    Bluetooth = "Bluetooth",
    Bold = "Bold",
    Book = "Book",
    Bookmark = "Bookmark",
    Box = "Box",
    Bug = "Bug",
    Building = "Building",
    BulletPoints = "BulletPoints",
    Calculator = "Calculator",
    Calendar = "Calendar",
    Camera = "Camera",
    Car = "Car",
    Cart = "Cart",
    Checkmark = "Checkmark",
    ChevronDown = "ChevronDown",
    ChevronLeft = "ChevronLeft",
    ChevronRight = "ChevronRight",
    ChevronUp = "ChevronUp",
    ChevronExpand = "ChevronExpand",
    Circle = "Circle",
    Clipboard = "Clipboard",
    Clock = "Clock",
    Cloud = "Cloud",
    CloudLightning = "CloudLightning",
    CloudRain = "CloudRain",
    CloudSnow = "CloudSnow",
    CloudSun = "CloudSun",
    Code = "Code",
    Gear = "Gear",
    Coin = "Coin",
    Command = "Command",
    Compass = "Compass",
    CreditCard = "CreditCard",
    Crop = "Crop",
    Dot = "Dot",
    Download = "Download",
    Eject = "Eject",
    ThreeDots = "ThreeDots",
    Envelope = "Envelope",
    Eraser = "Eraser",
    ExclamationMark = "ExclamationMark",
    Eye = "Eye",
    EyeDisabled = "EyeDisabled",
    EyeDropper = "EyeDropper",
    Female = "Female",
    Film = "Film",
    Filter = "Filter",
    Fingerprint = "Fingerprint",
    Flag = "Flag",
    Folder = "Folder",
    FolderAdd = "FolderAdd",
    FolderDelete = "FolderDelete",
    Forward = "Forward",
    GameController = "GameController",
    Virus = "Virus",
    Gift = "Gift",
    Glasses = "Glasses",
    Globe = "Globe",
    Hammer = "Hammer",
    HardDrive = "HardDrive",
    Headphones = "Headphones",
    Heart = "Heart",
    Heartbeat = "Heartbeat",
    Hourglass = "Hourglass",
    House = "House",
    Image = "Image",
    Info = "Info",
    Italics = "Italics",
    Key = "Key",
    Keyboard = "Keyboard",
    Layers = "Layers",
    LightBulb = "LightBulb",
    LightBulbDisabled = "LightBulbDisabled",
    Link = "Link",
    List = "List",
    Lock = "Lock",
    LockUnlocked = "LockUnlocked",
    Male = "Male",
    Map = "Map",
    Maximize = "Maximize",
    Megaphone = "Megaphone",
    MemoryModule = "MemoryModule",
    MemoryStick = "MemoryStick",
    Message = "Message",
    Microphone = "Microphone",
    MicrophoneDisabled = "MicrophoneDisabled",
    Minimize = "Minimize",
    Minus = "Minus",
    Mobile = "Mobile",
    Moon = "Moon",
    Mouse = "Mouse",
    Multiply = "Multiply",
    Music = "Music",
    Network = "Network",
    Paperclip = "Paperclip",
    Paragraph = "Paragraph",
    Pause = "Pause",
    Pencil = "Pencil",
    Person = "Person",
    Phone = "Phone",
    PieChart = "PieChart",
    Capsule = "Capsule",
    Play = "Play",
    Plug = "Plug",
    Plus = "Plus",
    Power = "Power",
    Printer = "Printer",
    QuestionMark = "QuestionMark",
    Quotes = "Quotes",
    Receipt = "Receipt",
    PersonRemove = "PersonRemove",
    Repeat = "Repeat",
    Reply = "Reply",
    Rewind = "Rewind",
    Rocket = "Rocket",
    Shield = "Shield",
    Shuffle = "Shuffle",
    Snippets = "Snippets",
    Snowflake = "Snowflake",
    Star = "Star",
    Stop = "Stop",
    Stopwatch = "Stopwatch",
    StrikeThrough = "StrikeThrough",
    Sun = "Sun",
    Scissors = "Scissors",
    Tag = "Tag",
    Thermometer = "Thermometer",
    Terminal = "Terminal",
    Text = "Text",
    TextCursor = "TextCursor",
    Trash = "Trash",
    Tree = "Tree",
    Trophy = "Trophy",
    People = "People",
    Umbrella = "Umbrella",
    Underline = "Underline",
    Upload = "Upload",
    Wallet = "Wallet",
    Wand = "Wand",
    Wifi = "Wifi",
    WifiDisabled = "WifiDisabled",
    Window = "Window",
    Tools = "Tools",
    Watch = "Watch",
    XMark = "XMark",
    Indent = "Indent",
    Unindent = "Unindent"
}
export type DataSourceUrl = {
    url: string;
};
export type DataSourceAsset = {
    asset: string;
};
export type DataSource = DataSourceUrl | DataSourceAsset;
export type ImageLike = DataSource | Icons;
export interface ActionProps {
    id?: string;
    label: string;
    onAction: (id: string | undefined) => void;
}
export const Action: FC<ActionProps> = (props: ActionProps): ReactNode => {
    return <gauntlet:action id={props.id} label={props.label} onAction={props.onAction}></gauntlet:action>;
};
export interface ActionPanelSectionProps {
    children?: ElementComponent<typeof Action>;
    title?: string;
}
export const ActionPanelSection: FC<ActionPanelSectionProps> & {
    Action: typeof Action;
} = (props: ActionPanelSectionProps): ReactNode => {
    return <gauntlet:action_panel_section title={props.title}>{props.children}</gauntlet:action_panel_section>;
};
ActionPanelSection.Action = Action;
export interface ActionPanelProps {
    children?: ElementComponent<typeof Action | typeof ActionPanelSection>;
    title?: string;
}
export const ActionPanel: FC<ActionPanelProps> & {
    Action: typeof Action;
    Section: typeof ActionPanelSection;
} = (props: ActionPanelProps): ReactNode => {
    return <gauntlet:action_panel title={props.title}>{props.children}</gauntlet:action_panel>;
};
ActionPanel.Action = Action;
ActionPanel.Section = ActionPanelSection;
export interface MetadataLinkProps {
    children?: StringComponent;
    label: string;
    href: string;
}
export const MetadataLink: FC<MetadataLinkProps> = (props: MetadataLinkProps): ReactNode => {
    return <gauntlet:metadata_link label={props.label} href={props.href}>{props.children}</gauntlet:metadata_link>;
};
export interface MetadataTagItemProps {
    children?: StringComponent;
    onClick?: () => void;
}
export const MetadataTagItem: FC<MetadataTagItemProps> = (props: MetadataTagItemProps): ReactNode => {
    return <gauntlet:metadata_tag_item onClick={props.onClick}>{props.children}</gauntlet:metadata_tag_item>;
};
export interface MetadataTagListProps {
    children?: ElementComponent<typeof MetadataTagItem>;
    label: string;
}
export const MetadataTagList: FC<MetadataTagListProps> & {
    Item: typeof MetadataTagItem;
} = (props: MetadataTagListProps): ReactNode => {
    return <gauntlet:metadata_tag_list label={props.label}>{props.children}</gauntlet:metadata_tag_list>;
};
MetadataTagList.Item = MetadataTagItem;
export const MetadataSeparator: FC = (): ReactNode => {
    return <gauntlet:metadata_separator></gauntlet:metadata_separator>;
};
export interface MetadataValueProps {
    children?: StringComponent;
    label: string;
}
export const MetadataValue: FC<MetadataValueProps> = (props: MetadataValueProps): ReactNode => {
    return <gauntlet:metadata_value label={props.label}>{props.children}</gauntlet:metadata_value>;
};
export interface MetadataIconProps {
    icon: Icons;
    label: string;
}
export const MetadataIcon: FC<MetadataIconProps> = (props: MetadataIconProps): ReactNode => {
    return <gauntlet:metadata_icon icon={props.icon} label={props.label}></gauntlet:metadata_icon>;
};
export interface MetadataProps {
    children?: ElementComponent<typeof MetadataTagList | typeof MetadataLink | typeof MetadataValue | typeof MetadataIcon | typeof MetadataSeparator>;
}
export const Metadata: FC<MetadataProps> & {
    TagList: typeof MetadataTagList;
    Link: typeof MetadataLink;
    Value: typeof MetadataValue;
    Icon: typeof MetadataIcon;
    Separator: typeof MetadataSeparator;
} = (props: MetadataProps): ReactNode => {
    return <gauntlet:metadata>{props.children}</gauntlet:metadata>;
};
Metadata.TagList = MetadataTagList;
Metadata.Link = MetadataLink;
Metadata.Value = MetadataValue;
Metadata.Icon = MetadataIcon;
Metadata.Separator = MetadataSeparator;
export interface ImageProps {
    source: ImageLike;
}
export const Image: FC<ImageProps> = (props: ImageProps): ReactNode => {
    return <gauntlet:image source={props.source}></gauntlet:image>;
};
export interface SvgProps {
    source: DataSource;
}
export const Svg: FC<SvgProps> = (props: SvgProps): ReactNode => {
    return <gauntlet:svg source={props.source}></gauntlet:svg>;
};
export interface H1Props {
    children?: StringComponent;
}
export const H1: FC<H1Props> = (props: H1Props): ReactNode => {
    return <gauntlet:h1>{props.children}</gauntlet:h1>;
};
export interface H2Props {
    children?: StringComponent;
}
export const H2: FC<H2Props> = (props: H2Props): ReactNode => {
    return <gauntlet:h2>{props.children}</gauntlet:h2>;
};
export interface H3Props {
    children?: StringComponent;
}
export const H3: FC<H3Props> = (props: H3Props): ReactNode => {
    return <gauntlet:h3>{props.children}</gauntlet:h3>;
};
export interface H4Props {
    children?: StringComponent;
}
export const H4: FC<H4Props> = (props: H4Props): ReactNode => {
    return <gauntlet:h4>{props.children}</gauntlet:h4>;
};
export interface H5Props {
    children?: StringComponent;
}
export const H5: FC<H5Props> = (props: H5Props): ReactNode => {
    return <gauntlet:h5>{props.children}</gauntlet:h5>;
};
export interface H6Props {
    children?: StringComponent;
}
export const H6: FC<H6Props> = (props: H6Props): ReactNode => {
    return <gauntlet:h6>{props.children}</gauntlet:h6>;
};
export const HorizontalBreak: FC = (): ReactNode => {
    return <gauntlet:horizontal_break></gauntlet:horizontal_break>;
};
export interface CodeBlockProps {
    children?: StringComponent;
}
export const CodeBlock: FC<CodeBlockProps> = (props: CodeBlockProps): ReactNode => {
    return <gauntlet:code_block>{props.children}</gauntlet:code_block>;
};
export interface ParagraphProps {
    children?: StringComponent;
}
export const Paragraph: FC<ParagraphProps> = (props: ParagraphProps): ReactNode => {
    return <gauntlet:paragraph>{props.children}</gauntlet:paragraph>;
};
export interface ContentProps {
    children?: ElementComponent<typeof Paragraph | typeof Image | typeof Svg | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock>;
}
export const Content: FC<ContentProps> & {
    Paragraph: typeof Paragraph;
    Image: typeof Image;
    Svg: typeof Svg;
    H1: typeof H1;
    H2: typeof H2;
    H3: typeof H3;
    H4: typeof H4;
    H5: typeof H5;
    H6: typeof H6;
    HorizontalBreak: typeof HorizontalBreak;
    CodeBlock: typeof CodeBlock;
} = (props: ContentProps): ReactNode => {
    return <gauntlet:content>{props.children}</gauntlet:content>;
};
Content.Paragraph = Paragraph;
Content.Image = Image;
Content.Svg = Svg;
Content.H1 = H1;
Content.H2 = H2;
Content.H3 = H3;
Content.H4 = H4;
Content.H5 = H5;
Content.H6 = H6;
Content.HorizontalBreak = HorizontalBreak;
Content.CodeBlock = CodeBlock;
export interface DetailProps {
    children?: ElementComponent<typeof Metadata | typeof Content>;
    isLoading?: boolean;
    actions?: ElementComponent<typeof ActionPanel>;
}
export const Detail: FC<DetailProps> & {
    Metadata: typeof Metadata;
    Content: typeof Content;
} = (props: DetailProps): ReactNode => {
    return <gauntlet:detail isLoading={props.isLoading}>{props.actions as any}{props.children}</gauntlet:detail>;
};
Detail.Metadata = Metadata;
Detail.Content = Content;
export interface TextFieldProps {
    label?: string;
    value?: string;
    onChange?: (value: string | undefined) => void;
}
export const TextField: FC<TextFieldProps> = (props: TextFieldProps): ReactNode => {
    return <gauntlet:text_field label={props.label} value={props.value} onChange={props.onChange}></gauntlet:text_field>;
};
export interface PasswordFieldProps {
    label?: string;
    value?: string;
    onChange?: (value: string | undefined) => void;
}
export const PasswordField: FC<PasswordFieldProps> = (props: PasswordFieldProps): ReactNode => {
    return <gauntlet:password_field label={props.label} value={props.value} onChange={props.onChange}></gauntlet:password_field>;
};
export interface CheckboxProps {
    label?: string;
    title?: string;
    value?: boolean;
    onChange?: (value: boolean) => void;
}
export const Checkbox: FC<CheckboxProps> = (props: CheckboxProps): ReactNode => {
    return <gauntlet:checkbox label={props.label} title={props.title} value={props.value} onChange={props.onChange}></gauntlet:checkbox>;
};
export interface SelectItemProps {
    children?: StringComponent;
    value: string;
}
export const SelectItem: FC<SelectItemProps> = (props: SelectItemProps): ReactNode => {
    return <gauntlet:select_item value={props.value}>{props.children}</gauntlet:select_item>;
};
export interface SelectProps {
    children?: ElementComponent<typeof SelectItem>;
    label?: string;
    value?: string;
    onChange?: (value: string | undefined) => void;
}
export const Select: FC<SelectProps> & {
    Item: typeof SelectItem;
} = (props: SelectProps): ReactNode => {
    return <gauntlet:select label={props.label} value={props.value} onChange={props.onChange}>{props.children}</gauntlet:select>;
};
Select.Item = SelectItem;
export const Separator: FC = (): ReactNode => {
    return <gauntlet:separator></gauntlet:separator>;
};
export interface FormProps {
    children?: ElementComponent<typeof TextField | typeof PasswordField | typeof Checkbox | typeof Select | typeof Separator>;
    isLoading?: boolean;
    actions?: ElementComponent<typeof ActionPanel>;
}
export const Form: FC<FormProps> & {
    TextField: typeof TextField;
    PasswordField: typeof PasswordField;
    Checkbox: typeof Checkbox;
    Select: typeof Select;
    Separator: typeof Separator;
} = (props: FormProps): ReactNode => {
    return <gauntlet:form isLoading={props.isLoading}>{props.actions as any}{props.children}</gauntlet:form>;
};
Form.TextField = TextField;
Form.PasswordField = PasswordField;
Form.Checkbox = Checkbox;
Form.Select = Select;
Form.Separator = Separator;
export interface InlineSeparatorProps {
    icon?: Icons;
}
export const InlineSeparator: FC<InlineSeparatorProps> = (props: InlineSeparatorProps): ReactNode => {
    return <gauntlet:inline_separator icon={props.icon}></gauntlet:inline_separator>;
};
export interface InlineProps {
    children?: ElementComponent<typeof Content | typeof InlineSeparator | typeof Content | typeof Content>;
    actions?: ElementComponent<typeof ActionPanel>;
}
export const Inline: FC<InlineProps> & {
    Left: typeof Content;
    Separator: typeof InlineSeparator;
    Right: typeof Content;
    Center: typeof Content;
} = (props: InlineProps): ReactNode => {
    return <gauntlet:inline>{props.actions as any}{props.children}</gauntlet:inline>;
};
Inline.Left = Content;
Inline.Separator = InlineSeparator;
Inline.Right = Content;
Inline.Center = Content;
export interface EmptyViewProps {
    title: string;
    description?: string;
    image?: ImageLike;
}
export const EmptyView: FC<EmptyViewProps> = (props: EmptyViewProps): ReactNode => {
    return <gauntlet:empty_view title={props.title} description={props.description} image={props.image}></gauntlet:empty_view>;
};
export interface IconAccessoryProps {
    icon: ImageLike;
    tooltip?: string;
}
export const IconAccessory: FC<IconAccessoryProps> = (props: IconAccessoryProps): ReactNode => {
    return <gauntlet:accessory_icon icon={props.icon} tooltip={props.tooltip}></gauntlet:accessory_icon>;
};
export interface TextAccessoryProps {
    text: string;
    icon?: ImageLike;
    tooltip?: string;
}
export const TextAccessory: FC<TextAccessoryProps> = (props: TextAccessoryProps): ReactNode => {
    return <gauntlet:accessory_text text={props.text} icon={props.icon} tooltip={props.tooltip}></gauntlet:accessory_text>;
};
export interface SearchBarProps {
    value?: string;
    placeholder?: string;
    onChange?: (value: string | undefined) => void;
}
export const SearchBar: FC<SearchBarProps> = (props: SearchBarProps): ReactNode => {
    return <gauntlet:search_bar value={props.value} placeholder={props.placeholder} onChange={props.onChange}></gauntlet:search_bar>;
};
export interface ListItemProps {
    id: string;
    title: string;
    subtitle?: string;
    icon?: ImageLike;
    accessories?: (ElementComponent<typeof TextAccessory> | ElementComponent<typeof IconAccessory>)[];
}
export const ListItem: FC<ListItemProps> = (props: ListItemProps): ReactNode => {
    return <gauntlet:list_item id={props.id} title={props.title} subtitle={props.subtitle} icon={props.icon}>{props.accessories as any}</gauntlet:list_item>;
};
export interface ListSectionProps {
    children?: ElementComponent<typeof ListItem>;
    title: string;
    subtitle?: string;
}
export const ListSection: FC<ListSectionProps> & {
    Item: typeof ListItem;
} = (props: ListSectionProps): ReactNode => {
    return <gauntlet:list_section title={props.title} subtitle={props.subtitle}>{props.children}</gauntlet:list_section>;
};
ListSection.Item = ListItem;
export interface ListProps {
    children?: ElementComponent<typeof ListItem | typeof ListSection | typeof SearchBar | typeof EmptyView | typeof Detail>;
    actions?: ElementComponent<typeof ActionPanel>;
    isLoading?: boolean;
    onItemFocusChange?: (itemId: string | undefined) => void;
}
export const List: FC<ListProps> & {
    Item: typeof ListItem;
    Section: typeof ListSection;
    SearchBar: typeof SearchBar;
    EmptyView: typeof EmptyView;
    Detail: typeof Detail;
} = (props: ListProps): ReactNode => {
    return <gauntlet:list isLoading={props.isLoading} onItemFocusChange={props.onItemFocusChange}>{props.actions as any}{props.children}</gauntlet:list>;
};
List.Item = ListItem;
List.Section = ListSection;
List.SearchBar = SearchBar;
List.EmptyView = EmptyView;
List.Detail = Detail;
export interface GridItemProps {
    children?: ElementComponent<typeof Content>;
    id: string;
    title?: string;
    subtitle?: string;
    accessory?: ElementComponent<typeof IconAccessory>;
}
export const GridItem: FC<GridItemProps> & {
    Content: typeof Content;
} = (props: GridItemProps): ReactNode => {
    return <gauntlet:grid_item id={props.id} title={props.title} subtitle={props.subtitle}>{props.accessory as any}{props.children}</gauntlet:grid_item>;
};
GridItem.Content = Content;
export interface GridSectionProps {
    children?: ElementComponent<typeof GridItem>;
    title: string;
    subtitle?: string;
    columns?: number;
}
export const GridSection: FC<GridSectionProps> & {
    Item: typeof GridItem;
} = (props: GridSectionProps): ReactNode => {
    return <gauntlet:grid_section title={props.title} subtitle={props.subtitle} columns={props.columns}>{props.children}</gauntlet:grid_section>;
};
GridSection.Item = GridItem;
export interface GridProps {
    children?: ElementComponent<typeof GridItem | typeof GridSection | typeof SearchBar | typeof EmptyView>;
    isLoading?: boolean;
    actions?: ElementComponent<typeof ActionPanel>;
    columns?: number;
    onItemFocusChange?: (itemId: string | undefined) => void;
}
export const Grid: FC<GridProps> & {
    Item: typeof GridItem;
    Section: typeof GridSection;
    SearchBar: typeof SearchBar;
    EmptyView: typeof EmptyView;
} = (props: GridProps): ReactNode => {
    return <gauntlet:grid isLoading={props.isLoading} columns={props.columns} onItemFocusChange={props.onItemFocusChange}>{props.actions as any}{props.children}</gauntlet:grid>;
};
Grid.Item = GridItem;
Grid.Section = GridSection;
Grid.SearchBar = SearchBar;
Grid.EmptyView = EmptyView;
