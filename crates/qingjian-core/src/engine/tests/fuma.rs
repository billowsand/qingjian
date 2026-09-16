//! 辅码（双拼辅助码）：触发、严格过滤、键消耗与学习。
//!
//! 样例词库只有 开发 / 开发者 / 开饭 / 开放 / 开 等词，表用它们：开 = `fk`、发 = `xa`。
//! 「开发」的双码期望是（开第 1 码 f, 发第 1 码 x）；单字「开」的期望是它自己的两码 (f, k)。

use super::*;

use crate::FumaTable;

fn fuma_table() -> Arc<FumaTable> {
    Arc::new(FumaTable::parse("开=fk\n发=xa\n").unwrap())
}

fn fuma_engine() -> Engine {
    let mut engine = xiaohe();
    engine.set_fuma(Some(fuma_table()));
    engine
}

#[test]
fn fuma_filters_candidates_strictly() {
    let mut engine = fuma_engine();
    // 末 2 键含大写才激活：`fX` → (f, x)，正是（开第 1 码, 发第 1 码）
    engine.set_input("kdfafX");
    let items = &engine.query().unwrap().candidates.items;
    assert_eq!(items[0].text, "开发");
    // 对不上一个候选都不剩
    engine.set_input("kdfafY");
    assert!(engine.query().unwrap().candidates.items.is_empty());
    // 第一键大写是反转顺序：`Xf` → 交换成 (f, x)，同样对上
    engine.set_input("kdfaXf");
    assert_eq!(engine.query().unwrap().candidates.items[0].text, "开发");
    // 全小写不激活：末 2 键仍是双拼，前缀候选照常在（开 也在）
    engine.set_input("kdfafa");
    let items = &engine.query().unwrap().candidates.items;
    assert!(items.iter().any(|c| c.text == "开"));
    assert!(items.iter().any(|c| c.text == "开发"));
}

#[test]
fn fuma_filters_single_char_by_both_codes() {
    let mut engine = fuma_engine();
    // 单字期望本字两码：开 = (f, k)
    engine.set_input("kdfK");
    let items = &engine.query().unwrap().candidates.items;
    assert!(!items.is_empty());
    assert!(items.iter().all(|c| c.text == "开"));
    // 反转输入：`Kf` → 交换成 (f, k)
    engine.set_input("kdKf");
    assert!(
        engine
            .query()
            .unwrap()
            .candidates
            .items
            .iter()
            .all(|c| c.text == "开")
    );
    // 对不上就严格不剩
    engine.set_input("kdXk");
    assert!(engine.query().unwrap().candidates.items.is_empty());
    // 单键大写不激活（3 键是奇数）：末键按声母继续
    engine.set_input("kdN");
    assert_eq!(engine.query().unwrap().marked_text(), "kai'n");
}

#[test]
fn fuma_needs_a_complete_shuangpin_prefix() {
    let mut engine = fuma_engine();
    // 前缀解不完整（`nih` 落单 h）：末尾大写字母按普通拼音处理
    engine.set_input("nihKz");
    assert_eq!(engine.query().unwrap().marked_text(), "ni'huai'z");
    // 不足 4 键也不激活
    engine.set_input("nzK");
    let query = engine.query().unwrap();
    assert_eq!(query.marked_text(), "nou'k");
}

#[test]
fn fuma_commit_consumes_the_codes_too() {
    let mut engine = fuma_engine();
    engine.set_input("kdfafX");
    let query = engine.query().unwrap();
    let candidate = query.candidates.items[0].clone();
    assert_eq!(candidate.text, "开发");
    engine.commit(&candidate);
    // 6 个键全部吃掉，缓冲区清空
    assert!(engine.composition().is_empty());
    // 学习键不含辅码：记的是 kaifa
    assert!(engine.recent_commits.last().unwrap().same_input("kaifa"));

    // 没激活辅码时部分消耗照旧
    engine.set_input("kdfa");
    let query = engine.query().unwrap();
    let kai = query
        .candidates
        .items
        .iter()
        .find(|c| c.text == "开")
        .cloned()
        .unwrap();
    engine.commit(&kai);
    assert_eq!(engine.composition().text(), "fa");
}

