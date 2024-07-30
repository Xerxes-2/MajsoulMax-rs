use const_format::formatcp;
use criterion::{criterion_group, criterion_main, Criterion};
use majsoul_max_rs::lq;
use std::{hint::black_box, sync::LazyLock};

const ANNOUNCEMENT: &str = formatcp!(
    "<color=#f9963b>作者: Xerxes-2        版本: 123</color>\n
<b>本工具完全免费、开源，如果您为此付费，说明您被骗了！</b>\n
<b>本工具仅供学习交流, 请在下载后24小时内删除, 不得用于商业用途, 否则后果自负！</b>\n
<b>本工具有可能导致账号被封禁，给猫粮充钱才是正道！</b>\n\n
<color=#f9963b>开源地址：</color>\n
<href=https://github.com/Xerxes-2/MajsoulMax-rs>https://github.com/Xerxes-2/MajsoulMax-rs</href>\n\n
<color=#f9963b>再次重申：脚本完全免费使用，没有收费功能！</color>"
);

static MY_ANNOUNCEMENT: LazyLock<lq::Announcement> = LazyLock::new(|| lq::Announcement {
    title: "雀魂Max-rs载入成功".to_string(),
    id: 1145141919,
    header_image: "internal://2.jpg".to_string(),
    content: ANNOUNCEMENT.to_string(),
});

fn bench_clone_static_vs_construct(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone_static_vs_construct");
    group.bench_function("clone_static", |b| {
        b.iter(|| {
            let _ = black_box(MY_ANNOUNCEMENT.clone());
        })
    });
    group.bench_function("construct", |b| {
        b.iter(|| {
            let _ = black_box(lq::Announcement {
                title: "雀魂Max-rs载入成功".to_string(),
                id: 1145141919,
                header_image: "internal://2.jpg".to_string(),
                content: ANNOUNCEMENT.to_string(),
            });
        })
    });
}

criterion_group!(clone_static_vs_construct, bench_clone_static_vs_construct);
criterion_main!(clone_static_vs_construct);
