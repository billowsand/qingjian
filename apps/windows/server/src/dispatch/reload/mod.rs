//! 配置热加载：空闲时看 `config.toml` 的 mtime，改了就重读并应用（与 macOS 壳对齐）。
//! 便宜的设置无条件重设；云联想 / 附加词库只在对应项变了才重建。热加载状态在 [`ConfigReload`]。

mod state;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use qingjian_core::{Engine, FumaScheme, FumaTable, NoGlossFiller, NoPredictor};
use qingjian_platform::{Config, extra_dictionaries};
use qingjian_predict::{CloudGlossFiller, CloudPredictor, PredictConfig};

pub(super) use self::state::ConfigReload;

/// 看配置文件 mtime 的最短间隔；工人循环空闲时按它等，重排的短节拍来得更勤时按这个节流。
pub(super) const CONFIG_POLL_INTERVAL: Duration = Duration::from_secs(1);
use super::{Router, RouterConfig};
use crate::assembly::user_dicts_dir;

fn mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}

/// 随包辅码表（`assets/fuma/<方案>.txt`）；读不到就当辅码关着。启动与热加载共用。
pub fn load_fuma(root: &Path, scheme: FumaScheme) -> Option<Arc<FumaTable>> {
    let path = root.join("assets").join(scheme.asset());
    match FumaTable::from_path(&path) {
        Ok(table) => {
            tracing::info!(scheme = scheme.key(), words = table.len(), "辅码表已加载");
            Some(Arc::new(table))
        }
        Err(error) => {
            tracing::error!(scheme = scheme.key(), %error, path = %path.display(), "辅码表加载失败，辅码关");
            None
        }
    }
}

/// 按 `[predict]` 接云联想与释义兜底；关着或缺密钥就退回本地实现。启动与热加载共用。
pub fn attach_cloud(engine: &mut Engine, predict: &PredictConfig) {
    if !predict.enabled {
        tracing::info!("云联想未开启（[predict] enabled = false）");
        engine.set_predictor(Box::new(NoPredictor));
        engine.set_gloss_filler(Box::new(NoGlossFiller));
        return;
    }
    match CloudPredictor::new(predict) {
        Ok(predictor) => {
            engine.set_predictor(Box::new(predictor));
            tracing::info!(model = %predict.model, "云联想已接入");
        }
        Err(error) => {
            tracing::warn!(%error, "云联想接入失败（缺 API key？），退回本地候选");
            engine.set_predictor(Box::new(NoPredictor));
        }
    }
    match CloudGlossFiller::new(predict) {
        Ok(filler) => engine.set_gloss_filler(Box::new(filler)),
        Err(error) => {
            tracing::warn!(%error, "释义兜底未启用");
            engine.set_gloss_filler(Box::new(NoGlossFiller));
        }
    }
}

impl Router {
    /// `config.toml` 路径；没开热加载（测试）时为 `None`。
    pub(super) fn config_path(&self) -> Option<&Path> {
        self.reload
            .as_ref()
            .map(|reload| reload.config_path.as_path())
    }

    /// 开启热加载：记下路径与当前已应用的 predict / dictionaries。
    pub fn watch_config(
        &mut self,
        config: &Config,
        config_path: PathBuf,
        root: PathBuf,
        user_dir: Option<PathBuf>,
    ) {
        let last_mtime = mtime(&config_path);
        let bundled_dicts_dir = Some(root.join("data/generated/dicts")).filter(|dir| dir.is_dir());
        self.reload = Some(ConfigReload {
            config_path,
            last_check: Instant::now(),
            root,
            bundled_dicts_dir,
            user_dir,
            last_mtime,
            applied_predict: config.predict.clone(),
            applied_dictionaries: config.dictionaries.clone(),
        });
    }

    /// 空闲时调；一秒内只真正看一次文件。解析失败保持原配置，mtime 照记（不每秒重试同一个坏文件）。
    pub fn poll_config_reload(&mut self) {
        let Some(reload) = &mut self.reload else {
            return;
        };
        if reload.last_check.elapsed() < CONFIG_POLL_INTERVAL {
            return;
        }
        reload.last_check = Instant::now();
        let current = mtime(&reload.config_path);
        if current == reload.last_mtime {
            return;
        }
        reload.last_mtime = current;
        let path = reload.config_path.clone();
        match Config::load(&path) {
            Ok(config) => {
                self.apply_config(&config);
                tracing::info!("配置已热加载");
            }
            Err(error) => tracing::error!(%error, "配置热加载解析失败，保持原配置"),
        }
    }

    /// 辅码按新配置重接。启动时辅码是关的就没读过表，用户在设置里刚打开时现读一次，
    /// 否则开关只在重启后才生效。
    fn apply_fuma(&mut self, config: &Config) {
        let Some(scheme) = config.general.fuma() else {
            self.engine.set_fuma(None);
            return;
        };
        if self.fuma_table.is_none()
            && let Some(root) = self.reload.as_ref().map(|reload| reload.root.clone())
        {
            self.fuma_table = load_fuma(&root, scheme);
        }
        self.engine.set_fuma(self.fuma_table.clone());
    }

    /// 应用新配置。
    fn apply_config(&mut self, config: &Config) {
        self.engine.set_fuzzy(config.fuzzy);
        self.engine.set_shuangpin(config.general.shuangpin());
        self.apply_fuma(config);
        self.engine.set_zhuyin_mode(config.general.zhuyin);
        self.engine.set_learning(config.general.learning);
        self.engine.set_mode_keys(config.shortcut.mode);
        self.engine.set_chinese_first(config.general.chinese_first);
        let previous = self.config.render_settings();
        self.config = RouterConfig::from(config);
        let settings = self.config.render_settings();
        if settings != previous {
            self.candidates.configure(settings);
        }
        self.reconcile_status();
        self.apply_model_config(&config.model);

        let Some(reload) = &mut self.reload else {
            return;
        };
        if config.predict != reload.applied_predict {
            attach_cloud(&mut self.engine, &config.predict);
            reload.applied_predict = config.predict.clone();
        }
        if config.dictionaries != reload.applied_dictionaries {
            // 别传用户目录本身：那里的学习数据 .tsv 会被当词库装。
            let dicts = extra_dictionaries::load(
                reload.bundled_dicts_dir.as_deref(),
                user_dicts_dir(reload.user_dir.as_deref()).as_deref(),
                &config.dictionaries,
            );
            tracing::info!(count = dicts.len(), "附加词库已热重装");
            self.engine.set_extra_dictionaries(dicts);
            reload.applied_dictionaries = config.dictionaries.clone();
        }
    }
}
