//! 装配 Engine：Server 里唯一知道具体 Learner 类型的地方。
//! Windows 壳不带译文功能（不装释义表、不记词汇），所以这里没有 Translator，与 macOS 的 `host::init` 不同。

mod language_model;
mod spec;

use std::path::Path;
use std::time::Instant;

use qingjian_core::{EmojiTable, Engine};
use qingjian_dictionary::{Dictionary, WordList};
use qingjian_learning::{FrequencyLearner, InputLog, UsageStats};
use qingjian_platform::extra_dictionaries;

use crate::error::ServerError;

pub use self::language_model::LanguageModelFiles;
pub use self::spec::AssemblySpec;

pub fn assemble(spec: &AssemblySpec) -> Result<Engine, ServerError> {
    let started = Instant::now();
    let dictionary = Dictionary::from_path(&spec.dict)?;
    let learner = match &spec.user_dir {
        Some(dir) => load_learner(dir),
        None => FrequencyLearner::default(),
    };
    tracing::info!(
        entries = dictionary.len(),
        learned = learner.len(),
        dictionary_ms = started.elapsed().as_millis(),
        "词库与学习数据已加载"
    );
    let mut engine = Engine::new(dictionary).with_learner(Box::new(learner));
    if let Some(dir) = &spec.user_dir {
        engine = engine.with_usage_meter(Box::new(UsageStats::open(dir.join("usage.tsv"))));
        if spec.input_log {
            let path = dir.join("input-log.jsonl");
            tracing::info!(path = %path.display(), "输入日志开着");
            engine = engine.with_input_logger(Box::new(InputLog::open(path)));
        }
    }
    engine.set_extra_dictionaries(extra_dictionaries::load(
        spec.bundled_dicts_dir.as_deref(),
        user_dicts_dir(spec.user_dir.as_deref()).as_deref(),
        &spec.dictionaries,
    ));
    if let Some(path) = &spec.english {
        let words = WordList::from_path(path)?;
        tracing::info!(words = words.len(), "英文词表已加载");
        engine = engine.with_english(words);
    }
    if let Some(table) = load_emoji(&spec.emoji) {
        tracing::info!(words = table.len(), "emoji 表已加载");
        engine = engine.with_emoji(table);
    }
    if let Some(files) = &spec.language_model {
        let started = Instant::now();
        let model = files.load()?;
        tracing::info!(
            words = model.word_count(),
            bigrams = model.bigram_count(),
            load_ms = started.elapsed().as_millis(),
            "语言模型已加载"
        );
        engine = engine.with_language_model(Box::new(model));
    }
    Ok(engine)
}

/// 用户导入词库目录 `dicts/`，不存在则创建；建不了当没有。
pub(crate) fn user_dicts_dir(user_dir: Option<&Path>) -> Option<std::path::PathBuf> {
    let dir = user_dir?.join("dicts");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// 读不了就退回只在内存里学，不拿空表覆盖用户文件。
fn load_learner(dir: &Path) -> FrequencyLearner {
    let path = dir.join("user.tsv");
    match FrequencyLearner::from_path(&path) {
        Ok(learner) => learner,
        Err(error) => {
            tracing::error!(path = %path.display(), %error, "学习数据读取失败，本次只在内存里学习");
            FrequencyLearner::default()
        }
    }
}

/// 几张 emoji 表合成一张；坏的跳过。
fn load_emoji(paths: &[std::path::PathBuf]) -> Option<EmojiTable> {
    let mut merged: Option<EmojiTable> = None;
    for path in paths {
        match EmojiTable::from_path(path) {
            Ok(table) => match &mut merged {
                Some(all) => all.merge(table),
                None => merged = Some(table),
            },
            Err(error) => tracing::warn!(%error, path = %path.display(), "emoji 表加载失败，跳过"),
        }
    }
    merged
}
