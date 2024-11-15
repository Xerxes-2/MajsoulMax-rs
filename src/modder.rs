use crate::{
    proto::{base::BaseMessage, lq, lq_config::ConfigTables, sheets},
    settings::ModSettings,
};
use anyhow::{anyhow, bail, Context, Result};
use bytes::Bytes;
use const_format::formatcp;
use prost::Message;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const ANNOUNCEMENT: &str = formatcp!(
    "<color=#f9963b>作者: Xerxes-2        版本: {VERSION}</color>\n
<b>本工具完全免费、开源，如果您为此付费，说明您被骗了！</b>\n
<b>本工具仅供学习交流, 请在下载后24小时内删除, 不得用于商业用途, 否则后果自负！</b>\n
<b>本工具有可能导致账号被封禁，给猫粮充钱才是正道！</b>\n\n
<color=#f9963b>开源地址：</color>\n
<href=https://github.com/Xerxes-2/MajsoulMax-rs>https://github.com/Xerxes-2/MajsoulMax-rs</href>\n\n
<color=#f9963b>再次重申：脚本完全免费使用，没有收费功能！</color>"
);

#[derive(Default)]
pub struct Safe {
    pub account_id: u32,
    pub characters: Vec<lq::Character>,
    pub main_character_id: u32,
    pub nickname: String,
    pub skin: u32,
    pub title: u32,
    pub loading_image: Vec<u32>,
    pub items: Vec<lq::Item>,
}

#[derive(Default)]
pub struct Modder {
    characters: Vec<sheets::ItemDefinitionCharacter>,
    skins: Vec<sheets::ItemDefinitionSkin>,
    titles: Vec<sheets::ItemDefinitionTitle>,
    items: Vec<sheets::ItemDefinitionItem>,
    loading_images: Vec<sheets::ItemDefinitionLoadingImage>,
    emojis: HashMap<u32, Vec<u32>>,
    endings: Vec<sheets::SpotRewards>,
    mod_settings: RwLock<ModSettings>,
    safe: RwLock<Safe>,
    contract: RwLock<String>,
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn to_vec<T: Message + std::default::Default>(buf: &[Vec<u8>]) -> Vec<T> {
    buf.iter()
        .map(|d| {
            T::decode(d.as_ref())
                .unwrap_or_else(|_| panic!("Failed to decode {}", std::any::type_name::<T>()))
        })
        .collect()
}

pub struct ModifyResult {
    pub msg: Option<Bytes>,
    pub inject_msg: Option<Bytes>,
}

impl Modder {
    pub async fn new(mod_settings: RwLock<ModSettings>) -> Result<Self> {
        let config_tables = ConfigTables::decode(mod_settings.read().await.resource.as_ref())
            .context("Failed to decode config tables")?;
        let mut modder = Modder {
            mod_settings,
            ..Default::default()
        };
        for data in config_tables.datas {
            // get '_' splitted words in data.table and data.sheet, turn into CamelCase then join by ""
            let class_name = data
                .table
                .split('_')
                .chain(data.sheet.split('_'))
                .map(capitalize)
                .collect::<String>();
            match class_name.as_str() {
                "ItemDefinitionCharacter" => {
                    modder.characters = to_vec(data.data.as_ref());
                }
                "ItemDefinitionSkin" => {
                    modder.skins = to_vec(data.data.as_ref());
                }
                "ItemDefinitionTitle" => {
                    modder.titles = to_vec(data.data.as_ref());
                }
                "ItemDefinitionItem" => {
                    modder.items = to_vec(data.data.as_ref());
                }
                "ItemDefinitionLoadingImage" => {
                    modder.loading_images = to_vec(data.data.as_ref());
                }
                "CharacterEmoji" => {
                    // one character can have multiple emojis
                    for d in data.data {
                        let emoji = sheets::CharacterEmoji::decode(d.as_ref())
                            .context("Failed to decode CharacterEmoji")?;
                        modder
                            .emojis
                            .entry(emoji.charid)
                            .or_default()
                            .push(emoji.sub_id);
                    }
                }
                "SpotRewards" => {
                    modder.endings = to_vec(data.data.as_ref());
                }
                _ => {}
            }
        }
        Ok(modder)
    }

