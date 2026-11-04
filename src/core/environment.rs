use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub value: String,
    pub source: String,
    pub dynamic: bool,
    pub line_index: usize,
    pub original_line: String,
    pub enabled: bool,
    pub toggleable: bool,
}
#[derive(Debug, Clone, Copy)]
pub enum Scope {
    System,
    User,
}
#[derive(Debug, Default)]
pub struct Snapshot {
    pub entries: Vec<Entry>,
    pub unreadable: Vec<String>,
}

/// 读取当前进程的完整环境快照；来源无法从进程环境中恢复。
pub fn process_snapshot() -> Snapshot {
    let mut entries: Vec<Entry> = std::env::vars_os()
        .map(|(name, value)| Entry {
            name: name.to_string_lossy().into_owned(),
            value: value.to_string_lossy().into_owned(),
            source: String::new(),
            dynamic: false,
            line_index: 0,
            original_line: String::new(),
            enabled: true,
            toggleable: false,
        })
        .collect();
    entries.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.name.cmp(&b.name))
    });
    Snapshot {
        entries,
        unreadable: Vec::new(),
    }
}

/// 仅读取约定的 Shell 配置文件；缺失文件正常，读取失败单独报告。
pub fn snapshot(scope: Scope) -> Snapshot {
    let paths: Vec<PathBuf> = match scope {
        Scope::System => [
            "/etc/zshenv",
            "/etc/zprofile",
            "/etc/zshrc",
            "/etc/zlogin",
            "/etc/profile",
            "/etc/bashrc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect(),
        Scope::User => match dirs::home_dir() {
            Some(home) => [
                ".zshenv",
                ".zprofile",
                ".zshrc",
                ".zlogin",
                ".bash_profile",
                ".bash_login",
                ".profile",
                ".bashrc",
            ]
            .into_iter()
            .map(|name| home.join(name))
            .collect(),
            None => {
                return Snapshot {
                    entries: vec![],
                    unreadable: vec!["Home directory unavailable".into()],
                };
            }
        },
    };
    let mut result = Snapshot::default();
    for path in paths {
        match fs::read_to_string(&path) {
            Ok(content) => result
                .entries
                .extend(parse_source(&content, &path.to_string_lossy())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => result.unreadable.push(path.to_string_lossy().into_owned()),
        }
    }
    result.entries.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then(a.source.cmp(&b.source))
    });
    result
}