#[test]
fn fuma_codes_go_away_with_a_prefix_candidate() {
    let mut engine = fuma_engine();
    // `fK` → (f, k)，正是单字「开」的两码；「开」只盖住前一个音节 kai
    engine.set_input("kdfafK");
    let items = engine.query().unwrap().candidates.items;
    assert!(items.iter().all(|c| c.text == "开"));
    engine.commit(&items[0]);
    // 辅码键随这次上屏一起丢掉：剩下的拼音不再背着它，否则 `fa` 又被筛一遍
    assert_eq!(engine.composition().text(), "fa");
}

#[test]
fn fuma_shows_up_in_the_pinyin_line() {
    let mut engine = fuma_engine();
    // 激活时辅码段单独一段跟在拼音后面，原样保留大小写：敲的是 `fX` 就显示 `fX`
    engine.set_input("kdfafX");
    let query = engine.query().unwrap();
    assert_eq!(query.marked_text(), "kai'fa fX");
    let segments = query.marked_segments();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind, MarkedKind::Typed);
    assert_eq!(segments[1].kind, MarkedKind::Fuma);
    assert_eq!(segments[1].text, " fX");
    // 光标算在辅码段之后（用户正在末尾敲）
    assert_eq!(query.marked_cursor(), "kai'fa fX".chars().count());

    // 反转写法照样原样显示
    engine.set_input("kdfaXf");
    assert_eq!(engine.query().unwrap().marked_text(), "kai'fa Xf");

    // 没激活时拼音行不变，一个字都不多
    engine.set_input("kdfafa");
    let query = engine.query().unwrap();
    assert_eq!(query.marked_text(), "kai'fa'fa");
    assert!(query.fuma.is_none());
    assert!(
        query
            .marked_segments()
            .iter()
            .all(|s| s.kind != MarkedKind::Fuma)
    );
}

#[test]
fn fuma_stays_out_of_zhuyin() {
    let mut engine = fuma_engine();
    engine.set_zhuyin_mode(true);
    // 注音走自己的解码，末 2 键不是双拼键：辅码整套不介入
    assert!(!engine.fuma_enabled());
    engine.set_input("kdfafX");
    assert!(!engine.query().unwrap().candidates.items.is_empty());
}

#[test]
fn fuma_raw_commit_drops_the_codes() {
    let mut engine = fuma_engine();
    engine.set_input("kdfafY");
    assert!(engine.query().unwrap().candidates.items.is_empty());
    // 回车上屏字母本身：辅码段不是要打的内容
    assert_eq!(engine.take_raw(), "kdfa");
    // 全小写时（未激活）原样上屏
    engine.set_input("kdfafa");
    assert_eq!(engine.take_raw(), "kdfafa");
}

#[test]
fn fuma_backspace_deletes_codes_keywise() {
    let mut engine = fuma_engine();
    engine.set_input("kdfafX");
    // 末尾一对辅码键一起删
    assert!(engine.delete_syllable_backward());
    assert_eq!(engine.composition().text(), "kdfa");
    // 再删就是音节对
    assert!(engine.delete_syllable_backward());
    assert_eq!(engine.composition().text(), "kd");
    // 只敲了一个辅码键（未激活）单删
    engine.set_input("kdfaf");
    assert!(engine.delete_syllable_backward());
    assert_eq!(engine.composition().text(), "kdfa");
}

#[test]
fn fuma_without_table_or_full_pinyin_changes_nothing() {
    // 没有表：大写字母不是拼音键，照旧当英文直输
    let mut plain_xiaohe = xiaohe();
    plain_xiaohe.set_input("kdfaFX");
    assert!(plain_xiaohe.raw_mode());
    // 辅码只在双拼下生效：全拼 + 表不激活，大写字母照旧英文直输
    let mut plain = engine();
    plain.set_fuma(Some(fuma_table()));
    plain.set_input("kaifafX");
    assert!(plain.raw_mode());
    let query = plain.query().unwrap();
    assert_eq!(query.candidates.items[0].text, "kaifafX");
}
