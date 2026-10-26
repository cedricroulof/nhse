//! Prints the same values as the C# cross-validation harness for diffing.
use nhse_core::save::island::DesignKind;
use nhse_core::save::player::PlayerList;
use nhse_core::structs::design;
use nhse_core::HorizonSave;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let s = HorizonSave::open(std::path::Path::new(&args[1])).expect("open");
    let p = |k: &str, v: String| println!("{k}={v}");
    p("rev", s.revision.index.to_string());
    p("valid_sizes", "True".into());
    p("invalid_hashes", s.invalid_hashes().len().to_string());
    let is = s.island_settings();
    p("hemisphere", is.hemisphere.to_string());
    p("tour_hemisphere", is.tour_hemisphere.to_string());
    p("weather_seed", is.weather_seed.to_string());
    p("airport", is.airport_color.to_string());
    p("specialty_fruit", is.specialty_fruit.to_string());
    p("last_saved", is.last_saved.clone());
    let t = s.turnips();
    p("turnip_buy", t.buy_price.to_string());
    p("turnip_sell_mon_pm", t.sell[3].to_string());
    p("museum_level", s.museum().level.to_string());
    let b = s.buildings()[3];
    p(
        "building3",
        format!(
            "{},{},{},{},{},{}",
            b.kind, b.x, b.y, b.angle, b.type_, b.unique_id
        ),
    );
    p("acre5", s.acres()[5].to_string());
    let tt = s.terrain()[1234];
    p(
        "terrain1234",
        format!(
            "{},{},{},{},{}",
            tt.model, tt.variation, tt.angle, tt.road, tt.elevation
        ),
    );
    p(
        "field0_5",
        format!("{:016X}", s.field_layer(0).tiles[5].raw()),
    );
    let dn = |k: DesignKind, i: usize| design::info(s.design_raw(k, i).unwrap(), i, false).name;
    p("design0", dn(DesignKind::Normal, 0));
    p("designpro3", dn(DesignKind::Pro, 3));
    p("recycle0", format!("{:016X}", s.recycle_bin()[0].raw()));
    p("visitor_wisp", s.visitors().day_wisp.to_string());
    let pl = s.plaza();
    p("plaza", format!("{},{}", pl.x, pl.z));
    p("land_flag_10", s.land_flags()[10].to_string());
    for v in s.villagers() {
        let m = &v.memories[0];
        p(
            &format!("villager{}", v.index),
            format!(
                "{},{},{},{},{},{}",
                v.internal_name,
                v.personality,
                v.catchphrase,
                v.move_type,
                m.friendship,
                m.player.player_name
            ),
        );
    }
    let vh = &s.villager_houses()[2];
    p(
        "vhouse2",
        format!(
            "{},{},{},{},{}",
            vh.exterior.level, vh.exterior.wall, vh.exterior.roof, vh.npc1, vh.npc2
        ),
    );
    let ph = &s.player_houses()[1];
    p(
        "phouse1",
        format!(
            "{},{},{},{}",
            ph.exterior.level, ph.exterior.wall, ph.exterior.roof, ph.exterior.door
        ),
    );
    for i in 0..s.players.len() {
        let pr = s.player_profile(i).unwrap();
        p(
            &format!("p{i}"),
            format!(
                "{},{},{:08X},{:08X},{},{},{},{}",
                pr.name,
                pr.town_name,
                pr.player_id,
                pr.town_id,
                pr.wallet,
                pr.bank,
                pr.nook_miles,
                pr.total_nook_miles
            ),
        );
        p(
            &format!("p{i}_counts"),
            format!("{},{},{}", pr.pocket_count, pr.bag_count, pr.storage_count),
        );
        let items = s.player_items(i, PlayerList::Pocket).unwrap();
        p(&format!("p{i}_pocket0"), format!("{:016X}", items[0].raw()));
        p(&format!("p{i}_bag3"), format!("{:016X}", items[23].raw()));
        let st = s.player_items(i, PlayerList::Storage).unwrap();
        p(&format!("p{i}_storage7"), format!("{:016X}", st[7].raw()));
        p(
            &format!("p{i}_hotel"),
            pr.hotel_tickets
                .map(|x| x.to_string())
                .unwrap_or("none".into()),
        );
        p(
            &format!("p{i}_poki"),
            pr.poki.map(|x| x.to_string()).unwrap_or("none".into()),
        );
        p(
            &format!("p{i}_birthday"),
            format!("{:02}-{:02}", pr.birthday.month, pr.birthday.day),
        );
        p(
            &format!("p{i}_flag5"),
            s.player_flags(i).unwrap()[5].to_string(),
        );
        let r = s.recipes(i).unwrap()[17];
        let tf = |b: bool| if b { "True" } else { "False" };
        p(
            &format!("p{i}_recipe17"),
            format!("{},{},{},{}", tf(r[0]), tf(r[1]), tf(r[2]), tf(r[3])),
        );
        let a = s.achievements(i).unwrap();
        p(
            &format!("p{i}_ach2"),
            format!(
                "{},{},{:04}-{:02}-{:02},{}",
                a.counts[2],
                tf(a.read[2][1]),
                a.dates[2][1].year,
                a.dates[2][1].month,
                a.dates[2][1].day,
                a.flags[2]
            ),
        );
        let re = s.reactions(i).unwrap();
        p(
            &format!("p{i}_react"),
            format!(
                "{},{},{},{}",
                re.learned[0],
                re.learned[1],
                re.wheel[0],
                tf(re.new_flags[0])
            ),
        );
        let m = &s.mails(i).unwrap()[3];
        p(
            &format!("p{i}_mail3"),
            format!(
                "{},{},{}",
                m.sender_name,
                m.message.encode_utf16().count(),
                m.stationery
            ),
        );
    }
}
