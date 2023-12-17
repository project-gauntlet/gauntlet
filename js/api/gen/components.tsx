import { FC, JSXElementConstructor, ReactElement, ReactNode } from "react";
/**@internal*/
declare global {
    namespace JSX {
        interface IntrinsicElements {
            ["gauntlet:metadata_link"]: {
                children?: StringComponent;
                label: string;
                href: string;
            };
            ["gauntlet:metadata_tag"]: {
                children?: StringComponent;
                onClick?: () => void;
            };
            ["gauntlet:metadata_tags"]: {
                children?: ElementComponent<typeof MetadataTag>;
                label: string;
            };
            ["gauntlet:metadata_separator"]: {};
            ["gauntlet:metadata_value"]: {
                children?: StringComponent;
                label: string;
            };
            ["gauntlet:metadata_icon"]: {
                icon: string;
                label: string;
            };
            ["gauntlet:metadata"]: {
                children?: ElementComponent<typeof MetadataTags | typeof MetadataLink | typeof MetadataValue | typeof MetadataIcon | typeof MetadataSeparator>;
            };
            ["gauntlet:link"]: {
                children?: StringComponent;
                href: string;
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
            ["gauntlet:paragraph"]: {
                children?: StringOrElementComponent<typeof Link | typeof Code>;
            };
            ["gauntlet:content"]: {
                children?: ElementComponent<typeof Paragraph | typeof Link | typeof Image | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock | typeof Code>;
            };
            ["gauntlet:detail"]: {
                children?: ElementComponent<typeof Metadata | typeof Content>;
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
export interface MetadataLinkProps {
    children?: StringComponent;
    label: string;
    href: string;
}
export const MetadataLink: FC<MetadataLinkProps> = (props: MetadataLinkProps): ReactNode => {
    return <gauntlet:metadata_link children={props.children} label={props.label} href={props.href}/>;
};
export interface MetadataTagProps {
    children?: StringComponent;
    onClick?: () => void;
}
export const MetadataTag: FC<MetadataTagProps> = (props: MetadataTagProps): ReactNode => {
    return <gauntlet:metadata_tag children={props.children} onClick={props.onClick}/>;
};
export interface MetadataTagsProps {
    children?: ElementComponent<typeof MetadataTag>;
    label: string;
}
export const MetadataTags: FC<MetadataTagsProps> & {
    Tag: typeof MetadataTag;
} = (props: MetadataTagsProps): ReactNode => {
    return <gauntlet:metadata_tags children={props.children} label={props.label}/>;
};
MetadataTags.Tag = MetadataTag;
export const MetadataSeparator: FC = (): ReactNode => {
    return <gauntlet:metadata_separator />;
};
export interface MetadataValueProps {
    children?: StringComponent;
    label: string;
}
export const MetadataValue: FC<MetadataValueProps> = (props: MetadataValueProps): ReactNode => {
    return <gauntlet:metadata_value children={props.children} label={props.label}/>;
};
export interface MetadataIconProps {
    icon: string;
    label: string;
}
export const MetadataIcon: FC<MetadataIconProps> = (props: MetadataIconProps): ReactNode => {
    return <gauntlet:metadata_icon icon={props.icon} label={props.label}/>;
};
export interface MetadataProps {
    children?: ElementComponent<typeof MetadataTags | typeof MetadataLink | typeof MetadataValue | typeof MetadataIcon | typeof MetadataSeparator>;
}
export const Metadata: FC<MetadataProps> & {
    Tags: typeof MetadataTags;
    Link: typeof MetadataLink;
    Value: typeof MetadataValue;
    Icon: typeof MetadataIcon;
    Separator: typeof MetadataSeparator;
} = (props: MetadataProps): ReactNode => {
    return <gauntlet:metadata children={props.children}/>;
};
Metadata.Tags = MetadataTags;
Metadata.Link = MetadataLink;
Metadata.Value = MetadataValue;
Metadata.Icon = MetadataIcon;
Metadata.Separator = MetadataSeparator;
export interface LinkProps {
    children?: StringComponent;
    href: string;
}
export const Link: FC<LinkProps> = (props: LinkProps): ReactNode => {
    return <gauntlet:link children={props.children} href={props.href}/>;
};
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
export interface ParagraphProps {
    children?: StringOrElementComponent<typeof Link | typeof Code>;
}
export const Paragraph: FC<ParagraphProps> & {
    Link: typeof Link;
    Code: typeof Code;
} = (props: ParagraphProps): ReactNode => {
    return <gauntlet:paragraph children={props.children}/>;
};
Paragraph.Link = Link;
Paragraph.Code = Code;
export interface ContentProps {
    children?: ElementComponent<typeof Paragraph | typeof Link | typeof Image | typeof H1 | typeof H2 | typeof H3 | typeof H4 | typeof H5 | typeof H6 | typeof HorizontalBreak | typeof CodeBlock | typeof Code>;
}
export const Content: FC<ContentProps> & {
    Paragraph: typeof Paragraph;
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
Content.Paragraph = Paragraph;
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
    children?: ElementComponent<typeof Metadata | typeof Content>;
}
export const Detail: FC<DetailProps> & {
    Metadata: typeof Metadata;
    Content: typeof Content;
} = (props: DetailProps): ReactNode => {
    return <gauntlet:detail children={props.children}/>;
};
Detail.Metadata = Metadata;
Detail.Content = Content;
