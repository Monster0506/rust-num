use rust_num::numbers;

#[numbers]
use rust_num::{
    twothousandtwentysix,
    minus_forty_two,
    one_hundred_twenty_three_million_four_hundred_fifty_six_thousand_seven_hundred_and_eighty_nine as big_n,
    three_hundred_undecillion,
    three_point_one_four_one_five_nine,
    zero_point_five,
    minus_two_point_seven_five,
};

mod calendar {
    use rust_num::numbers;

    #[numbers]
    use rust_num::{seven, twentyfour};

    pub fn days() -> u32 {
        seven()
    }

    pub fn hours() -> u32 {
        seven::<u32>() * twentyfour::<u32>()
    }
}

fn main() {
    let year: u32 = twothousandtwentysix();
    let cold: i64 = minus_forty_two();
    let big: u64 = big_n();
    let huge: u128 = three_hundred_undecillion();
    println!("year={year} cold={cold} big={big} huge={huge}");

    let pi: f64 = three_point_one_four_one_five_nine();
    let half: f32 = zero_point_five();
    let chill: f64 = minus_two_point_seven_five();
    println!("pi={pi} half={half} chill={chill}");

    println!("days={} hours={}", calendar::days(), calendar::hours());
}
