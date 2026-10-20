//! The post office's own work (m_post_office, m_museum, mNpc_Remail):
//! letters handed in at the counter wait in its five-letter store and go
//! out at 09:00 and 17:00 (mPO_set_next_delivery_time), staying put while
//! the mailbox is full; villagers write back the day after a letter
//! (mNpc_Remail: a good letter gets a reply pieced together from the
//! disc's letter fragments and sometimes a present, a bad one a "wrong"
//! reply, a middling one none); the museum answers what was mailed to it
//! (mMsm_SendResultMail: up to three letters a day, identified fossils as
//! presents), sends its address the first day (mMsm_SendInformationMail)
//! and a gift when everything is on show (mMsm_SendCompMail).
//!
//! Letters land in the one letter store the pockets and the mailbox share.
use bevy::prelude::*;
use rustimal_logic::museum::{self as museum, Mailed};
use serde::{Deserialize, Serialize};

use crate::game_disc::GameDisc;
use crate::mail::{Letter, Mailbox, Recipient};
use crate::messages::MsgContext;

/// mPO_MAIL_STORAGE_SIZE.
pub const STORAGE: usize = 5;
/// The museum's name on its letters (l_museum_name_str).
pub const MUSEUM_NAME: &str = "Museum";
/// ITM_PAPER_START.
const PAPER_START: u16 = 0x2000;
/// ITM_FOSSIL.
const UNIDENTIFIED_FOSSIL: u16 = 0x2511;

/// A villager's reply owed (Anmmem_c.letter_info).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reply {
    pub villager: String,
    pub looks: u8,
    /// mNpc_LETTER_RANK_GOOD (else WRONG).
    pub good: bool,
    /// The day the letter arrived; the reply goes out on a later one.
    pub day: i64,
}

/// mMsm_record_c.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MuseumRecord {
    /// The museum's address is known (its first letter arrived).
    pub contacted: bool,
    /// Unidentified fossils mailed in, waiting to be looked at.
    pub stored_fossils: u8,
    /// Things mailed in: (mMsm_REMAIL_KIND, item).
    pub remail: Vec<(u8, u16)>,
    /// The letter for a finished museum went out.
    pub complete_sent: bool,
}

/// What the post office keeps between days.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Post {
    /// PostOffice_c.mail: letters waiting to go out.
    pub store: Vec<Letter>,
    /// The next delivery, as day_number * 24 + hour.
    pub next_delivery: Option<i64>,
    pub replies: Vec<Reply>,
    pub museum: MuseumRecord,
    /// The last day the daily work was done.
    pub day: Option<i64>,
}

impl Post {
    pub fn desk_full(&self) -> bool {
        self.store.len() >= STORAGE
    }

    /// mPO_receipt_proc for a letter the player handed in: a villager takes
    /// it to heart (mNpc_SendMailtoNpc), the museum reads it
    /// (mMsm_SendMuseumMail).
    pub fn receive(
        &mut self,
        letter: Letter,
        looks: Option<u8>,
        collection: &mut museum::Collection,
    ) -> Option<i32> {
        let today = crate::clock::now().day_number;
        match letter.to.as_ref()? {
            Recipient::Villager(name) => {
                let rank = rustimal_logic::letter_score::score_letter(
                    &crate::mail::letter_body_bytes(&letter.body),
                    rustimal_logic::letter_score::TrigramMode::Intended,
                )
                .total();
                let rank = rustimal_logic::letter_score::LetterRank::of_score(rank);
                use rustimal_logic::letter_score::LetterRank;
                // mNpc_SetMailCondThisLand: GOOD and WRONG are answered.
                let good = match rank {
                    LetterRank::Ok => Some(true),
                    LetterRank::Bad => Some(false),
                    LetterRank::Middle => None,
                };
                if let Some(good) = good {
                    self.replies.retain(|r| r.villager != *name);
                    self.replies.push(Reply {
                        villager: name.clone(),
                        looks: looks.unwrap_or(0),
                        good,
                        day: today,
                    });
                }
                // mNpc_SendMailtoNpc: +3, -5 for a bad letter, +3 a present.
                let mut friendship = 3;
                if rank == LetterRank::Bad {
                    friendship -= 5;
                }
                if letter.present != 0 {
                    friendship += 3;
                }
                info!("{name} got a letter ({rank:?})");
                Some(friendship)
            }
            Recipient::Museum => {
                match letter.present {
                    0 => {}
                    UNIDENTIFIED_FOSSIL => {
                        self.museum.stored_fossils =
                            (self.museum.stored_fossils + 1).min(museum::REMAIL_SLOTS as u8);
                    }
                    item => {
                        let kind = collection.mailed(item, 1);
                        if self.museum.remail.len() < museum::REMAIL_SLOTS {
                            self.museum.remail.push((kind as u8, item));
                        }
                    }
                }
                info!("the museum got a letter ({:#06x})", letter.present);
                None
            }
        }
    }
}

