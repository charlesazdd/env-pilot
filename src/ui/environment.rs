use super::theme;
use crate::{
    app::{App, Language, Message, Page},
    core::environment::{self, Entry, Snapshot},
};
use iced::{
    Element, Length,
    widget::text::Wrapping,
    widget::{Space, button, column, container, mouse_area, row, scrollable, text, text_input},
};

/// 解释当前声明无法编辑的具体原因，并提示可编辑声明如何进入编辑。
fn edit_reason(
    page: Page,
    entry: &Entry,
    editing_other: bool,
    show_values: bool,
    language: Language,
) -> Option<&'static str> {
    if page == Page::System {
        Some(language.text("System declarations are read-only", "系统级声明仅支持查看"))
    } else if entry.dynamic {
        Some(language.text(
            "Dynamic expressions cannot be edited yet",
            "动态表达式暂不支持编辑",
        ))
    } else if !entry.toggleable {
        Some(language.text(
            "This declaration format cannot be edited yet",
            "此声明格式暂不支持编辑",
        ))
    } else if editing_other {
        Some(language.text("Finish the current edit first", "请先完成当前编辑"))
    } else if !show_values {
        Some(language.text("Show values to edit", "显示值后可双击编辑"))
    } else {
        None
    }
}

/// 以可搜索的只读列表呈现进程快照，默认隐藏所有值以避免意外泄露。
pub fn view<'a>(snapshot: &'a Snapshot, app: &'a App) -> Element<'a, Message> {
    let process = &app.process;
    let query = &app.query;
    let show_values = app.show_values;
    let language = app.language;
    let page = app.page;
    let error = app.error.as_deref();
    let editing = app.editing.as_ref();
    let entries = &snapshot.entries;
    let matches = environment::filter(entries, query);
    let count = matches.len();
    let mut rows = column![].spacing(6);
    for entry in matches {
        let value = if show_values {
            entry.value.as_str()
        } else {
            "••••••••"
        };
        let index = entries.iter().position(|item| std::ptr::eq(item, entry));
        let is_editing = editing.is_some_and(|(selected, _)| {
            selected.source == entry.source && selected.line_index == entry.line_index
        });
        let value_column: Element<'_, Message> =
            if let Some((_, draft)) = editing.filter(|_| is_editing) {
                mouse_area(
                    text_input("", draft)
                        .id("envpilot-edit")
                        .on_input(Message::EditValue)
                        .on_submit(Message::SubmitEdit)
                        .style(theme::soft_input)
                        .padding(12)
                        .width(Length::FillPortion(3)),
                )
                .on_enter(Message::EditHover(true))
                .on_exit(Message::EditHover(false))
                .into()
            } else if entry.dynamic {
                let current = environment::process_value(&process.entries, &entry.name);
                let current_value = match current {
                    Some(value) if show_values => value,
                    Some(_) => "••••••••",
                    None => language.text("Unavailable in current process", "当前进程中不可用"),
                };
                column![
                    text(format!(
                        "{}: {}",
                        language.text("Configuration expression", "配置表达式"),
                        value
                    ))
                    .size(14)
                    .width(Length::Fill)
                    .wrapping(Wrapping::WordOrGlyph),
                    text(format!(
                        "{}: {}",
                        language.text("Current process value", "当前进程值"),
                        current_value
                    ))
                    .size(14)
                    .width(Length::Fill)
                    .wrapping(Wrapping::WordOrGlyph)
                ]
                .spacing(6)
                .width(Length::FillPortion(3))
                .into()
            } else if page == Page::User && entry.toggleable && show_values && editing.is_none() {
                if let Some(index) = index {
                    mouse_area(
                        container(
                            text(value)
                                .size(14)
                                .width(Length::Fill)
                                .wrapping(Wrapping::WordOrGlyph),
                        )
                        .width(Length::FillPortion(3))
                        .padding([8, 0]),
                    )
                    .on_double_click(Message::BeginEdit(index))
                    .into()
                } else {
                    text(value)
                        .size(14)
                        .width(Length::FillPortion(3))
                        .wrapping(Wrapping::WordOrGlyph)
                        .into()
                }
            } else {
                text(value)
                    .size(14)
                    .width(Length::FillPortion(3))
                    .wrapping(Wrapping::WordOrGlyph)
                    .into()
            };
        let mut action: Element<'_, Message> = Space::new().width(88).into();
        if is_editing {
            action = container(
                button(language.text("Save", "保存"))
                    .on_press(Message::SaveEdit)
                    .style(theme::soft_button)
                    .padding(12),
            )
            .width(88)
            .into();
        } else if page == Page::User
            && entry.toggleable
            && editing.is_none()
            && let Some(index) = index
        {
            action = container(
                button(if entry.enabled {
                    language.text("Disable", "禁用")
                } else {
                    language.text("Enable", "启用")
                })
                .on_press(Message::ToggleDeclaration(index))
                .style(move |theme, status| theme::toggle_button(theme, status, entry.enabled))
                .padding(12),
            )
            .width(88)
            .into();
        }
        let mut name_column = column![
            text(&entry.name)
                .size(15)
                .width(Length::Fill)
                .wrapping(Wrapping::WordOrGlyph),
            text(&entry.source)
                .size(12)
                .width(Length::Fill)
                .wrapping(Wrapping::WordOrGlyph)
        ]
        .spacing(5);
        if !is_editing
            && let Some(reason) = edit_reason(page, entry, editing.is_some(), show_values, language)
        {
            name_column = name_column.push(
                text(reason)
                    .size(12)
                    .color(iced::color!(0x9baebe))
                    .width(Length::Fill)
                    .wrapping(Wrapping::WordOrGlyph),
            );
        }
        let declaration_row = row![
            container(name_column)
                .width(Length::FillPortion(2))
                .clip(true),
            container(value_column)
                .width(Length::FillPortion(3))
                .clip(true),
            action,
        ]
        .spacing(20);
        rows = rows.push(
            container(declaration_row)
                .width(Length::Fill)
                .padding([14, 18])
                .style(theme::glass_row),
        );
    }
    // 进程快照不携带来源信息，单独列出以免错误划分为系统或用户级。
    if page == Page::User {
        rows = rows.push(
            container(
                text(language.text("Current process · origin unknown", "当前进程 · 来源未判定"))
                    .size(16),
            )
            .padding([16, 18]),
        );
        for entry in environment::filter(&process.entries, query) {
            let value = if show_values {
                entry.value.as_str()
            } else {
                "••••••••"
            };
            rows = rows.push(
                container(
                    row![
                        container(
                            column![
                                text(&entry.name)
                                    .size(15)
                                    .width(Length::Fill)
                                    .wrapping(Wrapping::WordOrGlyph),
                                text(language.text(
                                    "Process value; editable source unknown",
                                    "进程变量，未找到可编辑来源"
                                ))
                                .size(12)
                                .color(iced::color!(0x9baebe))
                                .width(Length::Fill)
                                .wrapping(Wrapping::WordOrGlyph)
                            ]
                            .spacing(5)
                        )
                        .width(Length::FillPortion(2))
                        .clip(true),
                        container(
                            text(value)
                                .size(14)
                                .width(Length::Fill)
                                .wrapping(Wrapping::WordOrGlyph)
                        )
                        .width(Length::FillPortion(3))
                        .clip(true),
                        Space::new().width(88)
                    ]
                    .spacing(20),
                )
                .width(Length::Fill)
                .padding([14, 18])
                .style(theme::glass_row),
            );
        }
    }
    if count == 0
        && (page == Page::System || environment::filter(&process.entries, query).is_empty())
    {
        rows = rows.push(
            container(
                text(language.text("No variables match your search.", "没有匹配的环境变量。"))
                    .size(17),
            )
            .padding(24),
        );
    }
    let add_form: Element<'_, Message> = if page == Page::User && app.adding {
        column![
            text(language.text("Add environment variable", "新增环境变量")).size(18),
            row![
                text_input(
                    language.text("Name (e.g. JAVA_HOME)", "变量名（例如 JAVA_HOME）"),
                    &app.add_name
                )
                .on_input(Message::AddName)
                .style(theme::soft_input)
                .padding(12)
                .width(Length::Fill),
                text_input(language.text("Value", "变量值"), &app.add_value)
                    .on_input(Message::AddValue)
                    .style(theme::soft_input)
                    .padding(12)
                    .width(Length::Fill)
            ]
            .spacing(12),
            text(language.text(
                "Saved to ~/.zshenv; takes effect in new zsh sessions that load it.",
                "保存至 ~/.zshenv，在之后加载该文件的 zsh 会话中生效。 "
            ))
            .size(13),
            row![
                button(language.text("Save", "保存"))
                    .on_press(Message::SaveAdd)
                    .style(theme::soft_button)
                    .padding(12),
                button(language.text("Cancel", "取消"))
                    .on_press(Message::CancelAdd)
                    .style(theme::soft_button)
                    .padding(12)
            ]
            .spacing(12)
        ]
        .spacing(12)
        .into()
    } else if page == Page::User {
        button(language.text("Add variable", "新增环境变量"))
            .on_press_maybe(if app.editing.is_none() {
                Some(Message::BeginAdd)
            } else {
                None
            })
            .style(theme::soft_button)
            .padding(12)
            .into()
    } else {
        Space::new().height(0).into()
    };
    column![
        text(if page == Page::System {
            language.text("System", "系统")
        } else {
            language.text("User", "用户")
        })
        .size(30),
        text(language.text(
            "User declarations can be enabled or disabled for new shells loading the file",
            "用户声明可启用或禁用，对之后加载配置的新 Shell 生效"
        ))
        .size(17),
        add_form,
        text(error.unwrap_or("")).size(14),
        text(if snapshot.unreadable.is_empty() {
            String::new()
        } else {
            format!(
                "{}: {}",
                language.text("Unable to read", "无法读取"),
                snapshot.unreadable.join(", ")
            )
        })
        .size(13),
        row![
            text_input(
                language.text("Search variable names", "搜索变量名称"),
                query
            )
            .on_input(Message::Search)
            .style(theme::soft_input)
            .padding(12)
            .width(Length::Fill),
            button(if show_values {
                language.text("Hide values", "隐藏值")
            } else {
                language.text("Show values", "显示值")
            })
            .on_press(Message::ToggleValues)
            .style(theme::soft_button)
            .padding(12),
            button(language.text("Refresh", "刷新"))
                .on_press(Message::Refresh)
                .style(theme::soft_button)
                .padding(12)
        ]
        .spacing(12),
        text(if language == Language::Chinese {
            format!("配置声明 {} 条，匹配 {count} 条", entries.len())
        } else {
            format!("{count} of {} declarations", entries.len())
        })
        .size(14),
        row![
            text(language.text("NAME", "名称"))
                .size(12)
                .width(Length::FillPortion(2)),
            text(language.text("VALUE", "值"))
                .size(12)
                .width(Length::FillPortion(3)),
            container(text(language.text("ACTION", "操作")).size(12)).width(88)
        ]
        .spacing(20)
        .padding([0, 18]),
        scrollable(rows).height(Length::Fill)
    ]
    .spacing(17)
    .into()
}
