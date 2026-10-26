//! US m_kabu_manager.c: one daily price, three native weekly trends.

pub fn quantity(item: u16) -> Option<u32> {
    match item {
        0x2F00 => Some(10),
        0x2F01 => Some(50),
        0x2F02 => Some(100),
        0x2F03 => Some(0),
        _ => None,
    }
}

pub fn spoil(item: &mut u16) -> bool {
    if matches!(*item, 0x2F00..=0x2F02) {
        *item = 0x2F03;
        true
    } else {
        false
    }
}

/// Preserving Sunday's price mirrors Kabu_manager on its update Sunday.
pub fn schedule(
    previous: u8,
    sunday: Option<u16>,
    random: &mut dyn FnMut() -> f32,
) -> (u8, [u16; 7]) {
    let mut prices = [0; 7];
    prices[0] = sunday.unwrap_or_else(|| ((1.0 + (random() - 0.5) * 0.6) * 100.0) as u16);
    let odds = match previous {
        1 => [0.6, 0.2],
        2 => [0.6, 0.3],
        _ => [0.5, 0.3],
    };
    let mut roll = random();
    let mut trend = 2;
    for (i, odd) in odds.into_iter().enumerate() {
        if roll < odd {
            trend = i as u8;
            break;
        }
        roll -= odd;
    }
    let mut price = f32::from(prices[0]);
    if trend == 2 {
        for day in &mut prices[1..] {
            price *= 0.8 + 0.14999998 * random();
            *day = price as u16;
        }
    } else {
        let mut increase = 1.05;
        let mut decrease = 0.1;
        let minimum = price * 0.8;
        for day in &mut prices[1..] {
            if random() < decrease {
                price = (price * 0.7).min(minimum);
                decrease = (decrease * 0.3).min(0.1);
            } else {
                price *= increase;
                increase += 0.01;
                decrease += 0.14;
            }
            *day = price as u16;
        }
        if trend == 0 {
            prices[1 + (random() * 5.0) as usize] = (f32::from(prices[0]) * 8.0) as u16;
        }
    }
    (trend, prices)
}