/// A letter from the disc's text (mMl_send_mail's mail_no).
pub fn disc_letter(
    disc: &GameDisc,
    mail_no: usize,
    ctx: &MsgContext,
    sender: &str,
    paper: u8,
    present: u16,
) -> Option<Letter> {
    let text = disc.text.as_ref()?.letter(mail_no, ctx)?;
    Some(Letter {
        sender: sender.to_string(),
        header: Some(text.header),
        body: text.body,
        footer: Some(text.footer),
        paper,
        present,
        ..default()
    })
}

fn letter_context(identity: &crate::save::PlayerIdentity) -> MsgContext {
    let mut free = vec![String::new(); 20];
    free[0] = identity.town.clone();
    free[1] = identity.player.clone();
    MsgContext {
        player_name: identity.player.clone(),
        free,
        items: vec![String::new(); 5],
        item_articles: vec![0; 5],
        ..default()
    }
}

/// xorshift for the post (RANDOM).
fn rand(seed: &mut u32, n: u32) -> u32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    *seed % n.max(1)
}

/// mSP_SelectRandomItem_New(mSP_KIND_PAPER): a paper's stationery index.
fn random_paper(seed: &mut u32) -> u8 {
    let roll = crate::goods::roll_now();
    let paper = crate::goods::select(&roll, crate::goods::Kind::Paper, 1, &[], &mut |n| {
        rand(seed, n)
    })
    .first()
    .copied()
    .unwrap_or(PAPER_START);
    (paper.wrapping_sub(PAPER_START) % 64) as u8
}

/// mNpc_GetRemailGoodData / mNpc_GetRemailWrongData (this town).
fn reply_letter(
    disc: &GameDisc,
    reply: &Reply,
    identity: &crate::save::PlayerIdentity,
    other_villager: &str,
    seed: &mut u32,
) -> Option<Letter> {
    let text = disc.text.as_ref()?;
    let mut ctx = letter_context(identity);
    // mNpc_SetRemailFreeString: player, villager, a neighbour, then words
    // for food, sports, hobbies, fish, insects, tastes, feelings, music,
    // satisfaction, praise and complaint.
    ctx.free[0] = identity.player.clone();
    ctx.free[1] = reply.villager.clone();
    ctx.free[2] = other_villager.to_string();
    const WORDS: [(usize, u32); 11] = [
        (0x314, 32),
        (0x334, 32),
        (0x2F4, 32),
        (0x6A1, 40),
        (0x679, 40),
        (0x354, 32),
        (0x374, 32),
        (0x394, 32),
        (0x3D4, 32),
        (0x3F4, 32),
        (0x3B4, 32),
    ];
    for (i, (base, n)) in WORDS.iter().enumerate() {
        ctx.free[3 + i] = text
            .string(base + rand(seed, *n) as usize)
            .unwrap_or_default();
    }
    let looks = (reply.looks as usize).min(5);
    let (letter, present) = if reply.good {
        const START: [usize; 6] = [0x020, 0x040, 0x000, 0x060, 0x080, 0x0A0];
        let base = START[looks];
        // give_present = RANDOM(4) & 1; 0 means a present.
        let give = rand(seed, 4) & 1;
        let present = if give == 0 {
            let roll = crate::goods::roll_now();
            let kind = if rand(seed, 4) & 1 == 0 {
                crate::goods::Kind::Furniture
            } else {
                crate::goods::Kind::Cloth
            };
            crate::goods::select(&roll, kind, 1, &[], &mut |n| rand(seed, n))
                .first()
                .copied()
                .unwrap_or(0)
        } else {
            0
        };
        let pieces = (
            base + rand(seed, 32) as usize,
            base + rand(seed, 32) as usize,
            base + rand(seed, 16) as usize + give as usize * 16,
            base + rand(seed, 32) as usize,
            base + rand(seed, 32) as usize,
        );
        (text.letter_z(pieces, &ctx)?, present)
    } else {
        let no = 0xC5 + looks * 3 + rand(seed, 3) as usize;
        (text.letter(no, &ctx)?, 0)
    };
    Some(Letter {
        sender: reply.villager.clone(),
        header: Some(letter.header),
        body: letter.body,
        footer: Some(letter.footer),
        paper: random_paper(seed),
        present,
        ..default()
    })
}

