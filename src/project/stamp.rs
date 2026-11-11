pub fn date_time() -> (String, String) {
    let mut time = SystemTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milli: 0,
    };
    unsafe { GetLocalTime(&mut time) };
    let month = MONTHS.get(time.month.saturating_sub(1) as usize).copied().unwrap_or("January");
    (
        format!("{} {month} {}", time.day, time.year),
        format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second),
    )
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December",
];

#[repr(C)]
struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milli: u16,
}

unsafe extern "system" {
    fn GetLocalTime(time: *mut SystemTime);
}