/// 仅识别独立的 export NAME=VALUE 声明，不执行脚本或推测实际生效顺序。
pub fn parse_source(content: &str, source: &str) -> Vec<Entry> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_index, original)| {
            let line = original.trim();
            let (line, enabled) = match line.strip_prefix("# envpilot-disabled: ") {
                Some(value) => (value, false),
                None => (line, true),
            };
            let rest = line.strip_prefix("export ")?.trim_start();
            let (name, raw) = rest.split_once('=')?;
            if name.is_empty()
                || !name.bytes().enumerate().all(|(i, c)| {
                    c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit())
                })
            {
                return None;
            }
            let raw = raw.trim();
            let single = raw.starts_with('\'')
                && raw.ends_with('\'')
                && raw.len() >= 2
                && !raw[1..raw.len() - 1].contains('\'');
            let double = raw.starts_with('"')
                && raw.ends_with('"')
                && raw.len() >= 2
                && !raw[1..raw.len() - 1].contains('"');
            // 只有可确定的简单字面量才作为值展示；其他语法统一视为动态表达式。
            let inner = if single || double {
                &raw[1..raw.len() - 1]
            } else {
                raw
            };
            let dynamic = if single {
                false
            } else if double {
                inner.contains(['$', '`', '\\'])
            } else {
                inner.contains(|c: char| {
                    c.is_whitespace()
                        || matches!(
                            c,
                            '$' | '`'
                                | '\\'
                                | ';'
                                | '|'
                                | '&'
                                | '#'
                                | '"'
                                | '\''
                                | '('
                                | ')'
                                | '<'
                                | '>'
                                | '*'
                                | '?'
                                | '['
                                | ']'
                                | '{'
                                | '}'
                                | '~'
                        )
                })
            };
            Some(Entry {
                name: name.to_owned(),
                value: if dynamic {
                    raw.to_owned()
                } else {
                    inner.to_owned()
                },
                source: source.to_owned(),
                dynamic,
                line_index,
                original_line: original.to_owned(),
                enabled,
                // 仅允许未缩进、简单字面量的独立声明，复杂 Shell 控制结构不做推测。
                toggleable: !dynamic
                    && (original.starts_with("export ")
                        || original.starts_with("# envpilot-disabled: export ")),
            })
        })
        .collect()
}
/// 校验变量名和值，生成追加后的配置文本；不执行任何 Shell 语句。
pub fn append_declaration(content: &str, name: &str, value: &str) -> anyhow::Result<String> {
    if name.is_empty()
        || !name
            .bytes()
            .enumerate()
            .all(|(i, c)| c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
    {
        anyhow::bail!("变量名须以字母或下划线开头，只能包含字母、数字和下划线");
    }
    if value.contains(['\'', '\n', '\r', '\0']) {
        anyhow::bail!("值不能包含单引号、换行或空字符");
    }
    if parse_source(content, "")
        .iter()
        .any(|entry| entry.name == name)
    {
        anyhow::bail!("该变量已在配置中声明");
    }
    let newline = if !content.is_empty() && !content.ends_with('\n') {
        "\n"
    } else {
        ""
    };
    Ok(format!("{content}{newline}export {name}='{value}'\n"))
}

/// 将新增声明追加到当前用户的 zsh 启动文件，保留文件已有内容及属性。
pub fn add_user_variable(name: &str, value: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法定位用户目录"))?;
    let path = home.join(".zshenv");
    let existing = snapshot(Scope::User);
    if !existing.unreadable.is_empty() {
        anyhow::bail!("部分用户配置无法读取，无法检查重名变量");
    }
    if existing.entries.iter().any(|entry| entry.name == name) {
        anyhow::bail!("用户配置中已存在同名变量");
    }
    let metadata = match fs::symlink_metadata(&path) {
        Ok(meta) if meta.file_type().is_file() => Some(meta),
        Ok(_) => anyhow::bail!(".zshenv 不是普通文件，拒绝修改"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let original = if metadata.is_some() {
        fs::read_to_string(&path)?
    } else {
        String::new()
    };
    let updated = append_declaration(&original, name, value)?;
    // 拒绝符号链接并检测文件替换；新建文件仅允许用户读写。
    let mut options = fs::OpenOptions::new();
    options.read(true).append(true);
    if metadata.is_none() {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(&path)?;
    #[cfg(unix)]
    if let Some(expected) = &metadata {
        use std::os::unix::fs::MetadataExt;
        let actual = file.metadata()?;
        if expected.dev() != actual.dev() || expected.ino() != actual.ino() {
            anyhow::bail!("文件已更改，请刷新后重试");
        }
    }
    use std::io::Read;
    let mut current = String::new();
    file.read_to_string(&mut current)?;
    if current != original {
        anyhow::bail!("文件已更改，请刷新后重试");
    }
    file.write_all(&updated.as_bytes()[original.len()..])?;
    file.sync_all()?;
    Ok(())
}

/// 根据精确行号和原文生成可逆修改；保留原有换行符并拒绝不支持的声明。
pub fn toggle_line(
    content: &str,
    index: usize,
    expected: &str,
    disable: bool,
) -> anyhow::Result<String> {
    let mut lines: Vec<&str> = content.split_inclusive('\n').collect();
    let original = *lines
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("文件已更改，请刷新"))?;
    let body = original.strip_suffix('\n').unwrap_or(original);
    let body = body.strip_suffix('\r').unwrap_or(body);
    if body != expected {
        anyhow::bail!("文件已更改，请刷新");
    }
    let parsed = parse_source(body, "");
    let entry = parsed
        .first()
        .ok_or_else(|| anyhow::anyhow!("该声明不可切换"))?;
    if !entry.toggleable || entry.enabled != disable {
        anyhow::bail!("该声明不可切换");
    }
    let replacement = if disable {
        format!("# envpilot-disabled: {body}")
    } else {
        body.strip_prefix("# envpilot-disabled: ")
            .ok_or_else(|| anyhow::anyhow!("无效标记"))?
            .to_owned()
    };
    let ending = &original[body.len()..];
    let updated = format!("{replacement}{ending}");
    lines[index] = &updated;
    Ok(lines.concat())
}

/// 用安全的单引号字面量替换原值，保持启用状态和换行格式。
pub fn edit_line(
    content: &str,
    index: usize,
    expected: &str,
    value: &str,
) -> anyhow::Result<String> {
    if value.contains(['\'', '\n', '\r', '\0']) {
        anyhow::bail!("值不能包含单引号、换行或空字符");
    }
    let mut lines: Vec<&str> = content.split_inclusive('\n').collect();
    let original = *lines
        .get(index)
        .ok_or_else(|| anyhow::anyhow!("文件已更改，请刷新"))?;
    let body = original.strip_suffix('\n').unwrap_or(original);
    let body = body.strip_suffix('\r').unwrap_or(body);
    if body != expected {
        anyhow::bail!("文件已更改，请刷新");
    }
    let entry = parse_source(body, "")
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("该声明不可编辑"))?;
    if !entry.toggleable {
        anyhow::bail!("该声明不可编辑");
    }
    let prefix = if entry.enabled {
        ""
    } else {
        "# envpilot-disabled: "
    };
    let updated = format!(
        "{prefix}export {}='{}'{}",
        entry.name,
        value,
        &original[body.len()..]
    );
    lines[index] = &updated;
    Ok(lines.concat())
}

/// 保存用户声明的字面量值，不改变该声明的启用状态。
pub fn edit(entry: &Entry, value: &str) -> anyhow::Result<()> {
    write_entry(entry, |content| {
        edit_line(content, entry.line_index, &entry.original_line, value)
    })
}

/// 仅修改当前用户目录下支持的普通配置文件，原子替换并保留权限。
pub fn toggle(entry: &Entry) -> anyhow::Result<()> {
    write_entry(entry, |content| {
        toggle_line(
            content,
            entry.line_index,
            &entry.original_line,
            entry.enabled,
        )
    })
}

/// 以同目录临时文件保存经校验的变更，避免覆盖读取期间发生的修改。
fn write_entry(
    entry: &Entry,
    update: impl FnOnce(&str) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法定位用户目录"))?;
    let path = PathBuf::from(&entry.source);
    let allowed = [
        ".zshenv",
        ".zprofile",
        ".zshrc",
        ".zlogin",
        ".bash_profile",
        ".bash_login",
        ".profile",
        ".bashrc",
    ];
    if !allowed.iter().any(|name| path == home.join(name)) || !entry.toggleable {
        anyhow::bail!("该声明不可切换");
    }
    let meta = fs::symlink_metadata(&path)?;
    if !meta.file_type().is_file() {
        anyhow::bail!("配置文件不是普通文件，拒绝修改");
    }
    let original = fs::read_to_string(&path)?;
    let updated = update(&original)?;
    let temp_path = home.join(format!(
        ".envpilot-{}-{}.tmp",
        std::process::id(),
        entry.line_index
    ));
    let mut temporary = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)?;
    let result = (|| -> anyhow::Result<()> {
        temporary.set_permissions(meta.permissions())?;
        use std::io::Write;
        temporary.write_all(updated.as_bytes())?;
        temporary.sync_all()?;
        // 避免覆盖读取之后已被用户或编辑器修改的配置。
        if fs::read_to_string(&path)? != original {
            anyhow::bail!("文件已更改，请刷新");
        }
        fs::rename(&temp_path, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result?;
    Ok(())
}

/// 按区分大小写的变量名关联进程快照，避免误匹配不同变量。
pub fn process_value<'a>(entries: &'a [Entry], name: &str) -> Option<&'a str> {
    entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.value.as_str())
}
/// 仅按变量名搜索，避免通过搜索结果泄露被隐藏的值。
pub fn filter<'a>(entries: &'a [Entry], query: &str) -> Vec<&'a Entry> {
    let query = query.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| entry.name.to_lowercase().contains(&query))
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_direct_exports_and_marks_dynamic_expressions() {
        let result = parse_source(
            "export FOO='hello world'\nexport PATH=\"$HOME/bin:$PATH\"\n# export NO=bad\necho x; export NOPE=bad\n",
            "/etc/zshenv",
        );
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].value, "hello world");
        assert!(!result[0].dynamic);
        assert!(result[1].dynamic);
        assert_eq!(result[1].value, "\"$HOME/bin:$PATH\"");
        assert_eq!(result[1].source, "/etc/zshenv");
    }
    #[test]
    fn finds_process_value_by_exact_name() {
        let entries = vec![Entry {
            name: "JAVA_HOME".into(),
            value: "/java".into(),
            source: String::new(),
            dynamic: false,
            line_index: 0,
            original_line: String::new(),
            enabled: true,
            toggleable: false,
        }];
        assert_eq!(process_value(&entries, "JAVA_HOME"), Some("/java"));
        assert_eq!(process_value(&entries, "java_home"), None);
    }
    #[test]
    fn toggle_preserves_content_and_rejects_stale_lines() {
        let input = "export FOO=bar\r\nexport OTHER=baz\r\n";
        let disabled = toggle_line(input, 0, "export FOO=bar", true).expect("disable");
        assert_eq!(
            disabled,
            "# envpilot-disabled: export FOO=bar\r\nexport OTHER=baz\r\n"
        );
        assert_eq!(
            toggle_line(&disabled, 0, "# envpilot-disabled: export FOO=bar", false).unwrap(),
            input
        );
        assert!(toggle_line(input, 0, "export FOO=changed", true).is_err());
        assert!(
            toggle_line(
                "export FOO=bar; echo danger\n",
                0,
                "export FOO=bar; echo danger",
                true
            )
            .is_err()
        );
    }
    #[test]
    fn editing_preserves_enabled_state_and_rejects_unsafe_values() {
        assert_eq!(
            edit_line("export FOO=old\n", 0, "export FOO=old", "new value").unwrap(),
            "export FOO='new value'\n"
        );
        assert_eq!(
            edit_line(
                "# envpilot-disabled: export FOO=old\n",
                0,
                "# envpilot-disabled: export FOO=old",
                "new"
            )
            .unwrap(),
            "# envpilot-disabled: export FOO='new'\n"
        );
        assert!(edit_line("export FOO=old\n", 0, "export FOO=old", "bad'quote").is_err());
        assert!(edit_line("export FOO=old\n", 0, "export FOO=changed", "new").is_err());
    }
    #[test]
    fn new_declaration_validates_and_preserves_existing_text() {
        assert_eq!(
            append_declaration("# config", "FOO_1", "hello world").unwrap(),
            "# config\nexport FOO_1='hello world'\n"
        );
        assert!(append_declaration("export FOO='old'\n", "FOO", "new").is_err());
        assert!(append_declaration("", "1BAD", "x").is_err());
        assert!(append_declaration("", "FOO", "x' ; bad").is_err());
    }
    #[test]
    fn filters_only_by_name() {
        let entries = parse_source("export ALPHA=secret\n", "test");
        assert_eq!(filter(&entries, "alp").len(), 1);
        assert!(filter(&entries, "secret").is_empty());
    }
}