/// mSP_RandomOneFossilSelect: a single fossil or a skeleton part, from the
/// furniture birth table (mRmTp_birth_type 12 / 13).
fn random_fossil(disc: &GameDisc, seed: &mut u32) -> u16 {
    let kind = if rand(seed, 2) == 0 { 12 } else { 13 };
    let table = disc
        .catalog
        .get("mRmTp_birth_type")
        .and_then(|a| disc.rom.slice(disc::RomSource::Rel, a.offset, a.size));
    let pool: Vec<u16> = table
        .map(|t| {
            t.iter()
                .enumerate()
                .filter(|&(i, &b)| b == kind && i < 0x400)
                .map(|(i, _)| 0x1000 + i as u16 * 4)
                .collect()
        })
        .unwrap_or_default();
    if pool.is_empty() {
        return museum::FOSSIL_FIRST + 4;
    }
    pool[rand(seed, pool.len() as u32) as usize]
}

/// mMl_send_mail: the mailbox if it has room, else the post office.
fn send(letter: Letter, mailbox: &mut Mailbox, post: &mut Post) -> bool {
    if mailbox.has_room() {
        mailbox.push(letter);
        true
    } else if !post.desk_full() {
        post.store.push(letter);
        true
    } else {
        false
    }
}

/// The day's work: replies, the museum's letters (m_all_grow /
/// mNpc_Remail at the first visit of a new day).
#[allow(clippy::too_many_arguments)]
fn daily(
    disc: Option<Res<GameDisc>>,
    mut post: ResMut<Post>,
    mut mailbox: ResMut<Mailbox>,
    identity: Res<crate::save::PlayerIdentity>,
    collection: Res<crate::museum::MuseumCollection>,
    villagers: Query<&crate::npc::Villager>,
    mut seed: Local<u32>,
) {
    let Some(disc) = disc.filter(|d| d.text.is_some()) else {
        return;
    };
    let today = crate::clock::now().day_number;
    if post.day == Some(today) {
        return;
    }
    post.day = Some(today);
    if *seed == 0 {
        *seed = (today as u32).wrapping_mul(0x9E37_79B9) | 1;
    }
    let names: Vec<&str> = villagers.iter().map(|v| v.name).collect();

    // mNpc_Remail: replies to letters from an earlier day, through the
    // post office while it has room.
    let due: Vec<Reply> = post
        .replies
        .iter()
        .filter(|r| r.day < today)
        .cloned()
        .collect();
    for reply in due {
        if post.desk_full() {
            break;
        }
        let others: Vec<&&str> = names.iter().filter(|n| **n != reply.villager).collect();
        let other = others
            .get(rand(&mut seed, others.len().max(1) as u32) as usize)
            .map_or("", |n| **n);
        if let Some(letter) = reply_letter(&disc, &reply, &identity, other, &mut seed) {
            info!("{} writes back", reply.villager);
            post.store.push(letter);
        }
        post.replies.retain(|r| r.villager != reply.villager);
    }

    let mut ctx = letter_context(&identity);
    let paper = museum::MUSEUM_PAPER;
    // mMsm_SendResultMail: answers, then identified fossils, three a day.
    if post.museum.contacted {
        let mut sent = 0;
        while sent < museum::MAX_MAIL && !post.museum.remail.is_empty() {
            let (kind, item) = post.museum.remail[0];
            let Some(kind) = Mailed::from_u8(kind) else {
                post.museum.remail.remove(0);
                continue;
            };
            ctx.free[1] = crate::items::name(item);
            let present = if kind == Mailed::Donated { 0 } else { item };
            let Some(letter) =
                disc_letter(&disc, kind.mail_no(), &ctx, MUSEUM_NAME, paper, present)
            else {
                break;
            };
            if !send(letter, &mut mailbox, &mut post) {
                break;
            }
            post.museum.remail.remove(0);
            sent += 1;
        }
        while sent < museum::MAX_MAIL && post.museum.stored_fossils > 0 {
            let fossil = random_fossil(&disc, &mut seed);
            ctx.free[1] = crate::items::name(fossil);
            let no = museum::fossil_mail_no(fossil);
            let Some(letter) = disc_letter(&disc, no, &ctx, MUSEUM_NAME, paper, fossil) else {
                break;
            };
            if !send(letter, &mut mailbox, &mut post) {
                break;
            }
            post.museum.stored_fossils -= 1;
            sent += 1;
            info!("the museum identified a fossil: {fossil:#06x}");
        }
    }
    // mMsm_SendInformationMail: the museum's address, straight to the box.
    if !post.museum.contacted && mailbox.has_room() {
        if let Some(letter) = disc_letter(&disc, museum::INFO_MAIL_NO, &ctx, MUSEUM_NAME, paper, 0)
        {
            mailbox.push(letter);
            post.museum.contacted = true;
            info!("the museum wrote with its address");
        }
    }
    // mMsm_SendCompMail: the gift for a finished museum.
    if !post.museum.complete_sent && collection.0.count() == museum::TOTAL && !post.desk_full() {
        if let Some(letter) = disc_letter(
            &disc,
            museum::COMPLETE_MAIL_NO,
            &ctx,
            MUSEUM_NAME,
            paper,
            museum::MUSEUM_MODEL,
        ) {
            post.store.push(letter);
            post.museum.complete_sent = true;
        }
    }
}

