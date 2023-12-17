import { FC, JSXElementConstructor, ReactElement, ReactNode } from "react";
/**@internal*/
declare global {
    namespace JSX {
        interface IntrinsicElements {
            ["gauntlet:text"]: {
                children?: StringComponent;
            };
            ["gauntlet:link"]: {
                children?: StringComponent;
                href: string;
            };
            ["gauntlet:tag"]: {
                children?: StringComponent;
                color?: string;
                onClick?: () => void;
            };
            ["gauntlet:metadata_item"]: {
                children?: Component<typeof Text | typeof Link | typeof Tag>;
            };
            ["gauntlet:separator"]: {};
            ["gauntlet:metadata"]: {
                children?: Component<typeof MetadataItem | typeof Separator>;
            };
            ["gauntlet:image"]: {};
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
            ["gauntlet:code"]: {
                children?: StringComponent;
            };
            ["gauntlet:content"]: {
                children?: Component<typeof Text | typeof Link | typeof Image | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock | typeof Code>;
            };
            ["gauntlet:detail"]: {
                children?: Component<typeof Metadata | typeof Content>;
            };
        }
    }
}
export type ElementParams<Comp extends FC<any>> = Comp extends FC<infer Params> ? Params : never;
export type Element<Comp extends FC<any>> = ReactElement<ElementParams<Comp>, JSXElementConstructor<ElementParams<Comp>>>;
export type StringNode = string | number;
export type EmptyNode = boolean | null | undefined;
export type Component<Comp extends FC<any>> = Element<Comp> | EmptyNode | Iterable<Component<Comp>>;
export type StringComponent = StringNode | EmptyNode | Iterable<StringComponent>;
export interface TextProps {
    children?: StringComponent;
}
export const Text: FC<TextProps> = (props: TextProps): ReactNode => {
    return <gauntlet:text children={props.children}/>;
};
export interface LinkProps {
    children?: StringComponent;
    href: string;
}
export const Link: FC<LinkProps> = (props: LinkProps): ReactNode => {
    return <gauntlet:link children={props.children} href={props.href}/>;
};
export interface TagProps {
    children?: StringComponent;
    color?: string;
    onClick?: () => void;
}
export const Tag: FC<TagProps> = (props: TagProps): ReactNode => {
    return <gauntlet:tag children={props.children} color={props.color} onClick={props.onClick}/>;
};
export interface MetadataItemProps {
    children?: Component<typeof Text | typeof Link | typeof Tag>;
}
export const MetadataItem: FC<MetadataItemProps> & {
    Text: typeof Text;
    Link: typeof Link;
    Tag: typeof Tag;
} = (props: MetadataItemProps): ReactNode => {
    return <gauntlet:metadata_item children={props.children}/>;
};
MetadataItem.Text = Text;
MetadataItem.Link = Link;
MetadataItem.Tag = Tag;
export const Separator: FC = (): ReactNode => {
    return <gauntlet:separator />;
};
export interface MetadataProps {
    children?: Component<typeof MetadataItem | typeof Separator>;
}
export const Metadata: FC<MetadataProps> & {
    Item: typeof MetadataItem;
    Separator: typeof Separator;
} = (props: MetadataProps): ReactNode => {
    return <gauntlet:metadata children={props.children}/>;
};
Metadata.Item = MetadataItem;
Metadata.Separator = Separator;
export const Image: FC = (): ReactNode => {
    return <gauntlet:image />;
};
export interface H1Props {
    children?: StringComponent;
}
export const H1: FC<H1Props> = (props: H1Props): ReactNode => {
    return <gauntlet:h1 children={props.children}/>;
};
export interface H2Props {
    children?: StringComponent;
}
export const H2: FC<H2Props> = (props: H2Props): ReactNode => {
    return <gauntlet:h2 children={props.children}/>;
};
export interface H3Props {
    children?: StringComponent;
}
export const H3: FC<H3Props> = (props: H3Props): ReactNode => {
    return <gauntlet:h3 children={props.children}/>;
};
export interface H4Props {
    children?: StringComponent;
}
export const H4: FC<H4Props> = (props: H4Props): ReactNode => {
    return <gauntlet:h4 children={props.children}/>;
};
export interface H5Props {
    children?: StringComponent;
}
export const H5: FC<H5Props> = (props: H5Props): ReactNode => {
    return <gauntlet:h5 children={props.children}/>;
};
export interface H6Props {
    children?: StringComponent;
}
export const H6: FC<H6Props> = (props: H6Props): ReactNode => {
    return <gauntlet:h6 children={props.children}/>;
};
export const HorizontalBreak: FC = (): ReactNode => {
    return <gauntlet:horizontal_break />;
};
export interface CodeBlockProps {
    children?: StringComponent;
}
export const CodeBlock: FC<CodeBlockProps> = (props: CodeBlockProps): ReactNode => {
    return <gauntlet:code_block children={props.children}/>;
};
export interface CodeProps {
    children?: StringComponent;
}
export const Code: FC<CodeProps> = (props: CodeProps): ReactNode => {
    return <gauntlet:code children={props.children}/>;
};
export interface ContentProps {
    children?: Component<typeof Text | typeof Link | typeof Image | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock | typeof Code>;
}
export const Content: FC<ContentProps> & {
    Text: typeof Text;
    Link: typeof Link;
    Image: typeof Image;
    H1: typeof H1;
    H2: typeof H2;
    H3: typeof H3;
    H4: typeof H4;
    H5: typeof H5;
    H6: typeof H6;
    HorizontalBreak: typeof HorizontalBreak;
    CodeBlock: typeof CodeBlock;
    Code: typeof Code;
} = (props: ContentProps): ReactNode => {
    return <gauntlet:content children={props.children}/>;
};
Content.Text = Text;
Content.Link = Link;
Content.Image = Image;
Content.H1 = H1;
Content.H2 = H2;
Content.H3 = H3;
Content.H4 = H4;
Content.H5 = H5;
Content.H6 = H6;
Content.HorizontalBreak = HorizontalBreak;
Content.CodeBlock = CodeBlock;
Content.Code = Code;
export interface DetailProps {
    children?: Component<typeof Metadata | typeof Content>;
}
export const Detail: FC<DetailProps> & {
    Metadata: typeof Metadata;
    Content: typeof Content;
} = (props: DetailProps): ReactNode => {
    return <gauntlet:detail children={props.children}/>;
};
Detail.Metadata = Metadata;
Detail.Content = Content;
