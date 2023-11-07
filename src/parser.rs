use std::{
    fs,
    time::Duration
};

use crate::Reminder;

pub fn read_from_file() -> Vec<Reminder> {
    let mut reminders: Vec<Reminder> = vec![];
    let schedule_string: String =
        fs::read_to_string("schedule.rmdr").expect("Should have been able to read the file");
    let schedules: Vec<_> = schedule_string.lines().collect();

    for line in schedules {
        let char0 = line.chars().nth(0);
        if !(char0 == Some('/') || char0 == None) {
            // Skip empty and comment lines
            let line_vec: Vec<_> = line.split(':').collect();
            let line_vec_len = line_vec.len();
            if line_vec_len >= 2 {
                // Make sure each string has a duration associated with it
                let string = line_vec[0].to_string();
                let repeating = line.chars().last() != Some('N');
                let mut seconds: u32 = 0;
                let mut minutes: u32 = 0;
                let mut hours: u32 = 0;
                if line_vec_len == 2 {
                    // In seconds
                    seconds = sanitise_to_u32(line_vec[1]);
                } else if line_vec_len == 3 {
                    // In Minutes, Seconds
                    minutes = sanitise_to_u32(line_vec[1]);
                    seconds = sanitise_to_u32(line_vec[2]);
                } else if line_vec_len == 4 {
                    // In Hours, Minutes, Seconds
                    hours = sanitise_to_u32(line_vec[1]);
                    minutes = sanitise_to_u32(line_vec[2]);
                    seconds = sanitise_to_u32(line_vec[3]);
                }

                let every = seconds + (minutes * 60) + (hours * 60 * 60);
                if every > 0 {
                    println!("Reminder: {}, repeating: {}, every {}s", string, repeating, every);
                    let duration_until = Duration::from_secs(every.into());
                    reminders.push(Reminder {
                        string,
                        repeating,
                        triggered: false,
                        every,
                        duration_until,
                    });
                }
            }
        }
    }

    reminders
}

fn sanitise_to_u32(string: &str) -> u32 {
    let mut new_string = String::new();
    for ch in string.chars() {
        if ch.is_digit(10) {
            new_string.push(ch);
        }
    }
    new_string.parse::<u32>().unwrap()
}