/// mPO_business_proc / mPO_delivery_proc: at 09:00 and 17:00 the store's
/// letters go to the mailbox, those that fit.
fn deliver(mut post: ResMut<Post>, mut mailbox: ResMut<Mailbox>) {
    let now = crate::clock::now();
    let key = now.day_number * 24 + i64::from(now.hour);
    let next_after = |hour: u8, day: i64| {
        let (h, tomorrow) = rustimal_logic::mail::next_delivery_time(hour);
        (day + i64::from(tomorrow)) * 24 + i64::from(h)
    };
    let Some(next) = post.next_delivery else {
        post.next_delivery = Some(next_after(now.hour, now.day_number));
        return;
    };
    if key < next {
        return;
    }
    let mut kept = Vec::new();
    for letter in std::mem::take(&mut post.store) {
        if mailbox.has_room() {
            mailbox.push(letter);
            info!("mail delivered");
        } else {
            kept.push(letter);
        }
    }
    post.store = kept;
    post.next_delivery = Some(next_after(now.hour, now.day_number));
}

fn load(loaded: Res<crate::save::LoadedSave>, mut post: ResMut<Post>) {
    if let Some(saved) = loaded.0.as_ref().and_then(|s| s.post.clone()) {
        *post = saved;
    }
}

pub struct PostPlugin;

impl Plugin for PostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Post>()
            .add_systems(Startup, load)
            .add_systems(
                Update,
                (daily, deliver).chain().in_set(crate::states::GameplaySet),
            );
    }
}