    pub async fn modify(
        &self,
        buf: Bytes,
        from_client: bool,
        method_name: impl AsRef<str>,
    ) -> ModifyResult {
        let msg_type = buf[0];
        let res = match msg_type {
            0x01 => self.modify_notify(buf.clone()).await,
            0x02 => self.modify_req(buf.clone(), from_client).await,
            0x03 => self.modify_res(buf.clone(), from_client, method_name).await,
            _ => Err(anyhow!("Unimplemented message type: {msg_type}")),
        };
        match res {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to modify message: {e}");
                ModifyResult {
                    msg: Some(buf),
                    inject_msg: None,
                }
            }
        }
    }

    async fn modify_res(
        &self,
        buf: Bytes,
        from_client: bool,
        method_name: impl AsRef<str>,
    ) -> Result<ModifyResult> {
        let method_name = method_name.as_ref();
        let mut msg_block = BaseMessage::decode(&buf[3..])?;
        assert!(!from_client);
        if !msg_block.method_name.is_empty() {
            bail!("Non-empty respond method name");
        }
        let mut modified_data: Option<Vec<u8>> = None;
        match method_name {
            ".lq.Lobby.fetchAccountInfo" => {
                let mut msg = lq::ResAccountInfo::decode(msg_block.data.as_ref())?;
                if let Some(ref mut acc) = msg.account {
                    if acc.account_id == self.safe.read().await.account_id {
                        acc.avatar_frame = self.mod_settings.read().await.views_presets
                            [self.mod_settings.read().await.preset_index as usize]
                            .iter()
                            .find(|v| v.slot == 5)
                            .map(|v| v.item_id)
                            .unwrap_or_default();
                        acc.avatar_id = self.mod_settings.read().await.char_skin
                            [&self.mod_settings.read().await.main_char];
                        acc.verified = self.mod_settings.read().await.verified;
                        modified_data = Some(msg.encode_to_vec());
                    }
                }
            }
            ".lq.Lobby.fetchCharacterInfo" => {
                let mut msg = lq::ResCharacterInfo::decode(msg_block.data.as_ref())?;
                self.safe.write().await.main_character_id = msg.main_character_id;
                msg.characters
                    .clone_into(&mut self.safe.write().await.characters);
                msg.characters.clear();
                let characters = &self.mod_settings.read().await.char_skin;
                for char in characters.keys() {
                    let character = self.perfect_character(*char).await;
                    msg.characters.push(character);
                }
                msg.skins.clear();
                msg.skins.extend(self.skins.iter().map(|s| s.id));
                msg.main_character_id = self.mod_settings.read().await.main_char;
                msg.character_sort.clear();
                msg.character_sort
                    .extend(self.mod_settings.read().await.star_character.iter());
                msg.hidden_characters.clear();
                msg.finished_endings.clear();
                msg.rewarded_endings.clear();
                msg.finished_endings
                    .extend(self.endings.iter().map(|e| e.id));
                msg.rewarded_endings
                    .extend(self.endings.iter().map(|e| e.id));
                modified_data = Some(msg.encode_to_vec());
            }
            name if name == ".lq.Lobby.login" || name == ".lq.Lobby.oauth2Login" => {
                let mut msg = lq::ResLogin::decode(msg_block.data.as_ref())?;
                self.safe.write().await.account_id = msg.account_id;
                if let Some(ref mut account) = msg.account {
                    self.safe
                        .write()
                        .await
                        .nickname
                        .clone_from(&account.nickname);
                    self.safe.write().await.skin = account.avatar_id;
                    self.safe.write().await.title = account.title;
                    self.safe
                        .write()
                        .await
                        .loading_image
                        .clone_from(&account.loading_image);
                    if let Some(av) = self
                        .mod_settings
                        .read()
                        .await
                        .char_skin
                        .get(&self.mod_settings.read().await.main_char)
                    {
                        account.avatar_id = *av;
                    } else {
                        account.avatar_id =
                            400001 + (self.mod_settings.read().await.main_char % 100) * 100;
                    }
                    if !self.mod_settings.read().await.nickname.is_empty() {
                        account
                            .nickname
                            .clone_from(&self.mod_settings.read().await.nickname);
                    }
                    account.title = self.mod_settings.read().await.title;
                    account.loading_image.clear();
                    account
                        .loading_image
                        .extend(self.mod_settings.read().await.loading_bg.iter());
                    account.verified = self.mod_settings.read().await.verified;
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.createRoom" => {
                let mut msg = lq::ResCreateRoom::decode(msg_block.data.as_ref())?;
                if let Some(ref mut room) = msg.room {
                    for p in &mut room.persons {
                        self.change_player(p).await;
                    }
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.FastTest.authGame" => {
                let mut msg = lq::ResAuthGame::decode(msg_block.data.as_ref())?;
                if self.mod_settings.read().await.hint_on() {
                    if let Some(c) = msg.game_config.as_mut() {
                        if let Some(r) = c.mode.as_mut().and_then(|m| m.detail_rule.as_mut()) {
                            r.bianjietishi = true;
                        }
                        if let Some(ref mut id) = c.meta.as_mut().map(|m| m.mode_id) {
                            match *id {
                                a if (15..=16).contains(&a) => *id -= 4,
                                b if (25..=26).contains(&b) => *id -= 2,
                                _ => {}
                            }
                        }
                    }
                }
                for p in &mut msg.players {
                    self.change_player(p).await;
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchTitleList" => {
                let mut msg = lq::ResTitleList::decode(msg_block.data.as_ref())?;
                msg.title_list.clear();
                msg.title_list.extend(self.titles.iter().map(|t| t.id));
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchRoom" => {
                let mut msg = lq::ResSelfRoom::decode(msg_block.data.as_ref())?;
                if let Some(ref mut room) = msg.room {
                    for p in &mut room.persons {
                        self.change_player(p).await;
                    }
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchBagInfo" => {
                let mut msg = lq::ResBagInfo::decode(msg_block.data.as_ref())?;
                if let Some(ref mut bag) = msg.bag {
                    self.safe.write().await.items.clone_from(&bag.items);
                    bag.items.clear();
                    self.fill_bag(bag).await;
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchAllCommonViews" => {
                let mut msg = lq::ResAllcommonViews::decode(msg_block.data.as_ref())?;
                msg.r#use = self.mod_settings.read().await.preset_index;
                msg.views.clear();
                for (i, view) in self
                    .mod_settings
                    .read()
                    .await
                    .views_presets
                    .iter()
                    .enumerate()
                {
                    let new_view = lq::res_allcommon_views::Views {
                        index: i as u32,
                        values: view.clone(),
                    };
                    msg.views.push(new_view);
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchAnnouncement" => {
                let mut msg = lq::ResAnnouncement::decode(msg_block.data.as_ref())?;
                msg.announcements.insert(
                    0,
                    lq::Announcement {
                        title: "雀魂Max-rs载入成功".to_string(),
                        id: 1145141919,
                        header_image: "internal://2.jpg".to_string(),
                        content: ANNOUNCEMENT.to_string(),
                    },
                );
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchInfo" => {
                let mut msg = lq::ResFetchInfo::decode(msg_block.data.as_ref())?;
                if let Some(ref mut char_info) = msg.character_info {
                    self.safe.write().await.main_character_id = char_info.main_character_id;
                    char_info
                        .characters
                        .clone_into(&mut self.safe.write().await.characters);
                    char_info.characters.clear();
                    for charid in self.characters.iter().map(|c| c.id) {
                        let character = self.perfect_character(charid).await;
                        char_info.characters.push(character);
                    }
                    char_info.skins.clear();
                    char_info.skins.extend(self.skins.iter().map(|s| s.id));
                    char_info.main_character_id = self.mod_settings.read().await.main_char;
                    char_info.character_sort.clear();
                    char_info
                        .character_sort
                        .extend(self.mod_settings.read().await.star_character.iter());
                    char_info.hidden_characters.clear();
                    char_info.finished_endings.clear();
                    char_info.rewarded_endings.clear();
                    char_info
                        .finished_endings
                        .extend(self.endings.iter().map(|e| e.id));
                    char_info
                        .rewarded_endings
                        .extend(self.endings.iter().map(|e| e.id));
                }
                if let Some(ref mut bag_info) = msg.bag_info {
                    if let Some(ref mut bag) = bag_info.bag {
                        bag.items.clear();
                        self.fill_bag(bag).await;
                    }
                }
                if let Some(ref mut views) = msg.all_common_views {
                    views.views.clear();
                    views.r#use = self.mod_settings.read().await.preset_index;
                    for (i, view) in self
                        .mod_settings
                        .read()
                        .await
                        .views_presets
                        .iter()
                        .enumerate()
                    {
                        let new_view = lq::res_allcommon_views::Views {
                            index: i as u32,
                            values: view.clone(),
                        };
                        views.views.push(new_view);
                    }
                }
                msg.title_list = Some(lq::ResTitleList {
                    title_list: self.titles.iter().map(|t| t.id).collect(),
                    ..Default::default()
                });
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.Lobby.fetchServerSettings" => {
                let mut msg = lq::ResServerSettings::decode(msg_block.data.as_ref())?;
                if self.mod_settings.read().await.anti_nickname_censorship() {
                    if let Some(ref mut settings) = msg.settings {
                        if let Some(ref mut nick_setting) = settings.nickname_setting {
                            nick_setting.enable = 0;
                            nick_setting.nicknames.clear();
                            modified_data = Some(msg.encode_to_vec());
                        }
                    }
                }
            }
            ".lq.Lobby.fetchGameRecord" => {
                let msg = lq::ResGameRecord::decode(msg_block.data.as_ref())?;
                if let Some(head) = msg.head.as_ref() {
                    let uuid = head.uuid.as_str();
                    const LOG_HEAD: &str = "发现读入牌谱！\n";
                    const LOG_TAIL: &str = "注意：只有在同一服务器才能添加好友！";
                    let mut logs = String::new();
                    for acc in &head.accounts {
                        if acc.account_id == self.safe.read().await.account_id {
                            logs += "（自己）";
                        }
                        logs += &format!(
                            "{}\n账号id: {}\t加好友id: {}\n主视角牌谱链接: {uuid}_a{}\n主视角牌谱链接(匿名): {}_a{}_2\n\n",
                            add_zone_id(acc.account_id, &acc.nickname),
                            acc.account_id,
                            encode_account_id2(acc.account_id),
                            encode_account_id(acc.account_id),
                            encode_uuid(uuid),
                            encode_account_id(acc.account_id),
                        );
                    }
                    info!("{LOG_HEAD}{logs}{LOG_TAIL}");
                }
            }
            _ => {}
        }
        if let Some(data) = modified_data {
            msg_block.data = data;
            let mut buf = buf[..3].to_vec();
            buf.extend(msg_block.encode_to_vec());
            Ok(ModifyResult {
                msg: Some(buf.into()),
                inject_msg: None,
            })
        } else {
            Ok(ModifyResult {
                msg: Some(buf.to_owned()),
                inject_msg: None,
            })
        }
    }

    async fn fill_bag(&self, bag: &mut lq::Bag) {
        for item in self.safe.read().await.items.iter() {
            if !self.items.iter().any(|i| i.id == item.item_id) {
                let new_item = lq::Item {
                    item_id: item.item_id,
                    stack: item.stack,
                };
                bag.items.push(new_item);
            }
        }
        for item in self.items.iter() {
            let new_item = lq::Item {
                item_id: item.id,
                stack: 1,
            };
            bag.items.push(new_item);
        }
        for item in self.loading_images.iter() {
            let new_item = lq::Item {
                item_id: item.id,
                stack: 1,
            };
            bag.items.push(new_item);
        }
    }

    async fn change_player(&self, p: &mut lq::PlayerGameView) {
        if let Some(ref mut character) = p.character {
            character.is_upgraded = true;
            character.level = 5;
            if p.account_id == self.safe.read().await.account_id {
                character.charid = self.mod_settings.read().await.main_char;
                *character = self.perfect_character(character.charid).await;
                p.avatar_id = self.mod_settings.read().await.char_skin
                    [&self.mod_settings.read().await.main_char];
                if !self.mod_settings.read().await.nickname.is_empty() {
                    p.nickname
                        .clone_from(&self.mod_settings.read().await.nickname);
                }
                p.title = self.mod_settings.read().await.title;
                p.views.clear();
                p.views.extend(
                    self.mod_settings.read().await.views_presets
                        [self.mod_settings.read().await.preset_index as usize]
                        .clone(),
                );
                // avatar_frame id is view.item_id which view.slot is 5
                if let Some(frame) = p.views.iter().find(|v| v.slot == 5) {
                    p.avatar_frame = frame.item_id;
                }
                p.verified = self.mod_settings.read().await.verified;
            }
        }
        if self.mod_settings.read().await.show_server() {
            p.nickname = add_zone_id(p.account_id, &p.nickname);
        }
    }

    async fn perfect_character(&self, id: u32) -> lq::Character {
        let mut character = lq::Character {
            charid: id,
            exp: 0,
            is_upgraded: true,
            level: 5,
            ..Default::default()
        };
        character.rewarded_level.extend(vec![1, 2, 3, 4, 5]);
        character.skin = *self
            .mod_settings
            .write()
            .await
            .char_skin
            .entry(id)
            .or_insert(400001 + (id % 100) * 100);
        if self.mod_settings.read().await.emoji_on() {
            character
                .extra_emoji
                .extend(self.emojis.get(&id).unwrap_or(&vec![]))
        }
        character.views.clear();
        character.views.extend(
            self.mod_settings.read().await.views_presets
                [self.mod_settings.read().await.preset_index as usize]
                .clone(),
        );
        character
    }

    async fn modify_req(&self, buf: Bytes, from_client: bool) -> Result<ModifyResult> {
        let msg_id = u16::from_le_bytes([buf[1], buf[2]]) as usize;
        let mut msg_block = BaseMessage::decode(&buf[3..])?;
        // Request message must be from client
        assert!(from_client);
        if msg_id >= 1 << 16 {
            bail!("Invalid request message id: {msg_id}");
        }
        let mut fake = false;
        let method_name = &msg_block.method_name;
        let mut inject_data: Option<Vec<u8>> = None;
        match method_name.as_str() {
            ".lq.Lobby.changeMainCharacter" => {
                fake = true;
                let msg = lq::ReqChangeMainCharacter::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.main_char = msg.character_id;
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.changeCharacterSkin" => {
                fake = true;
                let msg = lq::ReqChangeCharacterSkin::decode(msg_block.data.as_ref())?;
                self.mod_settings
                    .write()
                    .await
                    .char_skin
                    .insert(msg.character_id, msg.skin);
                let character = self.perfect_character(msg.character_id).await;
                let mut character_update = lq::account_update::CharacterUpdate::default();
                character_update.characters.push(character);
                let account_update = lq::AccountUpdate {
                    character: Some(character_update),
                    ..Default::default()
                };
                let update_data = lq::NotifyAccountUpdate {
                    update: Some(account_update),
                };
                let blocks = vec![
                    Block::String(1, ".lq.NotifyAccountUpdate".into()),
                    Block::String(2, update_data.encode_to_vec().into()),
                ];
                let mut inject_buf = vec![0x01];
                inject_buf.extend(blocks_to_pb(blocks));
                inject_data = Some(inject_buf);
            }
            ".lq.Lobby.addFinishedEnding" => {
                // drop
                return Ok(ModifyResult {
                    msg: None,
                    inject_msg: None,
                });
            }
            ".lq.Lobby.updateCharacterSort" => {
                fake = true;
                let msg = lq::ReqUpdateCharacterSort::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.star_character = msg.sort;
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.useTitle" => {
                fake = true;
                let msg = lq::ReqUseTitle::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.title = msg.title;
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.setLoadingImage" => {
                fake = true;
                let msg = lq::ReqSetLoadingImage::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.loading_bg = msg.images;
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.saveCommonViews" => {
                fake = true;
                let msg = lq::ReqSaveCommonViews::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.views_presets[msg.save_index as usize] = msg.views;
                if msg.is_use == 1 {
                    self.mod_settings.write().await.preset_index = msg.save_index;
                }
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.useCommonView" => {
                let msg = lq::ReqUseCommonView::decode(msg_block.data.as_ref())?;
                self.mod_settings.write().await.preset_index = msg.index;
                self.mod_settings.read().await.write();
            }
            ".lq.Lobby.loginBeat" => {
                let msg = lq::ReqLoginBeat::decode(msg_block.data.as_ref())?;
                *self.contract.write().await = msg.contract;
            }
            ".lq.Lobby.readAnnouncement" => {
                let msg = lq::ReqReadAnnouncement::decode(msg_block.data.as_ref())?;
                if msg.announcement_id == 1145141919 {
                    fake = true;
                }
            }
            ".lq.Lobby.receiveCharacterRewards" => {
                fake = true;
            }
            _ => {}
        }
        if fake {
            let data = lq::ReqLoginBeat {
                contract: self.contract.read().await.clone(),
            };
            msg_block.method_name = ".lq.Lobby.loginBeat".to_string();
            msg_block.data = data.encode_to_vec();
            let mut buf = buf[..3].to_vec();
            buf.extend(msg_block.encode_to_vec());
            Ok(ModifyResult {
                msg: Some(buf.into()),
                inject_msg: inject_data.map(|d| d.into()),
            })
        } else {
            // return original message
            Ok(ModifyResult {
                msg: Some(buf.to_owned()),
                inject_msg: inject_data.map(|d| d.into()),
            })
        }
    }

    async fn modify_notify(&self, buf: Bytes) -> Result<ModifyResult> {
        let mut msg_block = BaseMessage::decode(&buf[1..])?;
        let method_name = &msg_block.method_name;
        let mut modified_data: Option<Vec<u8>> = None;
        match method_name.as_str() {
            ".lq.NotifyAccountUpdate" => {
                let msg = lq::NotifyAccountUpdate::decode(msg_block.data.as_ref())?;
                if let Some(ref update) = msg.update {
                    if update.character.is_some() {
                        // drop message if character is updated
                        return Ok(ModifyResult {
                            msg: None,
                            inject_msg: None,
                        });
                    }
                }
            }
            ".lq.NotifyRoomPlayerUpdate" => {
                let mut msg = lq::NotifyRoomPlayerUpdate::decode(msg_block.data.as_ref())?;
                for player in msg.player_list.iter_mut().chain(msg.update_list.iter_mut()) {
                    if player.account_id == self.safe.read().await.account_id {
                        player.avatar_id = self.mod_settings.read().await.char_skin
                            [&self.mod_settings.read().await.main_char];
                        if !self.mod_settings.read().await.nickname.is_empty() {
                            self.mod_settings
                                .read()
                                .await
                                .nickname
                                .to_owned()
                                .clone_into(&mut player.nickname);
                        }
                        player.title = self.mod_settings.read().await.title;
                    }
                    if self.mod_settings.read().await.show_server() {
                        player.nickname = add_zone_id(player.account_id, &player.nickname);
                    }
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.NotifyGameFinishRewardV2" => {
                let mut msg = Box::new(lq::NotifyGameFinishRewardV2::decode(
                    msg_block.data.as_ref(),
                )?);
                let main = self.safe.read().await.main_character_id;
                for char in self.safe.write().await.characters.iter_mut() {
                    if char.charid == main {
                        if let Some(ref main_char) = msg.main_character {
                            char.exp = main_char.exp;
                            char.level = main_char.level;
                        }
                        break;
                    }
                }
                if let Some(ref mut main_char) = msg.main_character {
                    main_char.add = 0;
                    main_char.exp = 0;
                    main_char.level = 5;
                }
                modified_data = Some(msg.encode_to_vec());
            }
            ".lq.NotifyCustomContestSystemMsg" => {
                if self.mod_settings.read().await.show_server() {
                    let mut msg =
                        lq::NotifyCustomContestSystemMsg::decode(msg_block.data.as_ref())?;
                    if let Some(ref mut game) = msg.game_start {
                        game.players.iter_mut().for_each(|p| {
                            p.nickname = add_zone_id(p.account_id, &p.nickname);
                        });
                        modified_data = Some(msg.encode_to_vec());
                    }
                }
            }
            _ => {}
        }
        if let Some(data) = modified_data {
            info!("Notify method: {method_name}");
            // add 0x01 to the beginning of the message
            msg_block.data = data;
            let mut buf = vec![0x01];
            buf.extend(msg_block.encode_to_vec());
            Ok(ModifyResult {
                msg: Some(buf.into()),
                inject_msg: None,
            })
        } else {
            Ok(ModifyResult {
                msg: Some(buf),
                inject_msg: None,
            })
        }
    }
}

fn add_zone_id(id: u32, name: &str) -> String {
    const CN: &str = "[C\u{feff}N]";
    let zone_code = id >> 23;
    let zone = match zone_code {
        code if code <= 6 => CN,
        code if (7..=12).contains(&code) => "[JP]",
        code if (13..=15).contains(&code) => "[EN]",
        _ => "[??]",
    }
    .to_string();
    zone + name
}

fn encode_uuid(uuid: &str) -> String {
    let mut buf = "".to_string();
    const CODE_0: u32 = '0' as u32;
    const CODE_A: u32 = 'a' as u32;
    for (i, c) in uuid.chars().enumerate() {
        let code = c as u32;
        let mut tmp = 0xFF;
        if (CODE_0..CODE_0 + 10).contains(&code) {
            tmp = code - CODE_0;
        } else if (CODE_A..CODE_A + 26).contains(&code) {
            tmp = code - CODE_A + 10;
        }
        if tmp != 0xFF {
            tmp = (tmp + 17 + i as u32) % 36;
            if tmp < 10 {
                buf.push((CODE_0 + tmp) as u8 as char);
            } else {
                buf.push((CODE_A + tmp - 10) as u8 as char);
            }
        } else {
            buf.push(c);
        }
    }
    buf
}

fn encode_account_id(id: u32) -> u32 {
    ((7 * id + 1117113) ^ 86216345) + 1358437
}

fn encode_account_id2(id: u32) -> u32 {
    let p = 6139246 ^ id;
    const H: u32 = 67108863;
    let s = p & !H;
    let mut z = p & H;
    for _ in 0..5 {
        z = (511 & z) << 17 | z >> 9;
    }
    z + s + 1e7 as u32
}

enum Block {
    _VarInt(u32, u64),
    String(u32, Bytes),
}

fn blocks_to_pb(blocks: Vec<Block>) -> Bytes {
    let mut pb = Vec::new();
    for block in blocks {
        match block {
            Block::_VarInt(id, data) => {
                // ((d['id'] << 3)+0).to_bytes(length=1, byteorder='little')
                let bytes = (id << 3).to_le_bytes();
                let byte = bytes[0];
                pb.push(byte);
                pb.extend(to_var_int(data));
            }
            Block::String(id, data) => {
                let bytes = ((id << 3) + 2).to_le_bytes();
                let byte = bytes[0];
                pb.push(byte);
                pb.extend(to_var_int(data.len() as u64));
                pb.extend(data);
            }
        }
    }
    pb.into()
}

fn to_var_int(mut x: u64) -> Bytes {
    if x == 0 {
        return Bytes::from_static(&[0]);
    }
    let mut data: u64 = 0;
    let mut base = 0;
    let mut length = 0;
    while x > 0 {
        length += 1;
        data += (x & 127) << base;
        x >>= 7;
        if x > 0 {
            data += 1 << (base + 7);
        }
        base += 8;
    }
    data.to_le_bytes()[..length].to_vec().into()
}
