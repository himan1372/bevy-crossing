//! What a villager talks about, decided by the translated GameCube topic
//! tree in `rustimal_logic::talk_topics` (`aQMgr_talk_normal_select_talk`).
//!
//! From the decomp:
//! - Happy villagers draw from the KI tree {chat 40, weather/time 30, free
//!   gift 10, furniture 10, gift-for-bells 10}; other moods from
//!   {normal 70, game 30} -> normal-2 {letter 15, normal-3 35, trade 35,
//!   memory 15} / game {hint 40, event 60}, normal-3 {chat 49,
//!   weather/time 17, weather 17, season 17}. Weights are exact
//!   (`decide_idx_prob_table`).
//! - Eligibility gates (gift needs a free pocket; gift-for-bells also needs
//!   3,000 bells) and the verified fallbacks: a failed KI leaf becomes KI
//!   chat, a failed normal-2 leaf becomes normal-3.
//! - Message IDs are computed with the decomp formulas and logged, so a
//!   talk can be traced back to the original table.
//!
//! Ours: every line of text (the original script isn't used), the memory
//! topic (no memory system yet, so it falls back like any failed leaf),
//! and the first-job hint is skipped (the slice has no first job).

use rustimal_logic::talk_topics::{self as tt, base, TalkKind};

use crate::items::ItemId;

/// A topic the villager will talk about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    Chat,
    /// Time of day (KI or normal-3 weather/time leaf).
    TimeOfDay,
    /// Villager hands over an item (KI free item). Needs a free pocket.
    Gift,
    /// Talks about home furniture (KI furniture leaf).
    Furniture,
    /// Talks about saving up bells (KI free item + money leaf).
    Bells,
    /// Asks about a letter they sent you.
    Letter,
    /// Admires something in your pockets.
    Trade(TradeKind),
    Weather,
    Season,
    GameHint,
    Rumor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeKind {
    /// Wants a fish or bug you're carrying.
    Catch(ItemId),
    /// Likes a piece of furniture you're carrying.
    Furniture(ItemId),
}

/// What the topic tree needs to know about the world.
pub struct TalkContext {
    pub looks: usize,
    pub happy: bool,
    pub free_pocket: bool,
    pub bells: u32,
    pub has_letter_from_them: bool,
    /// A fish or bug in the pockets, if any.
    pub carried_catch: Option<ItemId>,
    /// A piece of furniture in the pockets, if any.
    pub carried_furniture: Option<ItemId>,
    /// 1-12.
    pub month: u8,
    pub hour: u8,
}

/// Time-of-day bucket for the weather/time formula and the lines.
pub fn time_kind(hour: u8) -> usize {
    match hour {
        5..=10 => 0,  // morning
        11..=16 => 1, // afternoon
        17..=20 => 2, // evening
        _ => 3,       // night
    }
}

