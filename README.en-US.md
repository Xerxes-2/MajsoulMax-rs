# Majsoul Max-rs

[![en](https://img.shields.io/badge/lang-en-blue.svg)](https://github.com/Xerxes-2/MajsoulMax-rs/blob/master/README.en-US.md)
[![cn](https://img.shields.io/badge/lang-cn-green.svg)](https://github.com/Xerxes-2/MajsoulMax-rs/blob/master/README.md)

**This project is inspired by [MajsoulMax](https://github.com/Avenshy/MajsoulMax)**

Unlock all Mahjong Soul characters, skins, outfits, and more via a man-in-the-middle approach powered by [hudsucker](https://github.com/omjadas/hudsucker). Supports both the web version and the PC / Android client.

Also supports sending live Mahjong Soul games to the [Japanese Mahjong Assistant mahjong-helper](https://github.com/EndlessCheng/mahjong-helper); game log analysis is not supported.

This tool is completely free and open-source—if you paid for it, you were scammed!

## 🤔 Why Reinvent the Wheel

### 🥰 Advantages

-   Written in Rust for better performance and a smaller binary than Python (the Python version often feels laggy due to high latency).
-   Uses multi-threaded async processing for higher performance.
-   Native support for Windows, Linux, macOS, and Android—just download the binary and run.
-   Supports the Android client (via Termux and NekoBox).

### 🥲 Disadvantages

-   Compared with mitmproxy, hudsucker does not support upstream proxies and needs Clash.
-   `lq.rs` cannot be updated dynamically; recompilation is required.

## 🧭 Current Mahjong Soul Versions (Real Time)

![CHINESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.maj-soul.com%2F1%2Fversion.json&label=CHINESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAACsklEQVQ4ja2Tf0zMcRjHX9/7Ud0l59RCFssWmemGom5mNysbLb+GuM2G8mOMCGu0rD9aNuRHymxnNqbGqcwMG1lrRLmkRXOESoit311X1/34+MO37YY/vf96Pu89z+f5fJ73+5H4G1nAWWAPkAzoZL4LeAiU+SdLfrESKAZOAzeAVpl3AiogABgD5gImwPNn5yYgDngAXANKgZbbFw+J+jsnBdAJvA8KUNnNqUYBaP2LrwFXgQrAAtjmzAwfFG23RHtNkUhNinMaYiJ7gI6c3SkeoA4Q/l+oAdqB/mPpSVsLTu3US6FpXwDF9nVL9RmbVwSHaCTcPiV5xZUj96oaPsp1lSpg/vSwENdhc+KGyHCdtmvIKxYkZA51PyucUf36Kxv3n+uKmBqqWGVapPGM9JG5KUFzr6ohWJ7NAQnIBmI1geoge/mh9f29fdR+HPZGTZ2kmBwRKRmiI3jywi6Mi2ZL+oXbPztsRbOiV+b2dnUPdAIxKmBKmE4b8qgwLZUxF0XWl+4Rj9LF4hiNVxGkvPGuQxgXRkt371f7ANWrNx84l71Jv/mopRMYVAE/50TqjZ9+OCi50+i+cnytOj79yvDwqMtRaUnR2Vvs0r7cy85qW2s3oG77Nugzxc1UAG5gIrJ0pUBFaU6KAL5PC9P1lOWbReaWZaOXcs2eoaarwmErEsD36yczPG+tBwXQCAyMy/gcsALNQFNZvlkAn0ae5ouHJXt98fOi+svPZIhtqxOd9ttZotayS8hS5qnkC6xALBBoWhA1wdnf7QUCghRuQrWSlGQI11Q+fuk4f2TNhJp6OxesdX3AEiDB30x1QEvBjqVe+XnNQGN6imFsoOqEuHlijVApFe2ATbZxqL+RxmEBlgOjgE/mBOAC1PLZAIz74J9IloczCgzLcScwxO8N/b/4BZ4sCAP6Ouu4AAAAAElFTkSuQmCC&logoWidth=16) ![ENGLISH](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fmahjongsoul.game.yo-star.com%2Fversion.json&label=ENGLISH&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16) ![JAPANESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.mahjongsoul.com%2Fversion.json&label=JAPANESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16)

## 📢 Before You Start

Note: Unlocking characters is only effective locally; others will still see your original character and emojis. For example, sending the 3rd emoji of the new character will still show the 3rd emoji of your original character to everyone else.

> [!CAUTION]
> Mod responsibly—safety first.
>
> Improper use may get your account banned.
>
> This project is for learning and communication only; delete it within 24 hours of download and never use it for commercial purposes, otherwise you bear all consequences.
>
> Mahjong Soul may detect this project and ban accounts; the authors assume no liability for any outcome.
>
> Using this project means you acknowledge and agree to all of the above.

![Deal-in](https://memeprod.ap-south-1.linodeobjects.com/user-gif-post/1647655593730.gif)

## ✈️ Telegram Channel & Group

| Channel                                                                                                            | Group                                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- |
| [![频道 https://t.me/Mahjong_Soul](https://s2.loli.net/2022/11/08/4vS2BLMGhudkXQy.jpg)](https://t.me/Mahjong_Soul) | [![交流 https://t.me/Mahjong_Soul_Chat](https://s2.loli.net/2022/11/08/KL8A7U9fDsZEmjp.jpg)](https://t.me/Mahjong_Soul_Chat) |

You can click the images to join or scan the QR codes.

## 🥰 Current Features

The program contains two parts: `mod` and `helper`, essentially combining [majsoul_mod_plus](https://github.com/Avenshy/majsoul_mod_plus) and [mahjong-helper-majsoul-mitmproxy](https://github.com/Avenshy/mahjong-helper-majsoul-mitmproxy).

By default `helper` is enabled and `mod` is disabled. To customize, edit `modSwitch` and `helperSwitch` in `./liqi_config/settings.json`.

### `mod` Features

-   Unlock all characters and skins
-   Unlock all decorations
-   Unlock all voices (callouts)
-   Unlock all titles
-   Unlock all loading CGs
-   Unlock all emojis (not recommended)
-   Force-enable convenient hints
    -   Due to Mahjong Soul's own code restrictions, the Throne Room cannot normally enable convenient hints, so **after enabling this feature, entering Throne Room games will change the top-left corner to “Jade Room”**. This is not a bug!
-   Favorite/star characters
-   Custom names
-   Display the player's server
-   TODO...

### `helper` Features

-   Send live games to [mahjong-helper](https://github.com/EndlessCheng/mahjong-helper)

## 🧐 Instructions for Use (Windows)

1. Modify the configuration
    - Adjust based on program prompts and your own needs
    - `liqi_config/settings.json` holds general settings, including the Helper and Mod toggles—`modSwitch` and `helperSwitch`; `false` means off, `true` means on
    - `liqi_config/settings.mod.json` holds Mod-specific settings
2. Start the program by running the executable
3. Start the game (web or client / Steam). Make sure Mahjong Soul traffic goes through the local `majsoul_max_rs` proxy (it listens on `127.0.0.1:23410` by default). A rule-based proxy app with override support such as `Clash` / `Surge` is recommended; see “Proxy & Routing” below for examples.
    - Web: usually you only need the browser to honor the system proxy or routing rules; no need to enable `TUN` / enhanced mode.
    - Client / Steam: likewise split the process traffic to `majsoul_max_rs` via `Clash` / `Surge`, but enable `TUN` / enhanced mode in the proxy app or the local process traffic will not be hijacked.
4. Log in and enjoy

macOS or Linux users can follow the same steps as Windows; step 3 differs slightly for proxy setup.

If you want to use the Android version, we assume you already have the technical know-how—keywords: `Termux`, `NekoBox`, only effective on route 1.

## 📦 Install the Certificate

Before configuring routing rules, import and trust the `hudsucker.cer` root certificate in your operating system (you can download it from [omjadas/hudsucker](https://github.com/omjadas/hudsucker/blob/main/examples/ca/hudsucker.cer)), otherwise HTTPS traffic may fail certificate verification.

### Windows

1. Locate the certificate file named `hudsucker.cer`
2. Double-click the certificate file
3. Click the `Install Certificate` button
4. If prompted, choose `Local Machine`, then click Next
5. Select `Place all certificates in the following store`, then click `Browse...`
6. Choose `Trusted Root Certification Authorities`, confirm, then click Next and Finish
7. If the system asks for permission, click Yes

### macOS

1. Locate the `hudsucker.cer` certificate file
2. Double-click the certificate file to open Keychain Access
3. In the left sidebar, select `System Keychain` -> `System`, search for `hudsucker`, and find the imported certificate (it will be untrusted)
4. Right-click the certificate named `hudsucker`, choose `Get Info`, and expand `Trust`
5. Set `When using this certificate` to `Always Trust`
6. Close the window and complete authentication when prompted

### iOS / iPadOS

If you deploy this project as a separate proxy node, you can use it on iOS / iPadOS, but you still need to trust the certificate on the device.

1. Send the `hudsucker.cer` certificate from your computer to your iPhone/iPad via AirDrop or another method. AirDrop is preferred because it imports automatically. For other methods, save it to Files first, then open the certificate from Files.
2. Go to `Settings -> Profile Downloaded` and tap Install
3. Go to `General -> About -> Certificate Trust Settings` and enable the `hudsucker` option

### Android

No test environment available; please search for the steps yourself.

## 🌐 Proxy & Routing

`majsoul_max_rs` starts a local HTTP proxy on `127.0.0.1:23410`. Use a rule-based proxy client that supports routing/override (such as `Clash` / `Surge`) to send Mahjong Soul traffic through this proxy.

> [!CAUTION]
>
> For native clients / Steam, enable `TUN` / enhanced mode in your proxy client to ensure the process traffic goes through `majsoul_max_rs`. Be sure to avoid loopback routing—traffic leaving `majsoul_max_rs` must not be routed back into itself.
>
> For the web version (browser), correctly configuring the system proxy or domain rules is usually sufficient; enhanced mode is typically unnecessary.

### Using Clash for Routing

```yml
proxies:
    - name: MajsoulMax-rs
      type: http
      server: 127.0.0.1
      port: 23410
      tls: false

proxy-groups:
    - name: 🀄 MahjongSoul
      type: select
      proxies:
          - MajsoulMax-rs
          - DIRECT

rules:
    # Required to avoid loopback routing
    - PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT
    # Choose one of the following based on your platform
    # Client / Steam
    - PROCESS-NAME,雀魂麻將,🀄 MahjongSoul
    - PROCESS-NAME,jantama_mahjongsoul.exe,🀄 MahjongSoul
    - PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 MahjongSoul
    # Web (browser)
    - DOMAIN-KEYWORD,majsoul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,maj-soul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 MahjongSoul
    - DOMAIN-KEYWORD,catmjstudio,🀄 MahjongSoul
```

### Using Surge for Routing

```text
[Proxy]
MajsoulMax-rs = http, 127.0.0.1, 23410

[Proxy Group]
🀄 MahjongSoul = select, MajsoulMax-rs, DIRECT

[Rule]
# Required to avoid loopback routing
PROCESS-NAME,majsoul_max_rs,DIRECT
# Choose one of the following based on your platform
# Client / Steam
PROCESS-NAME,雀魂麻將,🀄 MahjongSoul
# Web
DOMAIN-KEYWORD,majsoul,🀄 MahjongSoul
DOMAIN-KEYWORD,maj-soul,🀄 MahjongSoul
DOMAIN-KEYWORD,mahjongsoul.com,🀄 MahjongSoul
DOMAIN-KEYWORD,catmjstudio,🀄 MahjongSoul
```

### Scenarios Without PROCESS-NAME Rules

If you are on iOS / iPadOS or any platform that cannot use `PROCESS-NAME` rules, mimic the domain keyword routing used for the web version (Clash example below). In this scenario, you must deploy `majsoul_max_rs` separately—that is, not on the same machine as your main host—to avoid loopback routing. Consider running the proxy node on a VPS, for example via [MajsoulMax-rs-docker](https://github.com/zhuozhiyongde/MajsoulMax-rs-docker).

```yml
rules:
    - DOMAIN-KEYWORD,majsoul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,maj-soul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 MahjongSoul
    - DOMAIN-KEYWORD,catmjstudio,🀄 MahjongSoul
```

### Override Examples

If you use a proxy client that supports overrides (such as `Clash Verge`, `Clash Party`, or `Surge` with override files), put the above node and rules into a separate override file / global script and enable it only when you play Mahjong Soul.

#### Clash Verge Global Extension Script (JS) Example

See the [official docs](https://www.clashverge.dev/guide/script.html) for configuration. On the “Subscriptions” page, right-click `Global Extension Script` and choose “Edit file”:

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
        name: '🀄 MahjongSoul',
        type: 'select',
        proxies: ['DIRECT', 'MajsoulMax'],
        icon: 'https://www.maj-soul.com/homepage/img/logotaiwan.png',
    });

    const bypass = [
        'PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT',
    ];

    const clientRules = [
        'PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 MahjongSoul',
        'PROCESS-NAME,jantama_mahjongsoul.exe,🀄 MahjongSoul',
        'PROCESS-NAME,雀魂麻將,🀄 MahjongSoul',
    ];

    const webRules = [
        'DOMAIN-KEYWORD,majsoul,🀄 MahjongSoul',
        'DOMAIN-KEYWORD,maj-soul,🀄 MahjongSoul',
        'DOMAIN-KEYWORD,mahjongsoul.com,🀄 MahjongSoul',
        'DOMAIN-KEYWORD,catmjstudio,🀄 MahjongSoul',
    ];

    config.rules.unshift(...bypass, ...clientRules, ...webRules);
    return config;
}
```

#### Clash Party (formerly Mihomo Party) Override Example

See the [official docs](https://clashparty.org/docs/guide/override/yaml)). In Clash Party, click `+` on the left `Override` page, choose `New YAML`, paste the following, save, then click the `...` on that override card and set `Enable globally`.

```yml
+proxies:
    - name: MajsoulMax-rs
      type: http
      server: 127.0.0.1
      port: 23410
      tls: false
+proxy-groups:
    - name: 🀄 MahjongSoul
      type: select
      proxies:
          - MajsoulMax-rs
          - DIRECT
+rules:
    # Required to avoid loopback routing
    - PROCESS-NAME-REGEX,majsoul_max_rs.*?,DIRECT
    # Choose one of the following based on your platform
    # Client / Steam
    - PROCESS-NAME,雀魂麻將,🀄 MahjongSoul
    - PROCESS-NAME,jantama_mahjongsoul.exe,🀄 MahjongSoul
    - PROCESS-NAME,Jantama_MahjongSoul.exe,🀄 MahjongSoul
    # Web version (browser)
    - DOMAIN-KEYWORD,majsoul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,maj-soul,🀄 MahjongSoul
    - DOMAIN-KEYWORD,mahjongsoul.com,🀄 MahjongSoul
    - DOMAIN-KEYWORD,catmjstudio,🀄 MahjongSoul

```

## 🤔 Q&A

1. Why are `liqi` and `lqc.lqbin` updated automatically? What if the update fails?
    - liqi:
        - Consists of three files, including `liqi.json` and the generated `liqi.proto` and `liqi.desc`, used to parse Mahjong Soul protobuf messages
        - If the update fails, some messages may not parse (for example, new event messages)
    - lqc.lqbin:
        - Used to obtain all characters, outfits, items, and other game assets
        - If the update fails, new assets (such as new characters and items) may not be available
    - If auto-update fails, download them from [AutoLiqi > Releases](https://github.com/Xerxes-2/AutoLiqi/releases/latest) and manually replace the files with the same name under `./liqi_config`
2. How do I use this together with my own proxy (VPN / airport)?
    - Use a rule-based proxy with override support (such as `Clash` / `Surge`) to first route Mahjong Soul traffic to the local `MajsoulMax-rs` node, then forward from that node to your original proxy nodes.
    - See “Proxy & Routing” above for example configurations; you can also keep the examples in a separate override file and enable it only when needed.
3. Other questions?
   Join our [Telegram group](README.en-US.md#%EF%B8%8F-telegram-channel--group).

## 🛠️ Development Dependencies

-   [Rust](https://www.rust-lang.org/) >= 1.85
-   [Protoc](https://github.com/protocolbuffers/protobuf)

## 📜 License

[GNU General Public License v3.0](./LICENSE)
