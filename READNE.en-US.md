# MajsoulMax-rs

**This project is inspired by [MajsoulMax](https://github.com/Avenshy/MajsoulMax)**

Unlock all characters, skins, and decorations in Mahjong Soul, using a man-in-the-middle attack based on [hudsucker](https://github.com/omjadas/hudsucker). Supports both the web version and the PC/Android client.

Also capable of sending real time Mahjong Soul game data to [Japanese Mahjong Assistant mahjong-helper](https://github.com/EndlessCheng/mahjong-helper), but does not support game log analysis.

This tool is completely free and open-source. If you paid for it, you've been scammed!

## 🤔 Why Reinvent the Wheel?

### 🥰 Advantages

- This project is written in Rust, which offers better performance and smaller size compared to Python (the Python version often suffers from high latency and poor user experience).
- Utilizes multi-threaded asynchronous processing to improve performance.
- Natively supports multiple platforms including Windows, Linux, macOS, and Android; just download and run the binary.
- Supports the Android client (via Termux and NekoBox).

### 🥲 Disadvantages

- Unlike mitmproxy, hudsucker does not support upstream proxies and requires the help of Clash.
- Cannot dynamically update `lq.rs`, requiring recompilation.

## 🧭 Current Mahjong Soul Versions (Updated in Real Time)

![CHINESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.maj-soul.com%2F1%2Fversion.json&label=CHINESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAACsklEQVQ4ja2Tf0zMcRjHX9/7Ud0l59RCFssWmemGom5mNysbLb+GuM2G8mOMCGu0rD9aNuRHymxnNqbGqcwMG1lrRLmkRXOESoit311X1/34+MO37YY/vf96Pu89z+f5fJ73+5H4G1nAWWAPkAzoZL4LeAiU+SdLfrESKAZOAzeAVpl3AiogABgD5gImwPNn5yYgDngAXANKgZbbFw+J+jsnBdAJvA8KUNnNqUYBaP2LrwFXgQrAAtjmzAwfFG23RHtNkUhNinMaYiJ7gI6c3SkeoA4Q/l+oAdqB/mPpSVsLTu3US6FpXwDF9nVL9RmbVwSHaCTcPiV5xZUj96oaPsp1lSpg/vSwENdhc+KGyHCdtmvIKxYkZA51PyucUf36Kxv3n+uKmBqqWGVapPGM9JG5KUFzr6ohWJ7NAQnIBmI1geoge/mh9f29fdR+HPZGTZ2kmBwRKRmiI3jywi6Mi2ZL+oXbPztsRbOiV+b2dnUPdAIxKmBKmE4b8qgwLZUxF0XWl+4Rj9LF4hiNVxGkvPGuQxgXRkt371f7ANWrNx84l71Jv/mopRMYVAE/50TqjZ9+OCi50+i+cnytOj79yvDwqMtRaUnR2Vvs0r7cy85qW2s3oG77Nugzxc1UAG5gIrJ0pUBFaU6KAL5PC9P1lOWbReaWZaOXcs2eoaarwmErEsD36yczPG+tBwXQCAyMy/gcsALNQFNZvlkAn0ae5ouHJXt98fOi+svPZIhtqxOd9ttZotayS8hS5qnkC6xALBBoWhA1wdnf7QUCghRuQrWSlGQI11Q+fuk4f2TNhJp6OxesdX3AEiDB30x1QEvBjqVe+XnNQGN6imFsoOqEuHlijVApFe2ATbZxqL+RxmEBlgOjgE/mBOAC1PLZAIz74J9IloczCgzLcScwxO8N/b/4BZ4sCAP6Ouu4AAAAAElFTkSuQmCC&logoWidth=16) ![ENGLISH](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fmahjongsoul.game.yo-star.com%2Fversion.json&label=ENGLISH&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16) ![JAPANESE](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fgame.mahjongsoul.com%2Fversion.json&label=JAPANESE&query=$.version&color=FF8C00&logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAADT0lEQVQ4jUXN2WtcVQDA4d+520xmsbOkM5OYNJkak5hJNQtE2mCgkj6ILUVEoTSIG0KVgpT4oIjig4qi5k0U8cWttr5IoUGoFBrRSilN1DRdNBMbkzTNNklmJplz7tx7fPDB7x/4BMDJZ0/kDw4fznquj1/1AM3Ia5/hyQpVDQDzi7OkExl2ptLUJOKcuzbO2V8vCOv1x+s/MKans3LuIyIdRynd0ly7uoTcKpPt7mN2coKqlLRm2/B8jTYExcIa+1v28Ol7J7TIf9hRCaVigfDunQR2JHA3lxl+qYxhaBKpDCuLC3QPPgJas3j9KoZpQjhMOl1H+K4Y1sJs0agrCUztM/zxGMeH2gjYEaSSzM3O4FgO4+dGEQLC4Si1u5qRrqK0WUQrhTW3IrmnM45lC159vp1kMoowobauntvzc9i2hVKKUDiMcl0W/84jXUm5JowhQIzk7leTRW0fGfA4m0/QfsegIRpgNKDJZOqZmvmdvs5u1gvbuK6ivFXGlRLbMBGAtb5hkAraXPzRZSkh6acGUXbZdMu8eWyIny63MDVxk3+W5onVxHBxaWxuZGlhhYrcRjzduU/llGl3KY0SJnlHcysTx/VW+eP6DZoMh0LI4eT571GywtTYz1y8fIWp8VlCdg1W0AkS3VYULJMzVolUIkMgaPHOyLv8cPo8k59/SeqhfRTXitS2t/DgkQYSjfWE1Rl+ufIXljAMqgEHp1RBahfTsdHKRxgWj77wBLmeNpr2H0AIF9Bow+C7T74l29fL1tgkVkVJNu7OMDjQT/TUN6B9TMNEBwJQkTQ/cB+itASRHSAEIGgtbhOrzbArlUYcasqpunjSLhTXiIaieNojFkvSmmvj2NvHwXbAMEEDAnTVhdI6zww+RyaewPJ9g9urKwRtC9NyuLO6zuDe3RSki7YCCK1B6/92IRCmBaEohmUQb85iKe2vVIqbdenaDOubCoFg8OWnyJ8eRXo+QeGDAIQBmOAp3nrxDfDgz/wMAuDeUFL31LewXZGEgkFeOfoYyT2NLE/cxGpvpuvQAbTvUcXgyd6H0VWH3q5+Jm78tiH431pnvCGeSzZxsKODvQM9mKEwSkne/+Ir5pYXiVgRpCoSCcY4NX3paw1D/wJx5WDqjkxa0wAAAABJRU5ErkJggg==&logoWidth=16)

## 📢 Before You Start

Note: Unlocking characters is only effective locally; others will still see your original character and emojis.

**Warning:**
The official may detect and ban your account!
Any consequences are the responsibility of the user!
Using this script implies agreement to this term.

## ✈️ Telegram Channel & Group

[![频道 https://t.me/Mahjong_Soul](https://s2.loli.net/2022/11/08/4vS2BLMGhudkXQy.jpg)](https://t.me/Mahjong_Soul)[![交流 https://t.me/Mahjong_Soul_Chat](https://s2.loli.net/2022/11/08/KL8A7U9fDsZEmjp.jpg)](https://t.me/Mahjong_Soul_Chat)

You can click the images to join or scan the QR codes.

## 🥰 Current Features

The program consists of two parts: `mod` and `helper`, which are a combination of [Mahjong Soul mod_plus](https://github.com/Avenshy/mahjong_mod_plus) and [mahjong-helper-mahjong-mitmproxy](https://github.com/Avenshy/mahjong-helper-mahjong-mitmproxy).

The default configuration enables `helper` and disables `mod`. To customize, modify the `mod_switch` and `helper_switch` in `.\liqi_config\settings.json`.

### `mod` Features

- Unlock all characters and skins.
- Unlock all decorations.
- Unlock all voices (announce yakus).
- Unlock all titles.
- Unlock all loading CGs.
- Unlock all emojis (not recommended).
- Force enable convenient hints.
  - Due to Mahjong Soul's own code restrictions, the Throne Room cannot enable convenient hints normally, thus, **entering Throne Room matches will change the top-left corner to "Jade Room"**. Please note, this is not a BUG!
- Support for favorite characters.
- Customizable names.
- Display player's server.
- TODO...

### `helper` Features

- Sends game data to [mahjong-helper](https://github.com/EndlessCheng/mahjong-helper).

## 🧐 Instructions for Use (Windows)

1. Modify the configuration:
    - Follow program prompts and personal needs.
    - In `liqi_config/settings.json`, set the universal settings including toggles for Helper and Mod—`modSwitch` and `helperSwitch`, 0 is off, 1 is on.
    - In `liqi_config/settings.mod.json`, set the Mod-specific settings.
2. Start the program by running the executable file.
3. Start the game, either the web version or the client/Steam version.
   - For the web version (limited to `Chrome`/`Edge`):
     - Disable all Mahjong Soul related plugins and scripts in the browser, **completely disable or uninstall** any proxy-related plugins (e.g., `Proxy SwitchyOmega`).
     - Enter the game normally once using the browser.
     - Close all browser windows, check with Task Manager to ensure no processes remain.
     - Copy and paste the shortcut for Chrome or Edge to create a duplicate, right-click the shortcut duplicate, go to `Properties -> Target`, and **add a space** followed by `--proxy-server=127.0.0.1:23410 --ignore-certificate-errors https://mahjongsoul.yo-star.com/` after the target (modify the URL if playing on other servers).
   - For the client/Steam version:
     - Launch to the login screen, do not log in.
     - If automatically logged in, click the settings in the top right of the game to log out and return to the login screen.
     - Run [Proxifier](https://www.proxifier.com/) and configure it:
        - `Profile` > `Proxy Servers` > `Add`
        - `Address`: `127.0.0.1`
        - `Port`: `23410`
        - `Protocol`: `HTTPS`
        - After filling out, click Check to ensure you see a green `Test passed` under `Test 1`, ignore the rest.
        - `OK`
     - `Profile` > `Proxification Rules` > `Add`
        - `Name`: Choose any name.
        - `Enabled`: ✅
        - `Applications`: Fill according to the application you use to run the game, e.g., for the Steam client enter `jantama_mahjongsoul.exe`.
        - `Action`: `Proxy HTTPS 127.0.0.1`
        - `OK`
4. Log in to the game and enjoy.

Instructions for macOS and Linux are similar.

For the Android version, assuming you are technically proficient, only key terms are provided here: `Termux`, `NekoBox`, effective only on game route 1.

## 🤔 Q&A

1. Why are `liqi` and `lqc.lqbin` updated automatically? What if the update fails?
   - liqi:
     - Consists of 3 files including `liqi.json` and its derivatives `liqi.proto` and `liqi.desc`, used for parsing Mahjong Soul protobuf messages.
     - If the update fails, message parsing might fail (e.g., new event messages).
   - lqc.lqbin:
     - Used to obtain all characters, decorations, and game items.
     - If the update fails, new items (like new characters, items) may not be retrievable.
   - If auto-update fails, download manually from [AutoLiqi > Releases](https://github.com/Xerxes-2/AutoLiqi/releases/latest) and replace the same name files under `./liqi_config`.
2. How to enable a proxy simultaneously?
   1. Use [Clash Verge](https://github.com/clash-verge-rev/clash-verge-rev).
   2. Disable the system proxy, enable service mode, and Tun mode.
   3. (Optional) Enable LAN connection; go to system settings > network > proxy, turn on "Use a proxy server", fill in the address and port (default is `http://127.0.0.1` and `7897`), make sure to prefix the address.
3. Other questions?
   Join our [Telegram group](https://github.com/Xerxes-2/MajsoulMax-rs/README.en-US.md#%EF%B8%8Ftelegram%E9%A2%91%E9%81%93%E4%BA%A4%E6%B5%81%E7%BE%A4) linked above.

## 🛠️ Development Dependencies

- [Rust](https://www.rust-lang.org/) >= 1.80.0
- [Protoc](https://github.com/protocolbuffers/protobuf)