/// Choose a topic and its decomp message ID. `rng(n)` returns 0..n.
pub fn choose(ctx: &TalkContext, rng: &mut dyn FnMut(u32) -> u32) -> (Topic, i32) {
    let looks = ctx.looks.min(tt::LOOKS_NUM - 1);
    let weather = 0; // clear: the slice has no weather yet
    match tt::select_talk_kind(false, ctx.happy) {
        TalkKind::FjHint | TalkKind::Ki => {
            let mut cat = tt::decide_idx_prob_table(&tt::KI_PROB, rng);
            let eligible = match cat {
                tt::ki::FREE_ITEM => ctx.free_pocket,
                tt::ki::FREE_ITEM_MONEY => {
                    tt::ki_free_item_money_ok(ctx.free_pocket, ctx.bells as i32)
                }
                _ => true,
            };
            if !eligible {
                cat = tt::ki_fallback();
            }
            match cat {
                tt::ki::WEATHER_TIME => (
                    Topic::TimeOfDay,
                    tt::weather_time_msg_no(
                        base::KI_WEATHER_TIME[looks],
                        time_kind(ctx.hour) as i32,
                        weather,
                        rng(2) as i32,
                    ),
                ),
                tt::ki::FREE_ITEM => (Topic::Gift, base::KI_FREE_ITEM[looks] + rng(3) as i32),
                tt::ki::FTR => (Topic::Furniture, base::KI_FTR[looks] + rng(3) as i32),
                tt::ki::FREE_ITEM_MONEY => (
                    Topic::Bells,
                    base::KI_FREE_ITEM_MONEY[looks] + rng(3) as i32,
                ),
                _ => (Topic::Chat, base::KI_NORMAL[looks] + rng(10) as i32),
            }
        }
        TalkKind::Normal => {
            if tt::decide_idx_prob_table(&tt::NORMAL_1_PROB, rng) == 1 {
                return if tt::decide_idx_prob_table(&tt::GAME_PROB, rng) == tt::game::HINT {
                    (Topic::GameHint, base::GAME_HINT[looks] + rng(5) as i32)
                } else {
                    (Topic::Rumor, base::EV_CAL[looks])
                };
            }
            let leaf = match tt::decide_idx_prob_table(&tt::NORMAL_2_PROB, rng) {
                tt::normal2::LETTER if ctx.has_letter_from_them => {
                    Some((Topic::Letter, tt::letter_msg_no(base::LETTER[looks], 0)))
                }
                tt::normal2::TRADE => trade(ctx, looks, rng),
                // Memory: no memory system yet -> fails like any leaf.
                _ => None,
            };
            leaf.unwrap_or_else(|| normal3(ctx, looks, weather, rng))
        }
    }
}

fn trade(ctx: &TalkContext, looks: usize, rng: &mut dyn FnMut(u32) -> u32) -> Option<(Topic, i32)> {
    match tt::decide_idx_prob_table(&tt::TRADE_PROB, rng) {
        tt::trade::ITEM => ctx.carried_catch.map(|it| {
            (
                Topic::Trade(TradeKind::Catch(it)),
                base::TRADE_ITEM[looks] + rng(5) as i32,
            )
        }),
        tt::trade::FTR => ctx.carried_furniture.map(|it| {
            (
                Topic::Trade(TradeKind::Furniture(it)),
                base::TRADE_FTR[looks] + rng(5) as i32,
            )
        }),
        // Free-item trades need a choice prompt; not in the slice yet.
        _ => None,
    }
}

fn normal3(
    ctx: &TalkContext,
    looks: usize,
    weather: i32,
    rng: &mut dyn FnMut(u32) -> u32,
) -> (Topic, i32) {
    match tt::decide_idx_prob_table(&tt::NORMAL_3_PROB, rng) {
        tt::normal3::WEATHER_TIME => (
            Topic::TimeOfDay,
            tt::weather_time_msg_no(
                base::NORMAL3_WEATHER_TIME[looks],
                time_kind(ctx.hour) as i32,
                weather,
                rng(2) as i32,
            ),
        ),
        tt::normal3::WEATHER => (
            Topic::Weather,
            tt::normal3_weather_msg_no(base::NORMAL3_WEATHER[looks], weather, false, rng(5) as i32),
        ),
        tt::normal3::SEASON => (
            Topic::Season,
            tt::normal3_season_msg_no(base::NORMAL3_SEASON[looks], ctx.month as i32),
        ),
        _ => (Topic::Chat, base::NORMAL3_NORMAL[looks] + rng(10) as i32),
    }
}

// ---------------------------------------------------------------- text
//
// Original lines, two per topic per personality (looks order: normal,
// peppy, lazy, jock, cranky, snooty). `{item}`, `{time}` and `{season}`
// are filled in by `lines`.

