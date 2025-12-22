# 雀魂 Max-rs

[![en](https://img.shields.io/badge/lang-en-blue.svg)](https://github.com/Xerxes-2/MajsoulMax-rs/blob/master/README.en-US.md)
[![cn](https://img.shields.io/badge/lang-cn-green.svg)](https://github.com/Xerxes-2/MajsoulMax-rs/blob/master/README.md)

**本项目启发自 [MajsoulMax](https://github.com/Avenshy/MajsoulMax)**

雀魂解锁全角色、皮肤、装扮等，基于 [hudsucker](https://github.com/omjadas/hudsucker) 的中间人攻击方式，支持网页版与电脑 / Android 客户端。

同时支持将雀魂的实时牌局发到 [日本麻将助手 mahjong-helper](https://github.com/EndlessCheng/mahjong-helper)，不支持牌谱分析。

本工具完全免费、开源，如果您为此付费，说明您被骗了！

## 🤔 为什么重新造轮子

### 🥰 优点

-   本项目使用 Rust 语言编写，相比 Python 有更好的性能和更小的体积（Python 版经常因为延迟过高使用体验不佳）
-   使用了多线程异步处理，提高了性能
-   原生支持 Windows、Linux、macOS、Android 等多平台，只需下载二进制一键运行
-   支持 Android 客户端（通过 Termux 和 NekoBox）

### 🥲 缺点

-   hudsucker 相比 mitmproxy，不支持上游代理，需要借助 Clash
-   无法动态更新 `lq.rs`，需要重新编译

## 🧭 当前雀魂各服版本（实时更新）

![CHINESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.maj-soul.com%2F1%2Fversion.json&label=CHINESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAACsklEQVQ4ja2Tf0zMcRjHX9/7Ud0l59RCFssWmemGom5mNysbLb+GuM2G8mOMCGu0rD9aNuRHymxnNqbGqcwMG1lrRLmkRXOESoit311X1/34+MO37YY/vf96Pu89z+f5fJ73+5H4G1nAWWAPkAzoZL4LeAiU+SdLfrESKAZOAzeAVpl3AiogABgD5gImwPNn5yYgDngAXANKgZbbFw+J+jsnBdAJvA8KUNnNqUYBaP2LrwFXgQrAAtjmzAwfFG23RHtNkUhNinMaYiJ7gI6c3SkeoA4Q/l+oAdqB/mPpSVsLTu3US6FpXwDF9nVL9RmbVwSHaCTcPiV5xZUj96oaPsp1lSpg/vSwENdhc+KGyHCdtmvIKxYkZA51PyucUf36Kxv3n+uKmBqqWGVapPGM9JG5KUFzr6ohWJ7NAQnIBmI1geoge/mh9f29fdR+HPZGTZ2kmBwRKRmiI3jywi6Mi2ZL+oXbPztsRbOiV+b2dnUPdAIxKmBKmE4b8qgwLZUxF0XWl+4Rj9LF4hiNVxGkvPGuQxgXRkt371f7ANWrNx84l71Jv/mopRMYVAE/50TqjZ9+OCi50+i+cnytOj79yvDwqMtRaUnR2Vvs0r7cy85qW2s3oG77Nugzxc1UAG5gIrJ0pUBFaU6KAL5PC9P1lOWbReaWZaOXcs2eoaarwmErEsD36yczPG+tBwXQCAyMy/gcsALNQFNZvlkAn0ae5ouHJXt98fOi+svPZIhtqxOd9ttZotayS8hS5qnkC6xALBBoWhA1wdnf7QUCghRuQrWSlGQI11Q+fuk4f2TNhJp6OxesdX3AEiDB30x1QEvBjqVe+XnNQGN6imFsoOqEuHlijVApFe2ATbZxqL+RxmEBlgOjgE/mBOAC1PLZAIz74J9IloczCgzLcScwxO8N/b/4BZ4sCAP6Ouu4AAAAAElFTkSuQmCC&logoWidth=16) ![ENGLISH](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fmahjongsoul.game.yo-star.com%2Fversion.json&label=ENGLISH&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16) ![JAPANESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.mahjongsoul.com%2Fversion.json&label=JAPANESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16)

## 📢 用前须知

注意：解锁人物仅在本地有效，别人还是只能看到你原来的角色，发表情也是原来角色的表情。
比如使用新角色发第 3 个表情，实际上其他人看到的是原来角色的第 3 个表情。

> [!CAUTION]
> 魔改千万条，安全第一条。
>
> 使用不规范，账号两行泪。
>
> 本项目仅供学习参考交流，请使用者于下载 24 小时内自行删除，不得用于商业用途，否则后果自负。
>
> 雀魂官方可能会检测并封号，如产生任何后果与作者无关。
>
> 使用本项目则表示你已知悉并同意以上条款。

![放铳放铳](https://memeprod.ap-south-1.linodeobjects.com/user-gif-post/1647655593730.gif)

## ✈️Telegram 频道 & 交流群

| 频道                                                                                                               | 交流群                                                                                                                       |
| ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| [![频道 https://t.me/Mahjong_Soul](https://s2.loli.net/2022/11/08/4vS2BLMGhudkXQy.jpg)](https://t.me/Mahjong_Soul) | [![交流 https://t.me/Mahjong_Soul_Chat](https://s2.loli.net/2022/11/08/KL8A7U9fDsZEmjp.jpg)](https://t.me/Mahjong_Soul_Chat) |

可以直接点击图片进入，也可以通过扫码进入。

## 🥰 当前功能

程序包含两部分：`mod` 和 `helper`，可以说是 [雀魂 mod_plus](https://github.com/Avenshy/majsoul_mod_plus) 和 [mahjong-helper-majsoul-mitmproxy](https://github.com/Avenshy/mahjong-helper-majsoul-mitmproxy) 的融合。

程序默认配置为启用 `helper`、禁用 `mod`。如需自定义，请修改 `./liqi_config/settings.json` 中的 `mod_switch` 和 `helper_switch`。

### `mod` 功能

-   解锁所有角色与皮肤
-   解锁所有装扮
-   解锁所有语音（报菜名）
-   解锁所有称号
-   解锁所有加载 CG
-   解锁所有表情（不推荐开启）
-   强制启用便捷提示
    -   由于雀魂本身代码限制，王座无法正常启用便捷提示，因此，**开启此功能后进入王座对局，左上角会变成 “玉之间”**。请注意，这不是 BUG！
-   支持星标角色
-   自定义名称
-   显示玩家所在服务器
-   TODO……

### `helper` 功能

-   将对局发送到 [mahjong-helper（雀魂小助手）](https://github.com/EndlessCheng/mahjong-helper)

## 🧐 使用说明 (Windows)

1. 修改配置
    - 根据程序提示和自身需求修改
    - 在 `liqi_config/settings.json` 可以设置通用设置，包括 Helper 和 Mod 的开关 ——`modSwitch` 与 `helperSwitch`，`false` 为关 `true` 为开
    - 在 `liqi_config/settings.mod.json` 可以设置 Mod 专有设置
2. 启动程序，直接运行可执行文件
3. 启动游戏，分为网页版和客户端 / Steam 端。需要确保雀魂相关流量会经过本地 `majsoul_max_rs` 代理（默认监听 `127.0.0.1:23410`），推荐使用支持规则分流与覆写的代理软件（如 `Clash` / `Surge`），具体示例见下文 “代理与分流” 一节。
    - 网页版：通常只需让浏览器通过系统代理或规则分流使用 `majsoul_max_rs` 节点，无需开启 `TUN` / 增强模式。
    - 客户端 / Steam 端：同样通过 `Clash` / `Surge` 将进程流量分流到 `majsoul_max_rs`，但需要在代理软件中开启 `TUN` / 增强模式，否则本地进程流量不会被劫持。
4. 登录游戏开始享受

macOS 或 Linux 用户，可以参考 Windows 的步骤，步骤 3 有所不同。

如果你想使用 Android 版，默认你已经有足够的技术能力，这里不再赘述，只提几个关键词：`Termux`、`NekoBox`，并且只在线路 1 有效。

## 🌐 代理与分流

`majsoul_max_rs` 默认在本地 `127.0.0.1:23410` 启动一个 HTTP 代理。推荐使用支持规则分流和覆写的代理软件（如 `Clash` / `Surge`），将雀魂相关流量导向该代理。

在配置分流规则前，请先在系统中导入并信任 `hudsucker.cer` 根证书（可从 [omjadas/hudsucker](https://github.com/omjadas/hudsucker/blob/main/examples/ca/hudsucker.cer) 下载），否则 HTTPS 流量可能会因为证书校验失败而无法正常工作。

> [!CAUTION]
>
> 本地客户端 / Steam 端等进程需要在代理软件中开启 `TUN` / 增强模式，才能保证进程流量经过 `majsoul_max_rs`；但请务必注意避免回环代理，即你要保证从 `majsoul_max_rs` 发出的流量不会被分流回自身。
>
> 网页版（浏览器）一般只要正确配置系统代理或域名规则即可，通常不需要开启增强模式。

### 使用 Clash 分流

```yml
proxies:
    - name: MajsoulMax-rs
      type: http
      server: 127.0.0.1
      port: 23410
      tls: false

proxy-groups:
    - name: 🀄 雀魂麻将
      type: select
      proxies:
          - MajsoulMax-rs
          - DIRECT

rules:
    # 必须有这条，否则会导致回环代理
    - PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT
    # 下面两部分可选，根据你的游戏平台选择其一即可
    # 客户端 / Steam
    - PROCESS-NAME,雀魂麻將,🀄 雀魂麻将
    - PROCESS-NAME,jantama_mahjongsoul.exe,🀄 雀魂麻将
    - PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 雀魂麻将
    # 网页版（浏览器）
    - DOMAIN-KEYWORD,majsoul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,maj-soul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmjstudio,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmajsoul,🀄 雀魂麻将
```

### 使用 Surge 分流

```text
[Proxy]
MajsoulMax-rs = http, 127.0.0.1, 23410

[Proxy Group]
🀄 雀魂麻将 = select, MajsoulMax-rs, DIRECT

[Rule]
# 必须有这条，否则会导致回环代理
PROCESS-NAME,majsoul_max_rs,DIRECT
# 下面两部分可选，根据你的游戏平台选择其一即可
# 客户端 / Steam
PROCESS-NAME,雀魂麻將,🀄 雀魂麻将
# 网页版
DOMAIN-KEYWORD,majsoul,🀄 雀魂麻将
DOMAIN-KEYWORD,maj-soul,🀄 雀魂麻将
DOMAIN-KEYWORD,mahjongsoul.com,🀄 雀魂麻将
DOMAIN-KEYWORD,catmjstudio,🀄 雀魂麻将
DOMAIN-KEYWORD,catmajsoul,🀄 雀魂麻将
```

### 无法使用进程名规则的场景

如果你是 iOS / iPadOS 等无法使用 `PROCESS-NAME` 规则的平台，可以仿照网页版使用域名关键字或 IP 分流（Clash 示例），但此时需要分离部署 `majsoul_max_rs`，也即其不能和你的主机在同一机子上，否则会导致回环代理，分离部署可以考虑将代理节点部署在 VPS 上，参见 [MajsoulMax-rs-docker](https://github.com/zhuozhiyongde/MajsoulMax-rs-docker)

```yml
rules:
    - DOMAIN-KEYWORD,majsoul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,maj-soul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmjstudio,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmajsoul,🀄 雀魂麻将
```

### 覆写配置示例


如果你使用的是支持覆写的代理客户端（如 `Clash Verge`、`Clash Party` 或支持覆写配置的 `Surge`），建议将上述节点与规则写在单独的覆写文件 / 全局脚本中，需要玩雀魂时再一键启用。

#### Clash Verge 全局扩展脚本（JS）示例

参考 [官方文档](https://www.clashverge.dev/guide/script.html)，可以按照如下方法进行配置。

在 “订阅” 页面右键 `全局扩展脚本`，选择 “编辑文件”:

```js
function main(config) {
    config.proxies.push({
        name: 'MajsoulMax',
        type: 'http',
        server: '127.0.0.1',
        port: 23410,
        tls: false,
    });

    config['proxy-groups'].push({
        name: '🀄 雀魂麻将',
        type: 'select',
        proxies: ['DIRECT', 'MajsoulMax'],
        icon: 'https://www.maj-soul.com/homepage/img/logotaiwan.png',
    });

    const bypass = [
        'PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT',
    ];

    const clientRules = [
        'PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 雀魂麻将',
        'PROCESS-NAME,jantama_mahjongsoul.exe,🀄 雀魂麻将',
        'PROCESS-NAME,雀魂麻將,🀄 雀魂麻将',
    ];

    const webRules = [
        'DOMAIN-KEYWORD,majsoul,🀄 雀魂麻将',
        'DOMAIN-KEYWORD,maj-soul,🀄 雀魂麻将',
        'DOMAIN-KEYWORD,mahjongsoul.com,🀄 雀魂麻将',
        'DOMAIN-KEYWORD,catmjstudio,🀄 雀魂麻将',
        'DOMAIN-KEYWORD,catmajsoul,🀄 雀魂麻将',
    ];

    config.rules.unshift(...bypass, ...clientRules, ...webRules);
    return config;
}
```

#### Clash Party（原 Mihomo Party）覆写示例：

参考 [官方文档](https://clashparty.org/docs/guide/override/yaml)），可以按照如下方式进行配置。

在 Clash Party 左侧 `覆写` 页面点击 `+` 号，选择 `新建 YAML`，然后复制如下内容，点击 `确认` 保存，然后点击对应覆写卡片右上角的 `...` 图标，选择 `编辑信息` - `全局启用`。

```yml
+proxies:
    - name: MajsoulMax-rs
      type: http
      server: 127.0.0.1
      port: 23410
      tls: false
+proxy-groups:
    - name: 🀄 雀魂麻将
      type: select
      proxies:
          - MajsoulMax-rs
          - DIRECT
+rules:
    # 必须有这条，否则会导致回环代理
    - PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT
    # 下面两部分可选，根据你的游戏平台选择其一即可
    # 客户端 / Steam
    - PROCESS-NAME,雀魂麻將,🀄 雀魂麻将
    - PROCESS-NAME,jantama_mahjongsoul.exe,🀄 雀魂麻将
    - PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 雀魂麻将
    # 网页版（浏览器）
    - DOMAIN-KEYWORD,majsoul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,maj-soul,🀄 雀魂麻将
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmjstudio,🀄 雀魂麻将
    - DOMAIN-KEYWORD,catmajsoul,🀄 雀魂麻将

```

## 🤔 Q&A

1. 为什么要自动更新 liqi 和 lqc.lqbin？更新失败有什么影响？
    - liqi：
        - 共有 3 个文件，包括 `liqi.json` 和根据其生成的 `liqi.proto` 和 `liqi.desc`，用于解析雀魂 protobuf 消息
        - 如果更新失败，可能会导致消息无法解析（如新活动的消息）
    - lqc.lqbin：
        - 用于获取全部角色、装扮、物品等游戏资源
        - 如果更新失败，可能会导致无法获取新资源（如新角色、物品等）
    - 如果自动更新失败，可以在 [AutoLiqi > Releases](https://github.com/Xerxes-2/AutoLiqi/releases/latest) 下载，并手动替换 `./liqi_config` 文件夹下的同名文件
2. 如何同时启用自己的代理（机场 / VPN）？
    - 推荐使用支持规则和覆写的代理软件（如 `Clash` / `Surge`），将雀魂相关流量首先分流到本地 `MajsoulMax-rs` 节点，再由该节点转发到你原有的代理节点。
    - 具体示例配置见上文 “代理与分流” 一节，你也可以将示例写成单独的覆写配置文件，在需要时再启用。
3. 还有其它问题？
   在上方加入我们的 [Telegram 群](https://github.com/Xerxes-2/MajsoulMax-rs?tab=readme-ov-file#%EF%B8%8Ftelegram%E9%A2%91%E9%81%93%E4%BA%A4%E6%B5%81%E7%BE%A4)

## 🛠️ 开发依赖

-   [Rust](https://www.rust-lang.org/) >= 1.85
-   [Protoc](https://github.com/protocolbuffers/protobuf)
