mod tests;

use std::{cmp, io::{self, StdoutLock}};
use std::{env};
use std::{
    collections::{HashMap, VecDeque},
    thread,
    time::{Duration, Instant},
};
use tokio::{
    fs
};

use termion::{self, raw::{IntoRawMode, RawTerminal}};
#[derive(Debug)]
enum UserInputTypes {
    Numeric(f64),
    Int(u64),
    IntSeries(VecDeque<u64>),
}

type UserInput = HashMap<String, UserInputTypes>;
async fn read_inputs() -> io::Result<UserInput> {
    let args: Vec<String> = env::args().collect();
    let mut buffer = String::new();
    let mut input: UserInput = HashMap::new();
    match args.get(1) {
        Some(val) if val == "-" => {
            let mut count = 0;
            loop {
                if count == 0 {
                    println!("Type in the building floor count");
                } else if count == 1 {
                    println!("Type in the building floor height");
                } else {
                    println!("Type in the floor requests seperated by comma e.g 3,5,6,2")
                }
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        buffer = buffer.trim().to_string();
                        if count == 0 {
                            input.insert(
                                "floor_count".to_string(),
                                UserInputTypes::Int(
                                    buffer.parse::<u64>().expect("Unable to parse inro a float"),
                                ),
                            );
                        } else if count == 1 {
                            input.insert(
                                "floor_height".to_string(),
                                UserInputTypes::Numeric(
                                    buffer.parse::<f64>().expect("Unable to parse inro a float"),
                                ),
                            );
                        } else {
                            input.insert("floor_requests".to_string(), UserInputTypes::IntSeries(buffer.split(",").into_iter().map(|val| {
                                val.parse::<u64>().expect("Expected all floor requests to be positive integers")
                            }).collect::<VecDeque<u64>>()));
                        }
                        count += 1;
                        buffer.clear();
                    }
                    Err(err) => panic!("Failed to read inputs because: {:?}", err),
                }
                if count > 2 {
                    break;
                }
            }
        }
        Some(path) => {
            buffer = fs::read_to_string(path).await?;
            let buffer: Vec<&str> = buffer.split("\n").collect();
            if buffer.len() != 3 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Invalid input from file, expected three lines got {}",
                        buffer.len()
                    ),
                ));
            }
            for (count, val) in buffer.into_iter().enumerate() {
                if count == 0 {
                    input.insert(
                        "floor_count".to_string(),
                        UserInputTypes::Int(
                            val.split("floor_count")
                                .collect::<Vec<&str>>()
                                .get(1)
                                .unwrap()
                                .trim()
                                .parse::<u64>()
                                .expect("Unable to parse into a float"),
                        ),
                    );
                } else if count == 1 {
                    input.insert(
                        "floor_height".to_string(),
                        UserInputTypes::Numeric(
                            val.split("floor_height")
                                .collect::<Vec<&str>>()
                                .get(1)
                                .unwrap()
                                .trim()
                                .parse::<f64>()
                                .expect("Unable to parse into a float"),
                        ),
                    );
                } else {
                    input.insert(
                        "floor_requests".to_string(),
                        UserInputTypes::IntSeries(
                            val.split("floor_requests")
                                .collect::<Vec<&str>>()
                                .get(1)
                                .unwrap()
                                .trim()
                                .split(",")
                                .into_iter()
                                .map(|val| {
                                    val.parse::<u64>().expect(
                                        "Expected all floor requests to be positive integers",
                                    )
                                })
                                .collect::<VecDeque<u64>>(),
                        ),
                    );
                }
            }
        }
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid input received, check file",
            ))
        }
    }
    Ok(input)
}
pub async fn run_simulation() {
    use std::io::Write;
    //Store location, veclocity and acceleration

    let mut input = match read_inputs().await {
        Ok(input) => input,
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    };

    let mut location = 0.0;
    let mut acceleration = 0.0;
    let mut velocity = 0.0;

    //Store voltage

    let mut up_input_voltage = 0.0;
    let mut down_input_voltage = 0.0;

    //Store input from user
    let mut floor_count = 0u64;
    let mut floor_height = 0.0;
    let mut floor_requests = &mut VecDeque::<u64>::new();
    if let UserInputTypes::Int(count) = *input.get("floor_count").unwrap() {
        floor_count = count;
    }
    if let UserInputTypes::Numeric(height) = *input.get("floor_height").unwrap() {
        floor_height = height;
    };
    if let UserInputTypes::IntSeries(requests) = input.get_mut("floor_requests").unwrap() {
        floor_requests = requests;
    };
    let mut floor_requests = floor_requests
        .iter()
        .filter(|val| **val <= floor_count)
        .map(|val| *val)
        .collect::<VecDeque<u64>>();
    let mut prev_loop_time = Instant::now();

    let termsize = termion::terminal_size().ok();
    let termwidth = termsize.map(|(w, _)| w - 2).expect("termwidth") as u64;
    let termheight = termsize.map(|(_, h)| h - 2).expect("termheight") as u64;
    let mut _stdout = io::stdout();
    let mut stdout = _stdout.lock().into_raw_mode().unwrap();
    let mut record_location = Vec::<f64>::new();
    let mut record_velocity = Vec::<f64>::new();
    let mut record_acceleration = Vec::<f64>::new();
    let mut record_voltage = Vec::<f64>::new();

    
    while floor_requests.len() > 0 {
        let now = Instant::now();
        let dt = now.duration_since(prev_loop_time).as_secs_f64();
        prev_loop_time = Instant::now();


        velocity = velocity + (acceleration * dt);
        location = location + (velocity * dt);
        acceleration = {
            let f = (up_input_voltage - down_input_voltage) * 8.0;
            let mass = 1200000.0;
            -9.8 + (f / mass)
        };

        let next_floor = *floor_requests.get(0).unwrap() as f64;
        

        if (location - (next_floor * floor_height)).abs() < 0.01 && velocity.abs() < 0.01 {
            velocity = 0.0;
            floor_requests.pop_front();
        }

        // Time to taken to decerate at -1.0m/s^2
        let t = velocity.abs() / 1.0;

        // Distance that will be covered while decelerating

        let d = t * (velocity / 2.0);

        // Distance from destination

        let l = (location - (next_floor * floor_height)).abs();


        let target_acceleration = {
            // Check if we are going up
            let going_up = location < (next_floor * floor_height);
            if velocity.abs() >= 5.0 {
                if (going_up && velocity > 0.0) || (!going_up && velocity < 0.0) {
                    0.0
                } else if going_up {
                    1.0
                } else {
                    -1.0
                }
            } else if l < d && going_up == (velocity > 0.0) {
                if going_up {
                    -1.0
                } else {
                    1.0
                }
            } else {
                if going_up {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        let gravity_adjusted_acceleration = target_acceleration + 9.8;
        let force_required = 1200000.00 * gravity_adjusted_acceleration;
        let target_voltage: f64 = force_required / 8.0;

        if target_voltage > 0.0 {
            up_input_voltage = target_voltage;
            down_input_voltage = 0.0;
        } else {
            up_input_voltage = 0.0;
            down_input_voltage = target_voltage.abs();
        }

        //Print realtime statistics
        print!(
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        );
        let carriage_floor = (location / floor_height).floor() as u64;
        let carriage_floor = cmp::max(carriage_floor, 0);
        let carriage_floor = cmp::min(carriage_floor, (floor_count - 1) as u64);

   
        let mut terminal_buf = vec![' ' as u8; (termwidth * termheight) as usize];
        for ty in 0..floor_count {
            terminal_buf[(ty * termwidth  + 0) as usize] = '[' as u8;
            terminal_buf[(ty * termwidth + 1) as usize] =
                if (ty as u64) == ((floor_count - 1) - carriage_floor) {
                    'X' as u8
                } else {
                    ' ' as u8
                };
            terminal_buf[(ty * termwidth + 2) as usize] = ']' as u8;
            terminal_buf[(ty * termwidth + termwidth - 2) as usize] = '\r' as u8;
            terminal_buf[(ty * termwidth + termwidth - 1) as usize] = '\n' as u8;
        }
        let stats = vec![
            format!("Carriage at floor {}", carriage_floor + 1),
            format!("Location    {:.06}",location),
            format!("Velocity    {:.06}",velocity),
            format!("Acceleration      {:.06}",acceleration),
            format!("Voltage [up-down] {:.06}", up_input_voltage - down_input_voltage),
        ];

        for sy in 0..stats.len() {
            for (sx, sc) in stats[sy].chars().enumerate() {
                terminal_buf[sy * (termwidth as usize) + 6 + sx] = sc as u8;
            }
        }
  
        write!(stdout, "{}", String::from_utf8(terminal_buf).unwrap()).unwrap();
        stdout.flush().unwrap(); 
        record_acceleration.push(acceleration);
        record_location.push(location);
        record_velocity.push(velocity);
        record_voltage.push(up_input_voltage - down_input_voltage);
        thread::sleep(Duration::from_millis(10));
    }
    write!(stdout, "{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Show).unwrap();
    print_variable_stats(&mut stdout, "Location", variable_stats(&record_location));
    print_variable_stats(&mut stdout, "Velocity", variable_stats(&record_velocity));
    print_variable_stats(&mut stdout, "Voltage", variable_stats(&record_voltage));
    print_variable_stats(&mut stdout, "Acceleration", variable_stats(&record_acceleration));
    stdout.flush().unwrap(); 
}

pub fn variable_stats(data: &Vec<f64>) -> (f64, f64) {
    let avg = data.into_iter().fold(0.0, |acc, val| acc + *val) / data.len() as f64;
    let dev = data.into_iter().fold(0.0, |acc, val| (acc + (*val - avg).powi(2)));
    let dev = (dev / (data.len() as f64)).sqrt();
    (avg, dev)
}

fn print_variable_stats (stdout: &mut RawTerminal<StdoutLock>, vname: &str, (avg, dev):(f64, f64)) {
    use std::io::Write;
    write!(stdout, "Average of {}:  {}\r\n", vname, avg).unwrap();
    write!(stdout, "Standard deviation of {}: {:.06}\r\n", vname, dev ).unwrap();
    write!(stdout, "\r\n").unwrap();
}