type Pool = [[&'static str; 2]; 6];

const CHAT: Pool = [
    [
        "I hung my laundry out this morning. It smells like sunshine!",
        "Do you ever just sit and listen to the river? It's so calming.",
    ],
    [
        "Guess what? I've been practicing a brand-new dance move!",
        "I wish every day could be a party day. Don't you?",
    ],
    [
        "I had the best sandwich earlier. Still thinking about it.",
        "Is it weird that I already want another nap?",
    ],
    [
        "I ran three laps around town before breakfast. Feeling great!",
        "You look like you could use some stretches. Trust me!",
    ],
    [
        "Hmph. Kids these days, always running around.",
        "I've lived here a long time. Seen a lot of folks come and go.",
    ],
    [
        "One must always look one's best. Even on a quiet day.",
        "I redecorated again. A little elegance never hurts.",
    ],
];

const TIME_OF_DAY: Pool = [
    [
        "Isn't the {time} lovely? I always feel so cheerful now.",
        "This {time} is perfect for tidying the garden.",
    ],
    [
        "This {time} is SO good for being out and about!",
        "Ooh, I love the {time}! Everything feels sparkly!",
    ],
    [
        "The {time}, huh? Feels like snack time to me.",
        "I could nap right through this whole {time}.",
    ],
    [
        "Perfect {time} for a workout! Let's go!",
        "Nothing beats a jog in the {time}!",
    ],
    [
        "The {time} again. Keep the noise down, would ya?",
        "I do my best thinking in the {time}. Don't interrupt.",
    ],
    [
        "The light this {time} is rather flattering, isn't it?",
        "I take my tea every {time}. It's a refined habit.",
    ],
];

const GIFT: Pool = [
    [
        "I have something I don't really need. Would you like this {item}?",
        "Here, please take this {item}. I think you'll like it!",
    ],
    [
        "Surprise! I got you a {item}! Yay!",
        "This {item} is SO you. It's yours now!",
    ],
    [
        "I found this {item}... you can have it. Less to carry.",
        "Here, a {item}. Don't say I never gave you anything.",
    ],
    [
        "Take this {item}! Consider it a prize for hustling!",
        "You've earned this {item}, champ!",
    ],
    [
        "Here. Take this {item}. Don't make a fuss about it.",
        "I had a spare {item}. Just take it before I change my mind.",
    ],
    [
        "I suppose you may have this {item}. Do treasure it.",
        "This {item} deserves someone who'll appreciate it. Here.",
    ],
];

const FURNITURE: Pool = [
    [
        "I moved my furniture around yesterday. It feels like a new room!",
        "A cozy home makes everything better, don't you think?",
    ],
    [
        "My room is all pink now! Well, mostly!",
        "I want SO many cute things for my house!",
    ],
    [
        "My favorite furniture? The bed. Obviously.",
        "I put my couch right by the fridge. Genius, right?",
    ],
    [
        "I turned half my house into a gym!",
        "Who needs a sofa when you can have a bench press?",
    ],
    [
        "My chair is older than you. And comfier, too.",
        "Don't touch my stuff if you ever visit. Hmph.",
    ],
    [
        "Good furniture is an investment, darling.",
        "My home is a gallery of good taste.",
    ],
];

const BELLS: Pool = [
    [
        "I'm saving up bells for something special. It's a secret!",
        "Do you keep your bells somewhere safe? I worry about mine.",
    ],
    [
        "I'm gonna save up and buy the cutest thing EVER!",
        "Bells, bells, bells! I want to buy everything!",
    ],
    [
        "I keep bells under my pillow. Is that weird?",
        "Saving is hard when snacks exist.",
    ],
    [
        "I'm saving for new gear! Top of the line!",
        "Bells come and go, but gains are forever!",
    ],
    [
        "Back in my day, a hundred bells went a long way.",
        "Don't waste your bells on junk. Take it from me.",
    ],
    [
        "One should never discuss money. But I have plenty.",
        "Quality costs bells, darling. Worth every one.",
    ],
];

const LETTER: Pool = [
    [
        "Did my letter reach you? I hope you liked it!",
        "Writing letters is so fun. Did you read mine?",
    ],
    [
        "Did you get my letter?! I used my best pen!",
        "My letter was super cute, right? RIGHT?",
    ],
    [
        "I wrote you a letter. Took forever. Did you read it?",
        "Did my letter show up? Writing made me hungry.",
    ],
    [
        "You got my letter, right? I wrote it between sets!",
        "Hope my letter pumped you up!",
    ],
    [
        "You got my letter? Good. Don't expect another soon.",
        "I don't write letters for just anybody, you know.",
    ],
    [
        "My letter was on my finest paper. I trust you noticed.",
        "I hope you savored my letter.",
    ],
];

const TRADE_CATCH: Pool = [
    [
        "Oh! Is that a {item}? I've always wanted to see one up close!",
        "What a nice {item}! You must be good at this.",
    ],
    [
        "A {item}?! That's AMAZING! Can I look?",
        "Wow, you caught a {item}! You're so cool!",
    ],
    [
        "Is that a {item}? Can you eat it? Just asking.",
        "A {item}, huh? Nice. Very nice.",
    ],
    [
        "Whoa, a {item}! That takes real skill!",
        "You caught a {item}? Respect!",
    ],
    [
        "Hmph. A {item}. I caught bigger ones in my day.",
        "Not bad, that {item}. Not bad at all.",
    ],
    [
        "A {item}? How... rustic. But rather impressive.",
        "I suppose that {item} is a decent catch.",
    ],
];

const TRADE_FURNITURE: Pool = [
    [
        "Is that a {item}? It would look lovely in a sunny room!",
        "Oh, a {item}! You have good taste.",
    ],
    [
        "A {item}! Ooh, that's adorable!",
        "I LOVE that {item}! Where'd you get it?",
    ],
    [
        "A {item}... could I nap on it?",
        "Nice {item}. Looks comfy.",
    ],
    [
        "A {item}! Bet that's great for a quick rest between workouts.",
        "Solid {item}. Built strong!",
    ],
    [
        "A {item}, eh? Hmph. They don't make 'em like they used to.",
        "Careful not to scratch that {item}.",
    ],
    [
        "A {item}? Hmm. Acceptable. Barely.",
        "That {item} has a certain charm, I'll admit.",
    ],
];

const WEATHER: Pool = [
    [
        "What a clear day! Perfect for drying flowers.",
        "Not a cloud in the sky. I love days like this.",
    ],
    [
        "The sky is SO blue today! It matches my mood!",
        "Sunshine! Sunshine! Best weather ever!",
    ],
    [
        "Nice weather for napping outside.",
        "Sunny days make me sleepy. Well, all days do.",
    ],
    [
        "Clear skies! Perfect for a run!",
        "Great weather for working up a sweat!",
    ],
    [
        "At least it's not raining. Small mercies.",
        "Weather's fine. Don't get used to it.",
    ],
    [
        "The sun is simply perfect for my complexion today.",
        "Clear weather suits me, naturally.",
    ],
];

const SEASON: Pool = [
    [
        "I love {season}. Everything feels just right.",
        "There's something special about {season}, isn't there?",
    ],
    [
        "{season} is my FAVORITE season! Okay, they all are!",
        "{season} outfits are the best!",
    ],
    [
        "{season} is good napping season. So is every season.",
        "What do people eat in {season}? Asking for me.",
    ],
    [
        "{season} training is the best training!",
        "Gotta stay in shape all {season} long!",
    ],
    [
        "{season} again. They come around faster every year.",
        "In my day, {season} lasted longer. I'm sure of it.",
    ],
    [
        "{season} fashion is something I take very seriously.",
        "{season} brings out my best side.",
    ],
];

const GAME_HINT: Pool = [
    [
        "Tip: hold Shift to run. It gets you around town so much faster!",
        "If your pockets fill up, you can sell fish and bugs at the shop.",
    ],
    [
        "Ooh, tip! Press I to peek in your pockets!",
        "Want bells? Catch bugs with a net and sell them! So easy!",
    ],
    [
        "Fish shadows in the river... cast near them with a rod. Then wait.",
        "Fruit lying around? Pick it up. Free snacks. Or bells.",
    ],
    [
        "Pro tip: run with Shift! Never walk when you can run!",
        "Bugs hang around trees. Get your net ready, champ!",
    ],
    [
        "You want advice? Check your mailbox once in a while.",
        "Hmph. If someone has a '!' over their head, they want something.",
    ],
    [
        "A word of advice: the shop pays more for fish than for fruit.",
        "Do help your neighbors with their errands. It's only polite.",
    ],
];

const RUMOR: Pool = [
    [
        "I heard the shop might get bigger if people buy enough there.",
        "Someone said there are rare fish in the river. I wonder which?",
    ],
    [
        "I heard a rumor that there's going to be a party! Maybe!",
        "Somebody told me a secret, but I forgot it! Oops!",
    ],
    [
        "I heard there's a snack nobody's tried yet. Could be a rumor.",
        "Word is, sleeping in is good for you. I believe it.",
    ],
    [
        "Rumor has it someone's training for a big race. Me!",
        "I heard there's a secret workout spot in town.",
    ],
    [
        "People talk too much. Heard that about everyone, too.",
        "I heard the shopkeeper's been extra busy. Hmph.",
    ],
    [
        "I heard someone in town has terrible taste. Not you, surely.",
        "One hears things. One simply doesn't repeat them.",
    ],
];

const GREETING: Pool = [
    ["Oh, hello there!", "Hi! It's nice to see you."],
    ["Hiiii! Yay, it's you!", "Oh! Hi hi hi!"],
    ["Oh... hey. What's up?", "Mmm? Oh, hi."],
    ["Hey hey! What's going on?", "Yo! Good to see ya!"],
    ["Hmph. Oh, it's you.", "What? Oh. Hello."],
    [
        "Oh, hello. How nice of you to stop by.",
        "Ah, it's you. Good day.",
    ],
];

const ANNOYED: [&str; 6] = [
    "Sorry, I'm a little tired of chatting right now. Maybe later?",
    "Ugh, we JUST talked! Give me a minute!",
    "Can we talk later? I'm... really tired.",
    "Dude, I gotta cool down first. Catch you later!",
    "Enough already! Go bother someone else.",
    "I believe we've talked quite enough for now.",
];

const MILDLY_ANNOYED: [&str; 6] = [
    "Well, I should get going soon.",
    "Okay, okay, I gotta go soon!",
    "I think I need a nap after all this talking.",
    "Gotta get back to my workout soon!",
    "Hmph. Are we done yet?",
    "Do try not to monopolize my time.",
];

fn season_name(month: u8) -> &'static str {
    match month {
        3..=5 => "spring",
        6..=8 => "summer",
        9..=11 => "fall",
        _ => "winter",
    }
}

fn time_name(hour: u8) -> &'static str {
    ["morning", "afternoon", "evening", "night"][time_kind(hour)]
}

/// Capitalize the first letter (for lines starting with `{season}`).
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// The lines for a topic. `item` names the gift or traded item; `variant`
/// picks one of the two lines.
pub fn lines(topic: Topic, ctx: &TalkContext, item: Option<&str>, variant: usize) -> String {
    let looks = ctx.looks.min(5);
    let pool = match topic {
        Topic::Chat => &CHAT,
        Topic::TimeOfDay => &TIME_OF_DAY,
        Topic::Gift => &GIFT,
        Topic::Furniture => &FURNITURE,
        Topic::Bells => &BELLS,
        Topic::Letter => &LETTER,
        Topic::Trade(TradeKind::Catch(_)) => &TRADE_CATCH,
        Topic::Trade(TradeKind::Furniture(_)) => &TRADE_FURNITURE,
        Topic::Weather => &WEATHER,
        Topic::Season => &SEASON,
        Topic::GameHint => &GAME_HINT,
        Topic::Rumor => &RUMOR,
    };
    let line = pool[looks][variant % 2]
        .replace("{item}", item.unwrap_or("thing"))
        .replace("{time}", time_name(ctx.hour))
        .replace("{season}", season_name(ctx.month));
    capitalize(&line)
}

/// Opening line.
pub fn greeting(looks: usize, variant: usize) -> &'static str {
    GREETING[looks.min(5)][variant % 2]
}

/// Closing line when the villager is getting tired of talking.
pub fn mildly_annoyed(looks: usize) -> &'static str {
    MILDLY_ANNOYED[looks.min(5)]
}

/// The whole reply when the villager refuses to talk.
pub fn annoyed(looks: usize) -> &'static str {
    ANNOYED[looks.min(5)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(happy: bool) -> TalkContext {
        TalkContext {
            looks: 0,
            happy,
            free_pocket: true,
            bells: 0,
            has_letter_from_them: false,
            carried_catch: None,
            carried_furniture: None,
            month: 4,
            hour: 9,
        }
    }

    /// Simple LCG so the tests are deterministic.
    fn lcg(seed: u32) -> impl FnMut(u32) -> u32 {
        let mut s = seed;
        move |n| {
            s = s.wrapping_mul(1103515245).wrapping_add(12345);
            (s >> 16) % n
        }
    }

    fn tally(c: &TalkContext, n: usize) -> Vec<Topic> {
        let mut rng = lcg(7);
        (0..n).map(|_| choose(c, &mut rng).0).collect()
    }

    #[test]
    fn happy_uses_ki_tree_with_gates() {
        let mut c = ctx(true);
        let topics = tally(&c, 4000);
        let share = |t: Topic| topics.iter().filter(|&&x| x == t).count() as f64 / 40.0;
        // Bells topic needs 3000 bells: falls back to chat (40 + 10 = 50%).
        assert!(!topics.contains(&Topic::Bells));
        assert!((share(Topic::Chat) - 50.0).abs() < 3.0);
        assert!((share(Topic::TimeOfDay) - 30.0).abs() < 3.0);
        assert!((share(Topic::Gift) - 10.0).abs() < 2.0);
        // Full pockets: no gifts either.
        c.free_pocket = false;
        assert!(!tally(&c, 2000).contains(&Topic::Gift));
    }

    #[test]
    fn normal_mood_never_uses_ki_only_topics() {
        let topics = tally(&ctx(false), 4000);
        assert!(!topics.contains(&Topic::Gift) && !topics.contains(&Topic::Furniture));
        // No letter and nothing tradeable: those leaves fall back to normal-3.
        assert!(!topics
            .iter()
            .any(|t| matches!(t, Topic::Letter | Topic::Trade(_))));
        // Game branch is 30%: hints 40% of it, rumors 60%.
        let share = |t: Topic| topics.iter().filter(|&&x| x == t).count() as f64 / 40.0;
        assert!((share(Topic::GameHint) - 12.0).abs() < 2.5);
        assert!((share(Topic::Rumor) - 18.0).abs() < 2.5);
    }

    #[test]
    fn letters_and_trades_appear_when_eligible() {
        let mut c = ctx(false);
        c.has_letter_from_them = true;
        c.carried_catch = Some(crate::items::FRUIT);
        let topics = tally(&c, 4000);
        assert!(topics.contains(&Topic::Letter));
        assert!(topics
            .iter()
            .any(|t| matches!(t, Topic::Trade(TradeKind::Catch(_)))));
    }

    #[test]
    fn lines_fill_placeholders() {
        let c = ctx(false);
        assert!(lines(Topic::TimeOfDay, &c, None, 0).contains("morning"));
        assert!(lines(Topic::Season, &c, None, 0).starts_with("I love spring"));
        assert!(lines(Topic::Gift, &c, Some("Bookshelf"), 1).contains("Bookshelf"));
        let peppy = TalkContext {
            looks: 1,
            ..ctx(false)
        };
        assert!(lines(Topic::Season, &peppy, None, 0).starts_with("Spring is my FAVORITE"));
    }
}
